//! Deterministic symbolic reasoning over Semantic IR.
//!
//! This crate never reads source text and never depends on an adapter or a
//! language-specific type. It turns Semantic IR into immutable facts, collects
//! observations from independent providers, evaluates registered hypothesis
//! rules, and returns a serializable intent report.

pub mod composition;
pub mod constraints;
pub mod engine;
pub mod evidence;
pub mod facts;
pub mod hypotheses;
pub mod incremental;
pub mod query;
pub mod report;

pub use engine::ReasoningEngine;
pub use evidence::{Evidence, EvidenceKind, EvidenceProvider};
pub use facts::{FactExtractor, FactSet, FunctionFacts};
pub use hypotheses::{HypothesisDefinition, HypothesisRegistry, RuleBasedHypothesis};
pub use incremental::{CachedFunction, SemanticCache};
pub use query::{parse as parse_query, search as search_query, QueryExpression, QueryMatch};
pub use report::{FunctionIntentReport, HypothesisReport, IntentReport, SemanticStage};

/// A compatibility boundary for callers that schedule deterministic reasoning.
#[derive(Debug, Clone, Default)]
pub struct ReasoningRequest;

/// A named, deterministic reasoning strategy.
pub trait ReasoningStrategy {
    fn name(&self) -> &str;
}

/// A deterministic plan representation retained for future orchestration.
#[derive(Debug, Clone, Default)]
pub struct ReasoningPlan {
    pub steps: Vec<String>,
}
