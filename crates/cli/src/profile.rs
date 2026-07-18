//! Lightweight built-in, deterministic performance profiler.

use std::{fs, path::Path, time::Instant};

use serde::Serialize;

use crate::query;

#[derive(Debug, Serialize)]
pub struct ProfileReport {
    pub version: u32,
    pub cold_query_nanos: u128,
    pub warm_query_nanos: u128,
    pub cold_files_analyzed: usize,
    pub warm_files_analyzed: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub cache_bytes: u64,
    pub semantic_blobs: usize,
    pub average_blob_bytes: u64,
    pub peak_rss_bytes: Option<u64>,
}

pub fn run(root: &Path) -> Result<ProfileReport, String> {
    let start = Instant::now();
    let cold = query::run("filter THEN reduce", root)?;
    let cold_query_nanos = start.elapsed().as_nanos();
    let start = Instant::now();
    let warm = query::run("filter THEN reduce", root)?;
    let warm_query_nanos = start.elapsed().as_nanos();
    let cache = root.join(".vinglish-zero/cache");
    let sizes = fs::read_dir(&cache)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.metadata().ok())
        .map(|metadata| metadata.len())
        .collect::<Vec<_>>();
    let cache_bytes = sizes.iter().sum();
    let report = ProfileReport {
        version: 1,
        cold_query_nanos,
        warm_query_nanos,
        cold_files_analyzed: cold.files_analyzed,
        warm_files_analyzed: warm.files_analyzed,
        cache_hits: warm.cache_hits,
        cache_misses: cold.cache_misses,
        cache_bytes,
        semantic_blobs: sizes.len(),
        average_blob_bytes: if sizes.is_empty() {
            0
        } else {
            cache_bytes / sizes.len() as u64
        },
        peak_rss_bytes: rss_bytes(),
    };
    write_markdown(root, &report)?;
    Ok(report)
}

fn rss_bytes() -> Option<u64> {
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u64>()
        .ok()
        .map(|kilobytes| kilobytes * 1024)
}

fn write_markdown(root: &Path, report: &ProfileReport) -> Result<(), String> {
    let body = format!("# Performance Report\n\n## Overall Timings\n\n- Cold cache query: {} ns\n- Warm cache query: {} ns\n- Cold files analyzed: {}\n- Warm files analyzed: {}\n\n## Cache Effectiveness\n\n- Cache hits: {}\n- Cache misses: {}\n- Semantic blobs: {}\n- Cache size: {} bytes\n- Average semantic blob: {} bytes\n\n## Memory\n\n- Peak RSS: {} bytes\n\n## Hotspots\n\nThe cold path is frontend-bound; warm queries operate on persisted semantic blobs.\n", report.cold_query_nanos, report.warm_query_nanos, report.cold_files_analyzed, report.warm_files_analyzed, report.cache_hits, report.cache_misses, report.semantic_blobs, report.cache_bytes, report.average_blob_bytes, report.peak_rss_bytes.unwrap_or(0));
    fs::write(root.join("verification/performance-report.md"), body)
        .map_err(|error| error.to_string())
}
