//! Lightweight built-in, deterministic performance profiler.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

use serde::Serialize;
use vz_reasoning::incremental::{SemanticCache, SemanticCacheStats};

use crate::query;

static PROFILE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

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
    pub delta_bytes: u64,
    pub peak_rss_bytes: Option<u64>,
}

/// Persistent cache inventory returned by `vz stats` and `vz cache stats`.
/// It reads only cache metadata and never invokes an adapter or frontend.
#[derive(Debug, Serialize)]
pub struct CacheStatsReport {
    pub version: u32,
    pub cache_path: String,
    #[serde(flatten)]
    pub cache: SemanticCacheStats,
}

pub fn run(root: &Path) -> Result<ProfileReport, String> {
    let workspace = ProfileWorkspace::new(root)?;
    let start = Instant::now();
    let cold = query::run_with_cache("filter THEN reduce", root, workspace.cache_path())?;
    let cold_query_nanos = start.elapsed().as_nanos();
    let start = Instant::now();
    let warm = query::run_with_cache("filter THEN reduce", root, workspace.cache_path())?;
    let warm_query_nanos = start.elapsed().as_nanos();
    let cache_stats = SemanticCache::open(workspace.cache_path())
        .map_err(|error| error.to_string())?
        .stats()
        .map_err(|error| error.to_string())?;
    let report = ProfileReport {
        version: 1,
        cold_query_nanos,
        warm_query_nanos,
        cold_files_analyzed: cold.files_analyzed,
        warm_files_analyzed: warm.files_analyzed,
        cache_hits: warm.cache_hits,
        cache_misses: cold.cache_misses,
        cache_bytes: cache_stats.cache_bytes,
        semantic_blobs: cache_stats.semantic_blobs,
        average_blob_bytes: cache_stats.average_semantic_blob_bytes,
        delta_bytes: cache_stats.delta_bytes,
        peak_rss_bytes: rss_bytes(),
    };
    write_markdown(root, &report)?;
    Ok(report)
}

pub fn cache_stats(root: &Path) -> Result<CacheStatsReport, String> {
    let cache_path = root.join(".vinglish-zero/cache");
    let cache = SemanticCache::open(&cache_path).map_err(|error| error.to_string())?;
    Ok(CacheStatsReport {
        version: 1,
        cache_path: cache_path.display().to_string(),
        cache: cache.stats().map_err(|error| error.to_string())?,
    })
}

struct ProfileWorkspace {
    path: PathBuf,
}

impl ProfileWorkspace {
    fn new(root: &Path) -> Result<Self, String> {
        let parent = root.join(".vinglish-zero");
        fs::create_dir_all(&parent).map_err(|error| error.to_string())?;
        for _ in 0..16 {
            let sequence = PROFILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!("profile-{}-{sequence}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.to_string()),
            }
        }
        Err("cannot allocate an isolated profile cache directory".to_owned())
    }

    fn cache_path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ProfileWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
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
    let peak_rss = report
        .peak_rss_bytes
        .map(|bytes| format!("{bytes} bytes"))
        .unwrap_or_else(|| "unavailable on this platform".to_owned());
    let body = format!("# Performance Report\n\n## Overall Timings\n\n- Cold cache query: {} ns\n- Warm cache query: {} ns\n- Cold files analyzed: {}\n- Warm files analyzed: {}\n\n## Cache Effectiveness\n\n- Cache hits: {}\n- Cache misses: {}\n- Semantic blobs: {}\n- Cache size: {} bytes\n- Average semantic blob: {} bytes\n- Binary delta size: {} bytes\n\n## Memory\n\n- Peak RSS: {}\n\n## Method\n\nCold and warm queries run against the same isolated cache created for this profile invocation. The first pass starts empty; the second reuses only blobs produced by the first pass.\n", report.cold_query_nanos, report.warm_query_nanos, report.cold_files_analyzed, report.warm_files_analyzed, report.cache_hits, report.cache_misses, report.semantic_blobs, report.cache_bytes, report.average_blob_bytes, report.delta_bytes, peak_rss);
    fs::write(root.join("verification/performance-report.md"), body)
        .map_err(|error| error.to_string())
}
