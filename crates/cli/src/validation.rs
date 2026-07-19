//! Release-gate validation over the existing deterministic architecture.
//!
//! This module intentionally exercises adapters, Semantic IR, reasoning,
//! diagnostics, cache, query, and benchmark boundaries without adding rules or
//! changing their behavior.

use std::{
    fmt, fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

use serde::{Deserialize, Serialize};
use vz_adapter_vinglish::VinglishAdapter;
use vz_diagnostics::{model::Interpretation, CompilerDiagnostic, SemanticDiagnosticEngine};
use vz_reasoning::{
    incremental::{CacheWrite, CachedFunction, SemanticCache},
    ReasoningEngine,
};

use crate::{benchmark, profile, query, registry::default_registry, verification};

const EXPECTATIONS: &str = "verification/expected/semantic-corpus.json";
const ACCUMULATOR_FIXTURE: &str = "tests/fixtures/accumulate-v1.json";
const TYPE_MISMATCH_FIXTURE: &str = "tests/fixtures/type-mismatch.json";
const MISSING_DIVISION_FIXTURE: &str = "tests/fixtures/missing-division.json";
const PERFORMANCE_BUDGET: &str = "verification/expected/performance-budget.json";
static TEMPORARY_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Serialize)]
pub struct ValidationReport {
    pub version: u32,
    pub semantic_patterns: CategoryResult,
    pub pipelines: CategoryResult,
    pub diagnostics: CategoryResult,
    pub queries: CategoryResult,
    pub cache: CategoryResult,
    pub performance: PerformanceResult,
    pub stress: Vec<StressResult>,
    pub frontends: Vec<FrontendResult>,
}

#[derive(Debug, Serialize)]
pub struct CategoryResult {
    pub passed: usize,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct PerformanceResult {
    pub benchmark_samples: usize,
    pub profile_cold_query_nanos: u128,
    pub profile_warm_query_nanos: u128,
    pub profile_cache_hits: usize,
    pub profile_cache_misses: usize,
}

#[derive(Debug, Serialize)]
pub struct StressResult {
    pub functions: usize,
    pub elapsed_nanos: u128,
    pub functions_per_second: u64,
}

#[derive(Debug, Serialize)]
pub struct FrontendResult {
    pub language: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Deserialize)]
struct GoldenCorpus {
    version: u32,
    patterns: Vec<GoldenPattern>,
}

#[derive(Debug, Deserialize)]
struct GoldenPattern {
    id: String,
    primary_intent: String,
    minimum_confidence: u8,
}

#[derive(Debug, Deserialize)]
struct PerformanceBudget {
    version: u32,
    max_average_benchmark_nanos: u64,
    max_cold_query_nanos: u128,
    max_warm_query_nanos: u128,
}

#[derive(Debug)]
pub enum ValidationError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Decode {
        path: PathBuf,
        source: serde_json::Error,
    },
    Verification(verification::VerificationError),
    GoldenVersion(u32),
    MissingGolden(String),
    UnexpectedPrimary {
        pattern: String,
        language: String,
        expected: String,
        actual: Option<String>,
    },
    Confidence {
        pattern: String,
        language: String,
        minimum: u8,
        actual: Option<u8>,
    },
    Pipeline {
        language: String,
        actual: Vec<String>,
    },
    Diagnostic(String),
    Query {
        expression: String,
        detail: String,
    },
    Cache(String),
    Benchmark(benchmark::BenchmarkError),
    Profile(String),
    PerformanceBudget(String),
    Adapter(String),
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    Serialize(serde_json::Error),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } | Self::Write { path, source } => {
                write!(formatter, "cannot access '{}': {source}", path.display())
            }
            Self::Decode { path, source } => {
                write!(formatter, "cannot decode '{}': {source}", path.display())
            }
            Self::Verification(error) => write!(formatter, "cross-language verification: {error}"),
            Self::GoldenVersion(version) => {
                write!(
                    formatter,
                    "unsupported semantic golden corpus version: {version}"
                )
            }
            Self::MissingGolden(pattern) => {
                write!(formatter, "missing golden expectation for '{pattern}'")
            }
            Self::UnexpectedPrimary {
                pattern,
                language,
                expected,
                actual,
            } => write!(
                formatter,
                "{language} '{pattern}' primary intent is '{}' instead of '{expected}'",
                actual.as_deref().unwrap_or("none")
            ),
            Self::Confidence {
                pattern,
                language,
                minimum,
                actual,
            } => write!(
                formatter,
                "{language} '{pattern}' confidence is '{}' but must be at least {minimum}",
                actual.map_or_else(|| "none".to_owned(), |value| value.to_string())
            ),
            Self::Pipeline { language, actual } => write!(
                formatter,
                "{language} filter-map-reduce pipeline is [{}]",
                actual.join(", ")
            ),
            Self::Diagnostic(message)
            | Self::Cache(message)
            | Self::Adapter(message)
            | Self::PerformanceBudget(message) => formatter.write_str(message),
            Self::Query { expression, detail } => {
                write!(
                    formatter,
                    "query '{expression}' failed validation: {detail}"
                )
            }
            Self::Benchmark(error) => write!(formatter, "benchmark: {error}"),
            Self::Profile(error) => write!(formatter, "profile: {error}"),
            Self::Serialize(error) => {
                write!(formatter, "cannot serialize validation report: {error}")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

pub fn run() -> Result<ValidationReport, ValidationError> {
    let root = repository_root();
    let goldens = load_goldens(&root)?;
    let verification = verification::run().map_err(ValidationError::Verification)?;
    let semantic_patterns = validate_semantic_patterns(&verification, &goldens)?;
    let pipelines = validate_pipelines(&verification)?;
    let diagnostics = validate_diagnostics(&root)?;
    let queries = validate_queries(&root)?;
    let cache = validate_cache(&root)?;
    let frontends = validate_frontend_boundaries()?;
    let stress = run_stress(&root)?;
    let benchmark = benchmark::run().map_err(ValidationError::Benchmark)?;
    let profile = profile::run(&root).map_err(ValidationError::Profile)?;
    validate_performance_budget(&root, &benchmark, &profile)?;
    let report = ValidationReport {
        version: 1,
        semantic_patterns,
        pipelines,
        diagnostics,
        queries,
        cache,
        performance: PerformanceResult {
            benchmark_samples: benchmark.summary.samples,
            profile_cold_query_nanos: profile.cold_query_nanos,
            profile_warm_query_nanos: profile.warm_query_nanos,
            profile_cache_hits: profile.cache_hits,
            profile_cache_misses: profile.cache_misses,
        },
        stress,
        frontends,
    };
    write_report(&root, &report)?;
    Ok(report)
}

fn validate_performance_budget(
    root: &Path,
    benchmark: &benchmark::BenchmarkReport,
    profile: &profile::ProfileReport,
) -> Result<(), ValidationError> {
    let path = root.join(PERFORMANCE_BUDGET);
    let input = fs::read_to_string(&path).map_err(|source| ValidationError::Read {
        path: path.clone(),
        source,
    })?;
    let budget: PerformanceBudget =
        serde_json::from_str(&input).map_err(|source| ValidationError::Decode { path, source })?;
    if budget.version != 1 {
        return Err(ValidationError::PerformanceBudget(format!(
            "unsupported performance budget version: {}",
            budget.version
        )));
    }
    if benchmark.summary.average_total_nanos() > budget.max_average_benchmark_nanos
        || profile.cold_query_nanos > budget.max_cold_query_nanos
        || profile.warm_query_nanos > budget.max_warm_query_nanos
    {
        return Err(ValidationError::PerformanceBudget(format!(
            "performance budget exceeded: average benchmark {} ns (max {}), cold query {} ns (max {}), warm query {} ns (max {})",
            benchmark.summary.average_total_nanos(),
            budget.max_average_benchmark_nanos,
            profile.cold_query_nanos,
            budget.max_cold_query_nanos,
            profile.warm_query_nanos,
            budget.max_warm_query_nanos,
        )));
    }
    Ok(())
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_goldens(root: &Path) -> Result<GoldenCorpus, ValidationError> {
    let path = root.join(EXPECTATIONS);
    let input = fs::read_to_string(&path).map_err(|source| ValidationError::Read {
        path: path.clone(),
        source,
    })?;
    let corpus: GoldenCorpus =
        serde_json::from_str(&input).map_err(|source| ValidationError::Decode { path, source })?;
    if corpus.version != 1 {
        return Err(ValidationError::GoldenVersion(corpus.version));
    }
    Ok(corpus)
}

fn validate_semantic_patterns(
    report: &verification::VerificationReport,
    goldens: &GoldenCorpus,
) -> Result<CategoryResult, ValidationError> {
    let mut passed = 0;
    for pattern in &report.patterns {
        if pattern.pattern == "filter_map_reduce" {
            continue;
        }
        let golden = goldens
            .patterns
            .iter()
            .find(|candidate| candidate.id == pattern.pattern)
            .ok_or_else(|| ValidationError::MissingGolden(pattern.pattern.clone()))?;
        for entry in &pattern.entries {
            if entry.primary_intent.as_deref() != Some(golden.primary_intent.as_str()) {
                return Err(ValidationError::UnexpectedPrimary {
                    pattern: pattern.pattern.clone(),
                    language: entry.language.clone(),
                    expected: golden.primary_intent.clone(),
                    actual: entry.primary_intent.clone(),
                });
            }
            if entry.confidence.unwrap_or(0) < golden.minimum_confidence {
                return Err(ValidationError::Confidence {
                    pattern: pattern.pattern.clone(),
                    language: entry.language.clone(),
                    minimum: golden.minimum_confidence,
                    actual: entry.confidence,
                });
            }
        }
        passed += 1;
    }
    if goldens.patterns.len() != passed {
        return Err(ValidationError::MissingGolden(
            "golden corpus contains stale or missing patterns".to_owned(),
        ));
    }
    Ok(CategoryResult {
        passed,
        total: goldens.patterns.len(),
    })
}

fn validate_pipelines(
    report: &verification::VerificationReport,
) -> Result<CategoryResult, ValidationError> {
    let expected = ["filter", "mapper", "reducer"];
    let pipeline = report
        .patterns
        .iter()
        .find(|pattern| pattern.pattern == "filter_map_reduce")
        .ok_or_else(|| ValidationError::Pipeline {
            language: "corpus".to_owned(),
            actual: Vec::new(),
        })?;
    for entry in &pipeline.entries {
        if entry
            .semantic_pipeline
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            != expected
        {
            return Err(ValidationError::Pipeline {
                language: entry.language.clone(),
                actual: entry.semantic_pipeline.clone(),
            });
        }
    }
    Ok(CategoryResult {
        passed: pipeline.entries.len(),
        total: pipeline.entries.len(),
    })
}

fn validate_diagnostics(root: &Path) -> Result<CategoryResult, ValidationError> {
    let engine = ReasoningEngine::new();
    let graph = import_transport(root.join(ACCUMULATOR_FIXTURE))?;
    let report = engine.analyze(&graph);
    let diagnostic_engine = SemanticDiagnosticEngine::new();
    let type_mismatch = load_diagnostic(root.join(TYPE_MISMATCH_FIXTURE))?;
    let type_result = diagnostic_engine
        .diagnose(type_mismatch, &report)
        .map_err(|error| ValidationError::Diagnostic(error.to_string()))?;
    if type_result.interpretation != Interpretation::AccumulatorTypeConflict
        || type_result.suggestions.len() != 3
    {
        return Err(ValidationError::Diagnostic(
            "type mismatch did not produce the accumulator diagnostic contract".to_owned(),
        ));
    }
    let missing_division = load_diagnostic(root.join(MISSING_DIVISION_FIXTURE))?;
    let division_result = diagnostic_engine
        .diagnose(missing_division, &report)
        .map_err(|error| ValidationError::Diagnostic(error.to_string()))?;
    if division_result.interpretation != Interpretation::AverageImplementationIncomplete
        || division_result.suggestions.len() != 1
    {
        return Err(ValidationError::Diagnostic(
            "missing division did not produce the average diagnostic contract".to_owned(),
        ));
    }
    Ok(CategoryResult {
        passed: 2,
        total: 2,
    })
}

fn import_transport(path: PathBuf) -> Result<vz_semantic_ir::SemanticGraph, ValidationError> {
    let input = fs::read_to_string(&path).map_err(|source| ValidationError::Read {
        path: path.clone(),
        source,
    })?;
    VinglishAdapter
        .import_json(&input)
        .map_err(|error| ValidationError::Diagnostic(error.to_string()))
}

fn load_diagnostic(path: PathBuf) -> Result<CompilerDiagnostic, ValidationError> {
    let input = fs::read_to_string(&path).map_err(|source| ValidationError::Read {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&input).map_err(|source| ValidationError::Decode { path, source })
}

fn validate_queries(root: &Path) -> Result<CategoryResult, ValidationError> {
    let cases = [
        "accumulator",
        "histogram",
        "validator",
        "filter THEN reduce",
        "group",
        "binary search",
    ];
    for expression in cases {
        let report = query::run(expression, root).map_err(|detail| ValidationError::Query {
            expression: expression.to_owned(),
            detail,
        })?;
        for language_fragment in [
            "examples/python",
            "examples/java",
            "examples/c",
            "tests/fixtures",
        ] {
            if !report
                .matches
                .iter()
                .any(|entry| entry.source.contains(language_fragment))
            {
                let observed = report
                    .matches
                    .iter()
                    .map(|entry| entry.source.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(ValidationError::Query {
                    expression: expression.to_owned(),
                    detail: format!(
                        "missing a match from {language_fragment}; observed sources: {observed}"
                    ),
                });
            }
        }
    }
    Ok(CategoryResult {
        passed: cases.len(),
        total: cases.len(),
    })
}

fn validate_cache(root: &Path) -> Result<CategoryResult, ValidationError> {
    let graph = import_transport(root.join(ACCUMULATOR_FIXTURE))?;
    let engine = ReasoningEngine::new();
    let facts = engine.facts_for(&graph).functions.remove(0);
    let report = engine.analyze_function(&facts);
    let directory = std::env::temp_dir().join(format!(
        "vinglish-zero-validation-cache-{}-{}",
        std::process::id(),
        TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let cache = SemanticCache::open(&directory)
        .map_err(|error| ValidationError::Cache(error.to_string()))?;
    let entry = CachedFunction::new("validation::calculate".to_owned(), 1, facts, report);
    if cache
        .store(&entry)
        .map_err(|error| ValidationError::Cache(error.to_string()))?
        != CacheWrite::Created
    {
        return Err(ValidationError::Cache(
            "cold semantic cache write was not created".to_owned(),
        ));
    }
    if cache
        .store(&entry)
        .map_err(|error| ValidationError::Cache(error.to_string()))?
        != CacheWrite::Unchanged
    {
        return Err(ValidationError::Cache(
            "warm semantic cache write was not reused".to_owned(),
        ));
    }
    let cached = cache
        .load("validation::calculate")
        .map_err(|error| ValidationError::Cache(error.to_string()))?
        .ok_or_else(|| ValidationError::Cache("cached semantic entry disappeared".to_owned()))?;
    if !cached.is_compatible(1) || cached.is_compatible(2) {
        return Err(ValidationError::Cache(
            "source hash invalidation is not deterministic".to_owned(),
        ));
    }
    let mut incompatible = cached.clone();
    incompatible.semantic_ir_version += 1;
    if incompatible.is_schema_compatible() {
        return Err(ValidationError::Cache(
            "schema version invalidation did not trigger".to_owned(),
        ));
    }
    let mut changed = cached;
    changed.source_hash = 2;
    if !matches!(
        cache
            .store(&changed)
            .map_err(|error| ValidationError::Cache(error.to_string()))?,
        CacheWrite::Updated { .. }
    ) {
        return Err(ValidationError::Cache(
            "semantic cache update did not emit a delta".to_owned(),
        ));
    }
    let _ = fs::remove_dir_all(&directory);
    Ok(CategoryResult {
        passed: 5,
        total: 5,
    })
}

fn validate_frontend_boundaries() -> Result<Vec<FrontendResult>, ValidationError> {
    let registry =
        default_registry().map_err(|error| ValidationError::Adapter(error.to_string()))?;
    let unavailable = [
        ("Rust", "fixture.rs"),
        ("JavaScript", "fixture.js"),
        ("TypeScript", "fixture.ts"),
    ];
    unavailable
        .into_iter()
        .map(|(language, fixture)| {
            let error = registry
                .semantic_graph(Path::new(fixture))
                .expect_err("unavailable frontend must not return a graph")
                .to_string();
            if !error.contains("frontend is unavailable") {
                return Err(ValidationError::Adapter(format!(
                    "{language} did not return the deterministic unavailable-frontend error"
                )));
            }
            Ok(FrontendResult {
                language: language.to_owned(),
                status: "unavailable".to_owned(),
                detail: error,
            })
        })
        .collect()
}

fn run_stress(root: &Path) -> Result<Vec<StressResult>, ValidationError> {
    let graph = import_transport(root.join(ACCUMULATOR_FIXTURE))?;
    let engine = ReasoningEngine::new();
    let baseline =
        serde_json::to_vec(&engine.analyze(&graph)).map_err(ValidationError::Serialize)?;
    [100_usize, 500, 1_000]
        .into_iter()
        .map(|functions| {
            let start = Instant::now();
            for _ in 0..functions {
                let current = serde_json::to_vec(&engine.analyze(&graph))
                    .map_err(ValidationError::Serialize)?;
                if current != baseline {
                    return Err(ValidationError::Cache(
                        "stress corpus produced non-deterministic intent output".to_owned(),
                    ));
                }
            }
            let elapsed_nanos = start.elapsed().as_nanos();
            Ok(StressResult {
                functions,
                elapsed_nanos,
                functions_per_second: (functions as u128 * 1_000_000_000)
                    .checked_div(elapsed_nanos)
                    .unwrap_or(0) as u64,
            })
        })
        .collect()
}

fn write_report(root: &Path, report: &ValidationReport) -> Result<(), ValidationError> {
    let path = root.join("verification/validation-report.md");
    let frontend_rows = report
        .frontends
        .iter()
        .map(|frontend| {
            format!(
                "| {} | {} | {} |",
                frontend.language, frontend.status, frontend.detail
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let stress_rows = report
        .stress
        .iter()
        .map(|entry| {
            format!(
                "| {} | {} | {} |",
                entry.functions, entry.elapsed_nanos, entry.functions_per_second
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let body = format!(
        "# Validation Report\n\n## Overall\n\nPASS\n\n## Semantic Patterns\n\n- Passed: {} / {}\n\n## Pipelines\n\n- Passed: {} / {}\n\n## Diagnostics\n\n- Passed: {} / {}\n\n## Queries\n\n- Passed: {} / {}\n\n## Cache\n\n- Passed: {} / {}\n\n## Performance\n\n- Benchmark samples: {}\n- Cold query: {} ns\n- Warm query: {} ns\n- Cache hits: {}\n- Cache misses: {}\n\n## Stress\n\n| Functions | Elapsed (ns) | Functions/sec |\n| ---: | ---: | ---: |\n{}\n\n## Frontend Availability\n\n| Language | Status | Detail |\n| --- | --- | --- |\n{}\n\n## Known Limitations\n\nRust, JavaScript, and TypeScript are registered extension points but do not yet include official frontend integrations. Their deterministic unavailable-frontend diagnostics are validated here; they are not counted as analyzed languages. Performance data is host-dependent; validation enforces a deliberately broad release budget rather than a machine-specific microbenchmark baseline.\n",
        report.semantic_patterns.passed,
        report.semantic_patterns.total,
        report.pipelines.passed,
        report.pipelines.total,
        report.diagnostics.passed,
        report.diagnostics.total,
        report.queries.passed,
        report.queries.total,
        report.cache.passed,
        report.cache.total,
        report.performance.benchmark_samples,
        report.performance.profile_cold_query_nanos,
        report.performance.profile_warm_query_nanos,
        report.performance.profile_cache_hits,
        report.performance.profile_cache_misses,
        stress_rows,
        frontend_rows,
    );
    fs::write(&path, body).map_err(|source| ValidationError::Write { path, source })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_corpus_covers_every_current_core_contract_once() {
        let corpus = load_goldens(&repository_root()).unwrap();
        assert_eq!(corpus.patterns.len(), 31);
        assert!(corpus
            .patterns
            .iter()
            .any(|pattern| pattern.id == "binary_search"));
        assert!(corpus
            .patterns
            .iter()
            .all(|pattern| pattern.minimum_confidence > 0));
    }

    #[test]
    fn unavailable_frontends_keep_a_stable_failure_boundary() {
        let frontends = validate_frontend_boundaries().unwrap();
        assert_eq!(frontends.len(), 3);
        assert!(frontends
            .iter()
            .all(|frontend| frontend.status == "unavailable"));
    }
}
