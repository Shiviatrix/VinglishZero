//! The fixed deterministic reasoning pipeline.

use crate::{
    composition,
    constraints::ConstraintEngine,
    evidence::{default_providers, Evidence, EvidenceProvider},
    facts::{FactExtractor, FunctionFacts},
    hypotheses::{default_registry, HypothesisDefinition, HypothesisRegistry},
    report::{FunctionIntentReport, IntentReport, REPORT_VERSION},
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};

use vz_semantic_ir::SemanticGraph;

/// Orchestrates fact extraction, evidence collection, rule evaluation, and
/// report construction. It owns no language-specific state.
pub struct ReasoningEngine {
    fact_extractor: FactExtractor,
    providers: Vec<Box<dyn EvidenceProvider>>,
    registry: HypothesisRegistry,
    constraints: ConstraintEngine,
}

impl Default for ReasoningEngine {
    fn default() -> Self {
        Self {
            fact_extractor: FactExtractor,
            providers: default_providers(),
            registry: default_registry(),
            constraints: ConstraintEngine,
        }
    }
}

impl ReasoningEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_provider(&mut self, provider: impl EvidenceProvider + 'static) {
        self.providers.push(Box::new(provider));
    }

    pub fn register_hypothesis(&mut self, hypothesis: impl HypothesisDefinition + 'static) {
        self.registry.register(hypothesis);
    }

    pub fn analyze(&self, graph: &SemanticGraph) -> IntentReport {
        let facts = self.facts_for(graph);
        let function_reports = self.analyze_functions_parallel(&facts.functions);
        IntentReport {
            version: REPORT_VERSION,
            function_reports,
        }
    }

    pub fn facts_for(&self, graph: &SemanticGraph) -> crate::facts::FactSet {
        self.fact_extractor.extract(graph)
    }

    /// Analyzes one completed function fact set. Frontends can call this as
    /// soon as a function is lowered, without waiting for a whole program.
    pub fn analyze_function(&self, facts: &FunctionFacts) -> FunctionIntentReport {
        let mut evidence: Vec<Evidence> = self
            .providers
            .iter()
            .flat_map(|provider| provider.collect(facts))
            .collect();
        evidence.sort();
        let mut hypotheses: Vec<_> = self
            .registry
            .iter()
            .map(|definition| self.constraints.evaluate(definition, &evidence))
            .collect();
        hypotheses.sort_by(|left, right| {
            left.status
                .cmp(&right.status)
                .then_with(|| right.confidence.cmp(&left.confidence))
                .then_with(|| left.id.cmp(&right.id))
        });
        let primary_intent = hypotheses
            .iter()
            .find(|hypothesis| hypothesis.status == crate::report::HypothesisStatus::Active)
            .map(|hypothesis| hypothesis.id.clone());
        let semantic_pipeline = composition::infer(&hypotheses, &evidence);
        FunctionIntentReport {
            function_name: facts.function_name.clone(),
            evidence,
            hypotheses,
            primary_intent,
            semantic_pipeline,
        }
    }

    /// Bounded worker-pool execution for independent function facts. Results
    /// retain input order, so parallelism cannot affect report determinism.
    pub fn analyze_functions_parallel(&self, facts: &[FunctionFacts]) -> Vec<FunctionIntentReport> {
        let workers = std::thread::available_parallelism()
            .map_or(1, usize::from)
            .min(facts.len());
        if workers <= 1 {
            return facts
                .iter()
                .map(|facts| self.analyze_function(facts))
                .collect();
        }
        let next = AtomicUsize::new(0);
        let results = Mutex::new(Vec::with_capacity(facts.len()));
        std::thread::scope(|scope| {
            for _ in 0..workers {
                scope.spawn(|| {
                    let mut local = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(facts) = facts.get(index) else { break };
                        local.push((index, self.analyze_function(facts)));
                    }
                    results.lock().expect("reasoning worker lock").extend(local);
                });
            }
        });
        let mut results = results.into_inner().expect("reasoning worker lock");
        results.sort_by_key(|(index, _)| *index);
        results.into_iter().map(|(_, report)| report).collect()
    }
}
