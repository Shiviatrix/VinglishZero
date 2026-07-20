//! Repository-wide deterministic semantic query command.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use serde::Serialize;
use vz_adapters::AdapterRegistry;
use vz_reasoning::{
    incremental::{semantic_hash, stable_hash, CachedFunction, SemanticCache},
    parse_query, search_query, IntentReport, QueryMatch, ReasoningEngine,
};

use crate::registry::default_registry;

#[derive(Debug, Serialize)]
pub struct QueryReport {
    pub version: u32,
    pub expression: String,
    pub files_scanned: usize,
    pub files_analyzed: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub latency_nanos: u128,
    pub matches: Vec<QueryMatch>,
    pub skipped: Vec<QuerySkip>,
}

#[derive(Debug, Serialize)]
pub struct QuerySkip {
    pub path: String,
    pub reason: String,
}

pub fn run(expression: &str, root: &Path) -> Result<QueryReport, String> {
    run_with_cache(expression, root, &root.join(".vinglish-zero/cache"))
}

/// Runs a query with an explicit cache location. The normal CLI path uses the
/// repository cache; profiling uses an isolated cache to measure a true cold
/// pass without disturbing user state.
pub(crate) fn run_with_cache(
    expression: &str,
    root: &Path,
    cache_root: &Path,
) -> Result<QueryReport, String> {
    let query = parse_query(expression).map_err(|error| error.to_string())?;
    let start = Instant::now();
    let registry = default_registry().map_err(|error| error.to_string())?;
    let cache = SemanticCache::open(cache_root).map_err(|error| error.to_string())?;
    let recovered_cache = cache
        .load_all_recovering()
        .map_err(|error| error.to_string())?;
    let cached = recovered_cache.entries;
    let mut paths = Vec::new();
    collect_sources(root, &registry, &mut paths).map_err(|error| error.to_string())?;
    paths.sort();

    let engine = ReasoningEngine::new();
    let mut matches = Vec::new();
    let mut skipped = recovered_cache
        .invalid_blobs
        .into_iter()
        .map(|entry| QuerySkip {
            path: entry.path.display().to_string(),
            reason: format!("ignored invalid semantic cache blob: {}", entry.reason),
        })
        .collect::<Vec<_>>();
    let files_scanned = paths.len();
    let mut files_analyzed = 0;
    let mut cache_hits = 0;
    let mut cache_misses = 0;
    for path in paths {
        let source = path.display().to_string();
        let source_hash = match fs::read(&path) {
            Ok(bytes) => stable_hash(&bytes),
            Err(error) => {
                skipped.push(QuerySkip {
                    path: source,
                    reason: format!("cannot read source: {error}"),
                });
                continue;
            }
        };
        let prefix = format!("{source}::");
        let existing = cached
            .iter()
            .filter(|entry| entry.stable_id.starts_with(&prefix))
            .collect::<Vec<_>>();
        if !existing.is_empty()
            && existing
                .iter()
                .all(|entry| entry.is_compatible(source_hash))
        {
            cache_hits += existing.len();
            let report = IntentReport {
                version: 1,
                function_reports: existing
                    .iter()
                    .map(|entry| entry.intent_report.clone())
                    .collect(),
            };
            matches.extend(search_query(&source, &report, &query));
            continue;
        }
        // Source files and the compiler-owned Vinglish transport share the
        // same registry boundary. Querying must not grow a second importer.
        let graph = registry
            .semantic_graph(&path)
            .map_err(|error| error.to_string());
        match graph {
            Ok(graph) => {
                files_analyzed += 1;
                let facts = engine.facts_for(&graph);
                let reports = facts
                    .functions
                    .iter()
                    .enumerate()
                    .map(|(index, facts)| {
                        let stable_id = format!("{source}::{index}::{}", facts.function_name);
                        if let Some(entry) = existing.iter().find(|entry| {
                            entry.stable_id == stable_id
                                && entry.is_schema_compatible()
                                && entry.semantic_hash == semantic_hash(facts)
                        }) {
                            cache_hits += 1;
                            entry.intent_report.clone()
                        } else {
                            cache_misses += 1;
                            engine.analyze_function(facts)
                        }
                    })
                    .collect::<Vec<_>>();
                for ((index, facts), report) in facts.functions.iter().enumerate().zip(&reports) {
                    let entry = CachedFunction::new(
                        format!("{source}::{index}::{}", facts.function_name),
                        source_hash,
                        facts.clone(),
                        report.clone(),
                    );
                    cache.store(&entry).map_err(|error| error.to_string())?;
                }
                let report = IntentReport {
                    version: 1,
                    function_reports: reports,
                };
                matches.extend(search_query(&source, &report, &query));
            }
            Err(reason) => skipped.push(QuerySkip {
                path: path.display().to_string(),
                reason,
            }),
        }
    }
    matches.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.source.cmp(&right.source))
            .then_with(|| left.function_name.cmp(&right.function_name))
    });
    Ok(QueryReport {
        version: 1,
        expression: expression.to_owned(),
        files_scanned,
        files_analyzed,
        cache_hits,
        cache_misses,
        latency_nanos: start.elapsed().as_nanos(),
        matches,
        skipped,
    })
}

fn collect_sources(
    root: &Path,
    registry: &AdapterRegistry,
    paths: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() && !is_excluded_directory(&path) {
            collect_sources(&path, registry, paths)?;
        } else if file_type.is_file() && is_supported(&path, registry) {
            paths.push(path);
        }
    }
    Ok(())
}

fn is_excluded_directory(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(
            ".git"
                | ".idea"
                | ".venv"
                | ".vinglish-zero"
                | ".vscode"
                | "__pycache__"
                | "node_modules"
                | "target"
                | "verification"
        )
    )
}

fn is_supported(path: &Path, registry: &AdapterRegistry) -> bool {
    let Ok(adapter) = registry.adapter_for(path) else {
        return false;
    };
    adapter.capabilities().emits_semantic_ir && (!is_transport(path) || is_vinglish_transport(path))
}

fn is_transport(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
}

fn is_vinglish_transport(path: &Path) -> bool {
    fs::read_to_string(path)
        .ok()
        .and_then(|input| serde_json::from_str::<serde_json::Value>(&input).ok())
        .and_then(|document| {
            document
                .get("format")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .as_deref()
        == Some("vinglish.semantic-export")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_finds_the_cross_language_composition_fixture() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = run("filter THEN reduce", &root).unwrap();
        assert!(report
            .matches
            .iter()
            .any(|entry| entry.function_name == "filter_map_reduce"));
    }

    #[test]
    fn source_discovery_uses_operational_frontends_and_transport_documents_only() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let registry = default_registry().unwrap();

        assert!(is_supported(
            &root.join("tests/fixtures/accumulate-v1.json"),
            &registry
        ));
        assert!(!is_supported(
            &root.join("tests/fixtures/type-mismatch.json"),
            &registry
        ));
        assert!(!is_supported(Path::new("example.rs"), &registry));
    }

    #[test]
    fn query_recovers_from_a_corrupt_cache_blob() {
        let sequence = std::sync::atomic::AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "vz-query-cache-recovery-{}-{}",
            std::process::id(),
            sequence.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        fs::copy(
            repository.join("tests/fixtures/accumulate-v1.json"),
            root.join("accumulate-v1.json"),
        )
        .unwrap();
        let cache_root = root.join(".vinglish-zero/cache");
        fs::create_dir_all(&cache_root).unwrap();
        fs::write(cache_root.join("corrupt.bin"), b"invalid").unwrap();

        let report = run_with_cache("accumulator", &root, &cache_root).unwrap();

        assert!(report
            .matches
            .iter()
            .any(|entry| entry.function_name == "calculate"));
        assert!(report.skipped.iter().any(|entry| entry
            .reason
            .starts_with("ignored invalid semantic cache blob:")));
        let _ = fs::remove_dir_all(root);
    }
}
