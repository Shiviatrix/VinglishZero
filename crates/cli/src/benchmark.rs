//! Deterministic wall-clock benchmarking for the existing adapter and
//! reasoning pipeline.
//!
//! The benchmark measures the architecture boundary that exists today:
//! source adapter work up to `SemanticGraph`, then deterministic reasoning
//! over that graph. It does not change the pipeline or introduce a new
//! execution path.

use std::{
    fmt, fs,
    path::{Path, PathBuf},
    time::Instant,
};

use serde::Serialize;
use vz_reasoning::{parse_query, report::HypothesisStatus, search_query, ReasoningEngine};

use crate::registry::default_registry;

struct CorpusPattern {
    id: &'static str,
    function: &'static str,
    sources: &'static [(&'static str, &'static str)],
}

struct BenchmarkTarget<'a> {
    pattern: &'a str,
    function: &'a str,
    language: &'a str,
    relative: &'a str,
    source: &'a Path,
}

const ACCUMULATOR_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/accumulator.py"),
    ("Java", "examples/java/Accumulator.java"),
    ("C", "examples/c/accumulator.c"),
    ("Vinglish", "tests/fixtures/accumulate-v1.json"),
];
const MAXIMUM_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/maximum.py"),
    ("Java", "examples/java/Maximum.java"),
    ("C", "examples/c/maximum.c"),
    ("Vinglish", "tests/fixtures/maximum-v1.json"),
];
const COLLECTION_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/collection_patterns.py"),
    ("Java", "examples/java/CollectionPatterns.java"),
    ("C", "examples/c/collection_patterns.c"),
    ("Vinglish", "tests/fixtures/collection-patterns-v1.json"),
];
const COMPOSITION_SOURCES: &[(&str, &str)] = &[
    ("Python", "examples/python/compositions.py"),
    ("Java", "examples/java/CompositionPatterns.java"),
    ("C", "examples/c/compositions.c"),
    ("Vinglish", "tests/fixtures/compositions-v1.json"),
];
const CORPUS: &[CorpusPattern] = &[
    CorpusPattern {
        id: "accumulator",
        function: "calculate",
        sources: ACCUMULATOR_SOURCES,
    },
    CorpusPattern {
        id: "maximum",
        function: "find_maximum",
        sources: MAXIMUM_SOURCES,
    },
    CorpusPattern {
        id: "sum",
        function: "sum_values",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "minimum",
        function: "find_minimum",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "count_if",
        function: "count_if",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "any",
        function: "any_match",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "all",
        function: "all_positive",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "find_first",
        function: "find_first",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "find_last",
        function: "find_last",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "reverse_traversal",
        function: "reverse_traversal",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "prefix_sum",
        function: "prefix_sum",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "frequency_counter",
        function: "frequency_counter",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "histogram",
        function: "histogram",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "max_by",
        function: "max_by",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "min_by",
        function: "min_by",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "group_by",
        function: "group_by",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "partition",
        function: "partition",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "unique",
        function: "unique",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "distinct",
        function: "distinct",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "zip",
        function: "zip_items",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "flatten",
        function: "flatten",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "contains",
        function: "contains",
        sources: COLLECTION_SOURCES,
    },
    CorpusPattern {
        id: "filter_map_reduce",
        function: "filter_map_reduce",
        sources: COMPOSITION_SOURCES,
    },
];

const DEFAULT_ITERATIONS: usize = 3;

#[derive(Debug, Serialize)]
pub struct BenchmarkReport {
    version: u32,
    pub iterations: usize,
    entries: Vec<BenchmarkEntry>,
    pub summary: BenchmarkSummary,
}

#[derive(Debug, Serialize)]
struct BenchmarkEntry {
    pattern: String,
    language: String,
    source: String,
    graph: GraphSummary,
    primary_intent: Option<String>,
    confidence: Option<u8>,
    timings: Timings,
}

#[derive(Debug, Serialize)]
struct GraphSummary {
    nodes: usize,
    functions: usize,
}

#[derive(Debug, Serialize)]
struct Timings {
    frontend_nanos: u64,
    reasoning_nanos: u64,
    query_nanos: u64,
    total_nanos: u64,
}

#[derive(Debug, Serialize)]
pub struct BenchmarkSummary {
    pub samples: usize,
    average_frontend_nanos: u64,
    average_reasoning_nanos: u64,
    average_query_nanos: u64,
    average_total_nanos: u64,
    fastest_total_nanos: u64,
    slowest_total_nanos: u64,
}

#[derive(Debug)]
pub enum BenchmarkError {
    Registry(String),
    Adapter(String),
    Iterations(String),
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    Serialize(serde_json::Error),
}

impl fmt::Display for BenchmarkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registry(e) | Self::Adapter(e) | Self::Iterations(e) => f.write_str(e),
            Self::Write { path, source } => {
                write!(f, "cannot write '{}': {source}", path.display())
            }
            Self::Serialize(e) => write!(f, "cannot serialize report: {e}"),
        }
    }
}

impl std::error::Error for BenchmarkError {}

pub fn run() -> Result<BenchmarkReport, BenchmarkError> {
    let root = repo_root();
    run_with_roots(&root, &root)
}

#[cfg(test)]
pub fn run_in(root: &Path) -> Result<BenchmarkReport, BenchmarkError> {
    let source_root = repo_root();
    run_with_roots(root, &source_root)
}

fn run_with_roots(root: &Path, source_root: &Path) -> Result<BenchmarkReport, BenchmarkError> {
    let iterations = iterations_from_env()?;
    let registry =
        default_registry().map_err(|error| BenchmarkError::Registry(error.to_string()))?;
    let engine = ReasoningEngine::new();
    let mut entries = Vec::new();

    for pattern in CORPUS {
        for (language, relative) in pattern.sources {
            let source = source_root.join(relative);
            let entry = benchmark_source(
                &registry,
                &engine,
                BenchmarkTarget {
                    pattern: pattern.id,
                    function: pattern.function,
                    language,
                    relative,
                    source: &source,
                },
                iterations,
            )?;
            entries.push(entry);
        }
    }

    let summary = summarize(&entries);
    let report = BenchmarkReport {
        version: 1,
        iterations,
        entries,
        summary,
    };
    write_reports(root, &report)?;
    Ok(report)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn benchmark_source(
    registry: &vz_adapters::AdapterRegistry,
    engine: &ReasoningEngine,
    target: BenchmarkTarget<'_>,
    iterations: usize,
) -> Result<BenchmarkEntry, BenchmarkError> {
    let mut frontend_samples = Vec::with_capacity(iterations);
    let mut reasoning_samples = Vec::with_capacity(iterations);
    let mut query_samples = Vec::with_capacity(iterations);
    let mut total_samples = Vec::with_capacity(iterations);
    let mut graph = None;
    let mut intent = None;
    let mut confidence = None;

    for sample_index in 0..=iterations {
        let total_start = Instant::now();
        let frontend_start = Instant::now();
        let current_graph = registry
            .semantic_graph(target.source)
            .map_err(|error| BenchmarkError::Adapter(error.to_string()))?;
        let frontend_elapsed = frontend_start.elapsed();

        let reasoning_start = Instant::now();
        let current_report = engine.analyze(&current_graph);
        let reasoning_elapsed = reasoning_start.elapsed();

        let query_start = Instant::now();
        let query = parse_query(target.pattern).expect("benchmark patterns are valid queries");
        let _ = search_query(target.relative, &current_report, &query);
        let query_elapsed = query_start.elapsed();
        let total_elapsed = total_start.elapsed();

        if sample_index == 0 {
            graph = Some(GraphSummary {
                nodes: current_graph.len(),
                functions: current_report.function_reports.len(),
            });
            let primary = current_report
                .function_reports
                .iter()
                .find(|function| function.function_name == target.function)
                .and_then(|function| {
                    let intent = function
                        .hypotheses
                        .iter()
                        .find(|hypothesis| {
                            hypothesis.id == target.pattern
                                && hypothesis.status == HypothesisStatus::Active
                        })
                        .map(|hypothesis| hypothesis.id.as_str())
                        .or_else(|| {
                            function
                                .semantic_pipeline
                                .last()
                                .map(|stage| stage.hypothesis_id.as_str())
                        });
                    intent.and_then(|id| {
                        function
                            .hypotheses
                            .iter()
                            .find(|hypothesis| hypothesis.id == id)
                    })
                });
            intent = primary.map(|hypothesis| hypothesis.id.clone());
            confidence = primary.map(|hypothesis| hypothesis.confidence);
            continue;
        }

        frontend_samples.push(frontend_elapsed.as_nanos());
        reasoning_samples.push(reasoning_elapsed.as_nanos());
        query_samples.push(query_elapsed.as_nanos());
        total_samples.push(total_elapsed.as_nanos());
    }

    Ok(BenchmarkEntry {
        pattern: target.pattern.to_owned(),
        language: target.language.to_owned(),
        source: target.relative.to_owned(),
        graph: graph.unwrap_or(GraphSummary {
            nodes: 0,
            functions: 0,
        }),
        primary_intent: intent,
        confidence,
        timings: Timings {
            frontend_nanos: average_nanos(&frontend_samples),
            reasoning_nanos: average_nanos(&reasoning_samples),
            query_nanos: average_nanos(&query_samples),
            total_nanos: average_nanos(&total_samples),
        },
    })
}

fn summarize(entries: &[BenchmarkEntry]) -> BenchmarkSummary {
    let mut frontend = Vec::with_capacity(entries.len());
    let mut reasoning = Vec::with_capacity(entries.len());
    let mut query = Vec::with_capacity(entries.len());
    let mut total = Vec::with_capacity(entries.len());
    for entry in entries {
        frontend.push(entry.timings.frontend_nanos as u128);
        reasoning.push(entry.timings.reasoning_nanos as u128);
        query.push(entry.timings.query_nanos as u128);
        total.push(entry.timings.total_nanos as u128);
    }
    BenchmarkSummary {
        samples: entries.len(),
        average_frontend_nanos: average_nanos(&frontend),
        average_reasoning_nanos: average_nanos(&reasoning),
        average_query_nanos: average_nanos(&query),
        average_total_nanos: average_nanos(&total),
        fastest_total_nanos: total
            .iter()
            .copied()
            .min()
            .map(|value| value as u64)
            .unwrap_or(0),
        slowest_total_nanos: total
            .iter()
            .copied()
            .max()
            .map(|value| value as u64)
            .unwrap_or(0),
    }
}

fn average_nanos(samples: &[u128]) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let sum: u128 = samples.iter().copied().sum();
    u64::try_from(sum / samples.len() as u128).unwrap_or(u64::MAX)
}

fn iterations_from_env() -> Result<usize, BenchmarkError> {
    match std::env::var("VZ_BENCHMARK_ITERATIONS") {
        Ok(raw) => {
            let iterations = raw.parse::<usize>().map_err(|_| {
                BenchmarkError::Iterations(format!("invalid VZ_BENCHMARK_ITERATIONS value: {raw}"))
            })?;
            if iterations == 0 {
                return Err(BenchmarkError::Iterations(
                    "VZ_BENCHMARK_ITERATIONS must be greater than zero".to_owned(),
                ));
            }
            Ok(iterations)
        }
        Err(std::env::VarError::NotPresent) => Ok(DEFAULT_ITERATIONS),
        Err(error) => Err(BenchmarkError::Iterations(format!(
            "cannot read VZ_BENCHMARK_ITERATIONS: {error}"
        ))),
    }
}

fn write_reports(root: &Path, report: &BenchmarkReport) -> Result<(), BenchmarkError> {
    let directory = root.join("verification");
    fs::create_dir_all(&directory).map_err(|source| BenchmarkError::Write {
        path: directory.clone(),
        source,
    })?;

    let json = directory.join("benchmark-report.json");
    fs::write(
        &json,
        serde_json::to_vec_pretty(report).map_err(BenchmarkError::Serialize)?,
    )
    .map_err(|source| BenchmarkError::Write { path: json, source })?;

    let markdown = directory.join("benchmark-report.md");
    let body = render_markdown(report);
    fs::write(&markdown, body).map_err(|source| BenchmarkError::Write {
        path: markdown,
        source,
    })
}

fn render_markdown(report: &BenchmarkReport) -> String {
    let mut body = String::new();
    body.push_str("# Pipeline Benchmark\n\n");
    body.push_str(&format!(
        "Iterations per sample: **{}**\n\n",
        report.iterations
    ));
    body.push_str(&format!("Samples: **{}**\n\n", report.summary.samples));
    body.push_str(
        "| Pattern | Language | Source | Frontend (ns) | Reasoning (ns) | Query (ns) | Total (ns) | Intent |\n",
    );
    body.push_str("| --- | --- | --- | ---: | ---: | ---: | ---: | --- |\n");
    for entry in &report.entries {
        body.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            entry.pattern,
            entry.language,
            entry.source,
            entry.timings.frontend_nanos,
            entry.timings.reasoning_nanos,
            entry.timings.query_nanos,
            entry.timings.total_nanos,
            entry.primary_intent.as_deref().unwrap_or("none"),
        ));
    }
    body.push_str("\n## Summary\n\n");
    body.push_str(&format!(
        "- Average frontend time: {} ns\n",
        report.summary.average_frontend_nanos
    ));
    body.push_str(&format!(
        "- Average reasoning time: {} ns\n",
        report.summary.average_reasoning_nanos
    ));
    body.push_str(&format!(
        "- Average query time: {} ns\n",
        report.summary.average_query_nanos
    ));
    body.push_str(&format!(
        "- Average total time: {} ns\n",
        report.summary.average_total_nanos
    ));
    body.push_str(&format!(
        "- Fastest average sample: {} ns\n",
        report.summary.fastest_total_nanos
    ));
    body.push_str(&format!(
        "- Slowest average sample: {} ns\n",
        report.summary.slowest_total_nanos
    ));
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env, fs,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static NEXT_TEMPORARY_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

    struct TemporaryDirectory(PathBuf);

    impl TemporaryDirectory {
        fn new() -> Self {
            let sequence = NEXT_TEMPORARY_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = env::temp_dir().join(format!(
                "vz-cli-benchmark-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn benchmark_reports_wall_clock_timings_for_the_current_pipeline() {
        let directory = TemporaryDirectory::new();
        std::env::set_var("VZ_BENCHMARK_ITERATIONS", "1");

        let report = run_in(directory.path()).unwrap();

        assert_eq!(report.iterations, 1);
        assert_eq!(
            report.summary.samples,
            CORPUS
                .iter()
                .map(|pattern| pattern.sources.len())
                .sum::<usize>()
        );
        assert!(directory
            .path()
            .join("verification/benchmark-report.json")
            .exists());
        assert!(directory
            .path()
            .join("verification/benchmark-report.md")
            .exists());
    }
}
