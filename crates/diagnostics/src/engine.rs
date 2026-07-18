//! Semantic diagnostic orchestration over compiler diagnostics and intent reports.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    matcher::{default_rules, DiagnosticRule, DiagnosticRuleRegistry},
    model::{
        ApplicableEvidence, CompilerDiagnostic, IntentReference, Interpretation, SemanticDiagnostic,
    },
    suggestions,
};

pub const SEMANTIC_DIAGNOSTIC_VERSION: u32 = 1;

/// Combines a portable compiler diagnostic with deterministic intent output.
pub struct SemanticDiagnosticEngine {
    rules: DiagnosticRuleRegistry,
}

impl Default for SemanticDiagnosticEngine {
    fn default() -> Self {
        Self {
            rules: default_rules(),
        }
    }
}

impl SemanticDiagnosticEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_rule(&mut self, rule: DiagnosticRule) {
        self.rules.register(rule);
    }

    /// Accepts the real reasoning `IntentReport` without importing the
    /// reasoning crate. The report is projected through its serializable,
    /// versioned structure, preserving the one-way dependency graph.
    pub fn diagnose<T: Serialize>(
        &self,
        compiler_diagnostic: CompilerDiagnostic,
        intent_report: &T,
    ) -> Result<SemanticDiagnostic, DiagnosticEngineError> {
        let value = serde_json::to_value(intent_report)
            .map_err(DiagnosticEngineError::SerializeIntentReport)?;
        let report: IntentReportProjection = serde_json::from_value(value)
            .map_err(DiagnosticEngineError::DeserializeIntentReport)?;
        Ok(self.diagnose_projection(compiler_diagnostic, report))
    }

    fn diagnose_projection(
        &self,
        compiler_diagnostic: CompilerDiagnostic,
        report: IntentReportProjection,
    ) -> SemanticDiagnostic {
        let mut candidates = Vec::new();
        for function in report.function_reports {
            if compiler_diagnostic
                .function_name
                .as_ref()
                .is_some_and(|name| name != &function.function_name)
            {
                continue;
            }
            for hypothesis in function.hypotheses {
                for rule in self
                    .rules
                    .matching(&compiler_diagnostic.category, &hypothesis.id)
                {
                    if hypothesis.status == HypothesisStatusProjection::Eliminated
                        && !rule_allows_eliminated(rule)
                    {
                        continue;
                    }
                    candidates.push(Candidate {
                        function_name: function.function_name.clone(),
                        hypothesis: hypothesis.clone(),
                        rule,
                    });
                }
            }
        }

        candidates.sort_by(|left, right| {
            status_rank(right.hypothesis.status)
                .cmp(&status_rank(left.hypothesis.status))
                .then_with(|| right.hypothesis.confidence.cmp(&left.hypothesis.confidence))
                .then_with(|| left.function_name.cmp(&right.function_name))
                .then_with(|| left.hypothesis.id.cmp(&right.hypothesis.id))
                .then_with(|| left.rule.id.cmp(right.rule.id))
        });

        let Some(candidate) = candidates.into_iter().next() else {
            return SemanticDiagnostic {
                version: SEMANTIC_DIAGNOSTIC_VERSION,
                compiler_issue: compiler_diagnostic,
                intent: None,
                interpretation: Interpretation::NoMatchingIntentRule,
                suggestions: Vec::new(),
            };
        };

        let mut evidence = candidate.hypothesis.supporting_evidence;
        evidence.sort();
        let intent = IntentReference {
            function_name: candidate.function_name,
            hypothesis_id: candidate.hypothesis.id,
            confidence: candidate.hypothesis.confidence,
            evidence: evidence.clone(),
        };
        let suggestions = suggestions::generate(
            candidate.rule.suggestion_set,
            intent.confidence,
            &intent.evidence,
        );

        SemanticDiagnostic {
            version: SEMANTIC_DIAGNOSTIC_VERSION,
            compiler_issue: compiler_diagnostic,
            intent: Some(intent),
            interpretation: candidate.rule.interpretation,
            suggestions,
        }
    }
}

fn rule_allows_eliminated(rule: &DiagnosticRule) -> bool {
    matches!(
        rule.interpretation,
        Interpretation::AverageImplementationIncomplete
    )
}

fn status_rank(status: HypothesisStatusProjection) -> u8 {
    match status {
        HypothesisStatusProjection::Active => 1,
        HypothesisStatusProjection::Eliminated => 0,
    }
}

struct Candidate<'a> {
    function_name: String,
    hypothesis: HypothesisProjection,
    rule: &'a DiagnosticRule,
}

#[derive(Debug)]
pub enum DiagnosticEngineError {
    SerializeIntentReport(serde_json::Error),
    DeserializeIntentReport(serde_json::Error),
}

impl fmt::Display for DiagnosticEngineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SerializeIntentReport(error) => {
                write!(formatter, "cannot serialize intent report: {error}")
            }
            Self::DeserializeIntentReport(error) => {
                write!(formatter, "unsupported intent report structure: {error}")
            }
        }
    }
}

impl std::error::Error for DiagnosticEngineError {}

// This projection is deliberately private. It is the serializable contract the
// diagnostics layer needs from reasoning, not a dependency on reasoning code.
#[derive(Debug, Deserialize)]
struct IntentReportProjection {
    function_reports: Vec<FunctionIntentReportProjection>,
}

#[derive(Debug, Deserialize)]
struct FunctionIntentReportProjection {
    function_name: String,
    hypotheses: Vec<HypothesisProjection>,
}

#[derive(Debug, Clone, Deserialize)]
struct HypothesisProjection {
    id: String,
    confidence: u8,
    status: HypothesisStatusProjection,
    supporting_evidence: Vec<ApplicableEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum HypothesisStatusProjection {
    Active,
    Eliminated,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DiagnosticCategory, Severity, SuggestionTitle};

    fn diagnostic(category: DiagnosticCategory) -> CompilerDiagnostic {
        CompilerDiagnostic {
            code: "test".to_owned(),
            severity: Severity::Error,
            category,
            message: "test".to_owned(),
            location: None,
            function_name: Some("calculate".to_owned()),
        }
    }

    #[test]
    fn matches_accumulator_type_mismatch_with_deterministic_fixes() {
        let report = serde_json::json!({
            "version": 1,
            "function_reports": [{
                "function_name": "calculate",
                "hypotheses": [{
                    "id": "accumulator",
                    "confidence": 94,
                    "status": "active",
                    "supporting_evidence": [{
                        "provider_id": "mutation",
                        "kind": { "kind": "addition_mutation" },
                        "strength": 1
                    }]
                }]
            }]
        });

        let result = SemanticDiagnosticEngine::new()
            .diagnose(diagnostic(DiagnosticCategory::TypeMismatch), &report)
            .unwrap();

        assert_eq!(
            result.interpretation,
            Interpretation::AccumulatorTypeConflict
        );
        assert_eq!(result.intent.unwrap().confidence, 94);
        assert_eq!(result.suggestions.len(), 3);
        assert_eq!(
            result.suggestions[0].title,
            SuggestionTitle::ChangeAccumulatorType
        );
    }

    #[test]
    fn matches_eliminated_average_for_missing_division() {
        let report = serde_json::json!({
            "version": 1,
            "function_reports": [{
                "function_name": "calculate",
                "hypotheses": [{
                    "id": "average",
                    "confidence": 0,
                    "status": "eliminated",
                    "supporting_evidence": []
                }]
            }]
        });

        let result = SemanticDiagnosticEngine::new()
            .diagnose(diagnostic(DiagnosticCategory::MissingDivision), &report)
            .unwrap();

        assert_eq!(
            result.interpretation,
            Interpretation::AverageImplementationIncomplete
        );
        assert_eq!(result.suggestions.len(), 1);
    }

    #[test]
    fn serializes_identically_for_identical_inputs() {
        let report = serde_json::json!({ "version": 1, "function_reports": [] });
        let engine = SemanticDiagnosticEngine::new();
        let first = serde_json::to_string(
            &engine
                .diagnose(diagnostic(DiagnosticCategory::TypeMismatch), &report)
                .unwrap(),
        )
        .unwrap();
        let second = serde_json::to_string(
            &engine
                .diagnose(diagnostic(DiagnosticCategory::TypeMismatch), &report)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(first, second);
    }
}
