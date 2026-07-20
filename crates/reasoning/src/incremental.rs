//! Persistent, deterministic per-function semantic cache.

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use serde::{Deserialize, Serialize};

use crate::{
    facts::FunctionFacts,
    report::{FunctionIntentReport, SemanticStage},
};

pub const CACHE_VERSION: u16 = 1;
pub const SEMANTIC_IR_VERSION: u16 = 1;
pub const REASONING_ENGINE_VERSION: u16 = 1;
pub const HYPOTHESIS_REGISTRY_VERSION: u16 = 2;
const MAGIC: &[u8; 4] = b"VZSC";
static CACHE_WRITE_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedFunction {
    pub stable_id: String,
    pub source_hash: u64,
    pub semantic_hash: u64,
    pub semantic_ir_version: u16,
    pub reasoning_engine_version: u16,
    pub hypothesis_registry_version: u16,
    pub facts: FunctionFacts,
    pub intent_report: FunctionIntentReport,
    pub semantic_pipeline: Vec<SemanticStage>,
    pub diagnostics: Vec<serde_json::Value>,
    pub query_metadata: Vec<String>,
}

impl CachedFunction {
    pub fn new(
        stable_id: String,
        source_hash: u64,
        facts: FunctionFacts,
        intent_report: FunctionIntentReport,
    ) -> Self {
        let semantic_hash = semantic_hash(&facts);
        let semantic_pipeline = intent_report.semantic_pipeline.clone();
        let query_metadata = query_metadata(&intent_report);
        Self {
            stable_id,
            source_hash,
            semantic_hash,
            semantic_ir_version: SEMANTIC_IR_VERSION,
            reasoning_engine_version: REASONING_ENGINE_VERSION,
            hypothesis_registry_version: HYPOTHESIS_REGISTRY_VERSION,
            facts,
            intent_report,
            semantic_pipeline,
            diagnostics: Vec::new(),
            query_metadata,
        }
    }

    pub fn is_compatible(&self, source_hash: u64) -> bool {
        self.source_hash == source_hash && self.is_schema_compatible()
    }

    pub fn is_schema_compatible(&self) -> bool {
        self.semantic_ir_version == SEMANTIC_IR_VERSION
            && self.reasoning_engine_version == REASONING_ENGINE_VERSION
            && self.hypothesis_registry_version == HYPOTHESIS_REGISTRY_VERSION
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryDelta {
    pub prefix_len: u32,
    pub removed_len: u32,
    pub suffix_len: u32,
    pub replacement: Vec<u8>,
}

/// A cache blob that could not be decoded during a best-effort cache read.
///
/// Cache data is disposable: a malformed blob must never prevent unaffected
/// functions from being analyzed or queried. The path and stable error text
/// allow callers to surface the condition without treating it as source input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CacheLoadFailure {
    pub path: PathBuf,
    pub reason: String,
}

/// Valid cache entries plus blobs that were safely ignored during recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CacheLoad {
    pub entries: Vec<CachedFunction>,
    pub invalid_blobs: Vec<CacheLoadFailure>,
}

/// Deterministic summary of the persistent cache directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticCacheStats {
    pub cache_version: u16,
    pub semantic_blobs: usize,
    pub compatible_entries: usize,
    pub incompatible_entries: usize,
    pub invalid_blobs: Vec<CacheLoadFailure>,
    pub delta_blobs: usize,
    pub semantic_blob_bytes: u64,
    pub delta_bytes: u64,
    pub cache_bytes: u64,
    pub average_semantic_blob_bytes: u64,
}

pub fn delta(previous: &[u8], current: &[u8]) -> BinaryDelta {
    let prefix_len = previous
        .iter()
        .zip(current)
        .take_while(|(a, b)| a == b)
        .count();
    let suffix_len = previous[prefix_len..]
        .iter()
        .rev()
        .zip(current[prefix_len..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    BinaryDelta {
        prefix_len: prefix_len as u32,
        removed_len: (previous.len() - prefix_len - suffix_len) as u32,
        suffix_len: suffix_len as u32,
        replacement: current[prefix_len..current.len() - suffix_len].to_vec(),
    }
}

pub fn apply_delta(previous: &[u8], delta: &BinaryDelta) -> Option<Vec<u8>> {
    let prefix = delta.prefix_len as usize;
    let removed = delta.removed_len as usize;
    let suffix = delta.suffix_len as usize;
    (prefix + removed + suffix == previous.len()).then(|| {
        let mut next = Vec::with_capacity(prefix + delta.replacement.len() + suffix);
        next.extend_from_slice(&previous[..prefix]);
        next.extend_from_slice(&delta.replacement);
        next.extend_from_slice(&previous[previous.len() - suffix..]);
        next
    })
}

#[derive(Debug)]
pub struct SemanticCache {
    root: PathBuf,
}

impl SemanticCache {
    pub fn open(root: impl Into<PathBuf>) -> io::Result<Self> {
        let root = root.into();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn load(&self, stable_id: &str) -> io::Result<Option<CachedFunction>> {
        let path = self.path(stable_id);
        match fs::read(path) {
            Ok(bytes) => decode(&bytes).map(Some),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn load_all(&self) -> io::Result<Vec<CachedFunction>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let path = entry?.path();
            if path.extension().and_then(|extension| extension.to_str()) == Some("bin") {
                entries.push(decode(&fs::read(path)?)?);
            }
        }
        entries.sort_by(|left, right| left.stable_id.cmp(&right.stable_id));
        Ok(entries)
    }

    /// Loads every valid cache entry while preserving information about invalid
    /// blobs. Callers that can recover by reanalyzing source should use this
    /// method instead of allowing a disposable cache artifact to fail a whole
    /// repository operation.
    pub fn load_all_recovering(&self) -> io::Result<CacheLoad> {
        let mut entries = Vec::new();
        let mut invalid_blobs = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let path = entry?.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("bin") {
                continue;
            }
            match fs::read(&path).and_then(|bytes| decode(&bytes)) {
                Ok(entry) => entries.push(entry),
                Err(error) => invalid_blobs.push(CacheLoadFailure {
                    path,
                    reason: error.to_string(),
                }),
            }
        }
        entries.sort_by(|left, right| left.stable_id.cmp(&right.stable_id));
        invalid_blobs.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(CacheLoad {
            entries,
            invalid_blobs,
        })
    }

    /// Returns a deterministic cache inventory without reading source or
    /// invoking a frontend.
    pub fn stats(&self) -> io::Result<SemanticCacheStats> {
        let loaded = self.load_all_recovering()?;
        let mut semantic_blobs = 0;
        let mut delta_blobs = 0;
        let mut semantic_blob_bytes = 0;
        let mut delta_bytes = 0;
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let size = entry.metadata()?.len();
            match entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
            {
                Some("bin") => {
                    semantic_blobs += 1;
                    semantic_blob_bytes += size;
                }
                Some("delta") => {
                    delta_blobs += 1;
                    delta_bytes += size;
                }
                _ => {}
            }
        }
        let compatible_entries = loaded
            .entries
            .iter()
            .filter(|entry| entry.is_schema_compatible())
            .count();
        Ok(SemanticCacheStats {
            cache_version: CACHE_VERSION,
            semantic_blobs,
            compatible_entries,
            incompatible_entries: loaded.entries.len() - compatible_entries,
            invalid_blobs: loaded.invalid_blobs,
            delta_blobs,
            semantic_blob_bytes,
            delta_bytes,
            cache_bytes: semantic_blob_bytes + delta_bytes,
            average_semantic_blob_bytes: if semantic_blobs == 0 {
                0
            } else {
                semantic_blob_bytes / semantic_blobs as u64
            },
        })
    }

    pub fn store(&self, entry: &CachedFunction) -> io::Result<CacheWrite> {
        let path = self.path(&entry.stable_id);
        let current = encode(entry)?;
        match fs::read(&path) {
            Ok(previous) if previous == current => Ok(CacheWrite::Unchanged),
            Ok(previous) => {
                let change = delta(&previous, &current);
                write_atomic(&self.delta_path(&entry.stable_id), &encode_delta(&change))?;
                write_atomic(&path, &current)?;
                Ok(CacheWrite::Updated {
                    delta_bytes: change.replacement.len(),
                })
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                write_atomic(&path, &current)?;
                Ok(CacheWrite::Created)
            }
            Err(error) => Err(error),
        }
    }

    fn path(&self, stable_id: &str) -> PathBuf {
        self.root
            .join(format!("{:016x}.bin", stable_hash(stable_id.as_bytes())))
    }
    fn delta_path(&self, stable_id: &str) -> PathBuf {
        self.root
            .join(format!("{:016x}.delta", stable_hash(stable_id.as_bytes())))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheWrite {
    Created,
    Updated { delta_bytes: usize },
    Unchanged,
}

pub fn stable_hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}
pub fn semantic_hash(facts: &FunctionFacts) -> u64 {
    stable_hash(&serde_json::to_vec(facts).expect("function facts are serializable"))
}

fn query_metadata(report: &FunctionIntentReport) -> Vec<String> {
    let mut values = report.primary_intent.iter().cloned().collect::<Vec<_>>();
    values.extend(
        report
            .semantic_pipeline
            .iter()
            .map(|stage| stage.hypothesis_id.clone()),
    );
    values.extend(
        report
            .hypotheses
            .iter()
            .filter(|hypothesis| {
                matches!(hypothesis.status, crate::report::HypothesisStatus::Active)
            })
            .map(|hypothesis| hypothesis.id.clone()),
    );
    values.sort();
    values.dedup();
    values
}

fn encode(entry: &CachedFunction) -> io::Result<Vec<u8>> {
    let payload = serde_json::to_vec(entry).map_err(io::Error::other)?;
    let mut bytes = Vec::with_capacity(10 + payload.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&CACHE_VERSION.to_le_bytes());
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}
fn decode(bytes: &[u8]) -> io::Result<CachedFunction> {
    if bytes.len() < 10
        || &bytes[..4] != MAGIC
        || u16::from_le_bytes([bytes[4], bytes[5]]) != CACHE_VERSION
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported semantic cache blob",
        ));
    }
    let length = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]) as usize;
    if bytes.len() != 10 + length {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "truncated semantic cache blob",
        ));
    }
    serde_json::from_slice(&bytes[10..]).map_err(io::Error::other)
}
fn encode_delta(delta: &BinaryDelta) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(12 + delta.replacement.len());
    bytes.extend_from_slice(&delta.prefix_len.to_le_bytes());
    bytes.extend_from_slice(&delta.removed_len.to_le_bytes());
    bytes.extend_from_slice(&delta.suffix_len.to_le_bytes());
    bytes.extend_from_slice(&delta.replacement);
    bytes
}

/// Replaces a cache artifact only after a complete sibling temporary file is
/// flushed. This keeps interrupted cache writes from leaving truncated blobs
/// that would poison later query or profile runs.
fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "semantic cache path has no parent directory",
        )
    })?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("entry");
    let mut temporary_file = None;
    for _ in 0..16 {
        let sequence = CACHE_WRITE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(".{name}.{}.{sequence}.tmp", std::process::id()));
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&candidate)
        {
            Ok(opened) => {
                temporary_file = Some((candidate, opened));
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    let (temporary, mut file) = temporary_file.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::AlreadyExists,
            "cannot allocate a temporary semantic cache path",
        )
    })?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        replace_file(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(not(windows))]
fn replace_file(temporary: &Path, path: &Path) -> io::Result<()> {
    fs::rename(temporary, path)
}

#[cfg(windows)]
fn replace_file(temporary: &Path, path: &Path) -> io::Result<()> {
    match fs::rename(temporary, path) {
        Ok(()) => Ok(()),
        // Windows does not allow `rename` to replace an existing destination.
        // Removing the complete old cache blob can only create a cache miss;
        // it can never expose a partially written replacement.
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            fs::remove_file(path)?;
            fs::rename(temporary, path)
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use std::{env, sync::atomic::AtomicUsize};

    use super::*;

    static TEMPORARY_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn binary_delta_round_trips_losslessly() {
        let before = b"semantic-state-v1";
        let after = b"semantic-state-v2-expanded";
        assert_eq!(
            apply_delta(before, &delta(before, after)),
            Some(after.to_vec())
        );
    }

    #[test]
    fn atomic_cache_write_replaces_only_complete_contents() {
        let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = env::temp_dir().join(format!(
            "vz-semantic-cache-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("entry.bin");

        write_atomic(&path, b"first").unwrap();
        write_atomic(&path, b"second").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"second");
        assert_eq!(
            fs::read_dir(&directory)
                .unwrap()
                .filter_map(Result::ok)
                .count(),
            1
        );
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn recovering_load_keeps_valid_entries_when_a_blob_is_corrupt() {
        let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = env::temp_dir().join(format!(
            "vz-semantic-cache-recovery-{}-{sequence}",
            std::process::id()
        ));
        let cache = SemanticCache::open(&directory).unwrap();
        let entry = CachedFunction::new(
            "fixture::0::calculate".to_owned(),
            1,
            FunctionFacts {
                function_name: "calculate".to_owned(),
                parameter_count: 0,
                variable_count: 0,
                assignment_count: 0,
                mutation_count: 0,
                addition_mutation_count: 0,
                multiplication_mutation_count: 0,
                increment_mutation_count: 0,
                loop_count: 0,
                nested_loop_count: 0,
                collection_iteration_count: 0,
                conditional_count: 0,
                conditional_branch_count: 0,
                return_count: 0,
                boolean_return_count: 0,
                loop_return_count: 0,
                call_count: 0,
                recursion_count: 0,
                comparison_count: 0,
                maximum_update_count: 0,
                minimum_update_count: 0,
                division_count: 0,
                return_division_count: 0,
                allocation_count: 0,
                ownership_transfer_count: 0,
                reference_count: 0,
                api_usage_count: 0,
                has_accumulation: false,
                type_relationships: Vec::new(),
            },
            FunctionIntentReport {
                function_name: "calculate".to_owned(),
                evidence: Vec::new(),
                hypotheses: Vec::new(),
                primary_intent: None,
                semantic_pipeline: Vec::new(),
            },
        );
        cache.store(&entry).unwrap();
        fs::write(directory.join("corrupt.bin"), b"not a semantic cache blob").unwrap();

        let recovered = cache.load_all_recovering().unwrap();
        assert_eq!(recovered.entries, vec![entry]);
        assert_eq!(recovered.invalid_blobs.len(), 1);
        assert_eq!(
            recovered.invalid_blobs[0].reason,
            "unsupported semantic cache blob"
        );

        let stats = cache.stats().unwrap();
        assert_eq!(stats.semantic_blobs, 2);
        assert_eq!(stats.compatible_entries, 1);
        assert_eq!(stats.invalid_blobs.len(), 1);
        let _ = fs::remove_dir_all(directory);
    }
}
