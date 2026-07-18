//! Repository-wide deterministic semantic query command.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use serde::Serialize;
use vz_adapter_vinglish::VinglishAdapter;
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
    let query = parse_query(expression).map_err(|error| error.to_string())?;
    let start = Instant::now();
    let registry = default_registry().map_err(|error| error.to_string())?;
    let cache = SemanticCache::open(root.join(".vinglish-zero/cache"))
        .map_err(|error| error.to_string())?;
    let cached = cache.load_all().map_err(|error| error.to_string())?;
    let mut paths = Vec::new();
    collect_sources(root, &mut paths).map_err(|error| error.to_string())?;
    paths.sort();

    let engine = ReasoningEngine::new();
    let mut matches = Vec::new();
    let mut skipped = Vec::new();
    let files_scanned = paths.len();
    let mut files_analyzed = 0;
    let mut cache_hits = 0;
    let mut cache_misses = 0;
    for path in paths {
        let source = path.display().to_string();
        let source_hash = fs::read(&path)
            .map(|bytes| stable_hash(&bytes))
            .unwrap_or(0);
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
        let graph = if is_transport(&path) {
            fs::read_to_string(&path)
                .map_err(|error| error.to_string())
                .and_then(|input| {
                    VinglishAdapter
                        .import_json(&input)
                        .map_err(|error| error.to_string())
                })
        } else {
            registry
                .semantic_graph(&path)
                .map_err(|error| error.to_string())
        };
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

fn collect_sources(root: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            if !matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some(".git" | "target" | "verification")
            ) {
                collect_sources(&path, paths)?;
            }
        } else if is_supported(&path) {
            paths.push(path);
        }
    }
    Ok(())
}

fn is_supported(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("py" | "java" | "c" | "ving" | "json")
    )
}

fn is_transport(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("json")
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
}
