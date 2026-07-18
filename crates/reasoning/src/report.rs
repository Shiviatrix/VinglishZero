//! Serializable deterministic reasoning output.

use serde::{Deserialize, Serialize};

use crate::evidence::{Evidence, EvidenceKind};

pub const REPORT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentReport {
    pub version: u32,
    pub function_reports: Vec<FunctionIntentReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionIntentReport {
    pub function_name: String,
    pub evidence: Vec<Evidence>,
    pub hypotheses: Vec<HypothesisReport>,
    /// The legacy dominant active hypothesis, retained alongside composition.
    #[serde(default)]
    pub primary_intent: Option<String>,
    /// The most specific deterministic composition recognized for this function.
    /// An empty pipeline preserves the original single-intent behavior.
    #[serde(default)]
    pub semantic_pipeline: Vec<SemanticStage>,
}

/// One ordered stage in a semantic pipeline. Stages reference existing
/// hypotheses rather than introducing a second semantic vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticStage {
    pub hypothesis_id: String,
    pub confidence: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HypothesisReport {
    pub id: String,
    pub confidence: u8,
    pub status: HypothesisStatus,
    pub supporting_evidence: Vec<Evidence>,
    pub rejected_evidence: Vec<Rejection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisStatus {
    Active,
    Eliminated,
}

/// A rule-based rejection, including missing required evidence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Rejection {
    MissingRequiredEvidence {
        rule_id: String,
        evidence: EvidenceKind,
    },
    ContradictingEvidence {
        rule_id: String,
        evidence: Evidence,
    },
}
