//! Persistent, deterministic per-function semantic cache.

use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    facts::FunctionFacts,
    report::{FunctionIntentReport, SemanticStage},
};

pub const CACHE_VERSION: u16 = 1;
pub const SEMANTIC_IR_VERSION: u16 = 1;
pub const REASONING_ENGINE_VERSION: u16 = 1;
pub const HYPOTHESIS_REGISTRY_VERSION: u16 = 1;
const MAGIC: &[u8; 4] = b"VZSC";

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

    pub fn store(&self, entry: &CachedFunction) -> io::Result<CacheWrite> {
        let path = self.path(&entry.stable_id);
        let current = encode(entry)?;
        match fs::read(&path) {
            Ok(previous) if previous == current => Ok(CacheWrite::Unchanged),
            Ok(previous) => {
                let change = delta(&previous, &current);
                fs::write(self.delta_path(&entry.stable_id), encode_delta(&change))?;
                fs::write(path, current)?;
                Ok(CacheWrite::Updated {
                    delta_bytes: change.replacement.len(),
                })
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::write(path, current)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binary_delta_round_trips_losslessly() {
        let before = b"semantic-state-v1";
        let after = b"semantic-state-v2-expanded";
        assert_eq!(
            apply_delta(before, &delta(before, after)),
            Some(after.to_vec())
        );
    }
}
