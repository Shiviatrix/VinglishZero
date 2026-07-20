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

/// Renders a stable human-readable view of a deterministic intent report.
///
/// This presentation is derived exclusively from the serialized report model:
/// it does not recover source text or inspect adapter-specific state. The
/// concise view keeps a primary intent traceable while retaining the strongest
/// alternatives and eliminations that shaped the decision.
pub fn render_explanation(report: &IntentReport) -> String {
    if report.function_reports.is_empty() {
        return "Vinglish Zero semantic explanation\n\nNo functions were available for reasoning.\n"
            .to_owned();
    }

    let mut output = String::from("Vinglish Zero semantic explanation\n");
    for function in &report.function_reports {
        output.push('\n');
        render_function(&mut output, function);
    }
    output
}

fn render_function(output: &mut String, function: &FunctionIntentReport) {
    output.push_str(&format!("Function: {}\n", function.function_name));
    if let Some(primary_id) = function.primary_intent.as_deref() {
        let confidence = function
            .hypotheses
            .iter()
            .find(|hypothesis| hypothesis.id == primary_id)
            .map(|hypothesis| hypothesis.confidence)
            .unwrap_or_default();
        output.push_str(&format!(
            "Primary intent: {} ({confidence}%)\n",
            display_identifier(primary_id)
        ));
    } else {
        output.push_str("Primary intent: no active hypothesis\n");
    }

    if function.semantic_pipeline.is_empty() {
        output.push_str("Semantic pipeline: none\n");
    } else {
        output.push_str("Semantic pipeline: ");
        output.push_str(
            &function
                .semantic_pipeline
                .iter()
                .map(|stage| {
                    format!(
                        "{} ({}%)",
                        display_identifier(&stage.hypothesis_id),
                        stage.confidence
                    )
                })
                .collect::<Vec<_>>()
                .join(" -> "),
        );
        output.push('\n');
    }

    output.push_str("Observed evidence:\n");
    if function.evidence.is_empty() {
        output.push_str("  - none\n");
    } else {
        for evidence in &function.evidence {
            output.push_str(&format!("  - {}\n", display_evidence(evidence)));
        }
    }

    let alternatives = function
        .hypotheses
        .iter()
        .filter(|hypothesis| {
            hypothesis.status == HypothesisStatus::Active
                && function.primary_intent.as_deref() != Some(hypothesis.id.as_str())
        })
        .take(3)
        .collect::<Vec<_>>();
    if !alternatives.is_empty() {
        output.push_str("Other active hypotheses:\n");
        for hypothesis in alternatives {
            output.push_str(&format!(
                "  - {} ({}%)\n",
                display_identifier(&hypothesis.id),
                hypothesis.confidence
            ));
        }
    }

    let mut eliminations = function
        .hypotheses
        .iter()
        .filter(|hypothesis| {
            hypothesis.status == HypothesisStatus::Eliminated
                && !hypothesis.rejected_evidence.is_empty()
        })
        .collect::<Vec<_>>();
    eliminations.sort_by(|left, right| {
        right
            .supporting_evidence
            .len()
            .cmp(&left.supporting_evidence.len())
            .then_with(|| left.id.cmp(&right.id))
    });
    if !eliminations.is_empty() {
        output.push_str("Constraint eliminations:\n");
        for hypothesis in eliminations.iter().take(3) {
            output.push_str(&format!(
                "  - {}: {}\n",
                display_identifier(&hypothesis.id),
                hypothesis
                    .rejected_evidence
                    .iter()
                    .map(display_rejection)
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
        if eliminations.len() > 3 {
            output.push_str(&format!(
                "  - {} additional hypotheses eliminated\n",
                eliminations.len() - 3
            ));
        }
    }
}

fn display_identifier(identifier: &str) -> String {
    identifier
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            let Some(first) = characters.next() else {
                return String::new();
            };
            format!("{}{}", first.to_ascii_uppercase(), characters.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_evidence(evidence: &Evidence) -> String {
    format!(
        "{} [{}; strength {}]",
        display_evidence_kind(&evidence.kind),
        display_identifier(&evidence.provider_id),
        evidence.strength
    )
}

fn display_evidence_kind(kind: &EvidenceKind) -> String {
    match kind {
        EvidenceKind::NameToken { token } => format!("name token '{token}'"),
        EvidenceKind::Loop => "loop".to_owned(),
        EvidenceKind::NestedLoop => "nested loop".to_owned(),
        EvidenceKind::CollectionIteration => "collection iteration".to_owned(),
        EvidenceKind::Conditional => "conditional".to_owned(),
        EvidenceKind::Return => "return".to_owned(),
        EvidenceKind::BooleanReturn => "boolean return".to_owned(),
        EvidenceKind::Call => "call".to_owned(),
        EvidenceKind::Recursion => "recursion".to_owned(),
        EvidenceKind::Assignment => "assignment".to_owned(),
        EvidenceKind::Mutation => "mutation".to_owned(),
        EvidenceKind::AdditionMutation => "addition mutation".to_owned(),
        EvidenceKind::MultiplicationMutation => "multiplication mutation".to_owned(),
        EvidenceKind::IncrementMutation => "increment mutation".to_owned(),
        EvidenceKind::Division => "division".to_owned(),
        EvidenceKind::Comparison => "comparison".to_owned(),
        EvidenceKind::MaximumUpdate => "maximum update".to_owned(),
        EvidenceKind::MinimumUpdate => "minimum update".to_owned(),
        EvidenceKind::LoopReturn => "loop-local return".to_owned(),
        EvidenceKind::NumericParameter => "numeric parameter".to_owned(),
        EvidenceKind::NumericReturn => "numeric return".to_owned(),
        EvidenceKind::Allocation => "allocation".to_owned(),
        EvidenceKind::OwnershipTransfer => "ownership transfer".to_owned(),
        EvidenceKind::Reference => "reference".to_owned(),
        EvidenceKind::ApiUsage => "API usage".to_owned(),
        EvidenceKind::Accumulation => "accumulation".to_owned(),
    }
}

fn display_rejection(rejection: &Rejection) -> String {
    match rejection {
        Rejection::MissingRequiredEvidence { rule_id, evidence } => format!(
            "missing {} ({})",
            display_evidence_kind(evidence),
            display_identifier(rule_id)
        ),
        Rejection::ContradictingEvidence { rule_id, evidence } => format!(
            "conflicting {} ({})",
            display_evidence(evidence),
            display_identifier(rule_id)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explanation_is_stable_and_traces_primary_evidence() {
        let report = IntentReport {
            version: REPORT_VERSION,
            function_reports: vec![FunctionIntentReport {
                function_name: "count_if".to_owned(),
                evidence: vec![Evidence {
                    provider_id: "control_flow".to_owned(),
                    kind: EvidenceKind::Conditional,
                    strength: 1,
                }],
                hypotheses: vec![
                    HypothesisReport {
                        id: "count_if".to_owned(),
                        confidence: 100,
                        status: HypothesisStatus::Active,
                        supporting_evidence: vec![Evidence {
                            provider_id: "control_flow".to_owned(),
                            kind: EvidenceKind::Conditional,
                            strength: 1,
                        }],
                        rejected_evidence: Vec::new(),
                    },
                    HypothesisReport {
                        id: "average".to_owned(),
                        confidence: 0,
                        status: HypothesisStatus::Eliminated,
                        supporting_evidence: Vec::new(),
                        rejected_evidence: vec![Rejection::MissingRequiredEvidence {
                            rule_id: "division".to_owned(),
                            evidence: EvidenceKind::Division,
                        }],
                    },
                ],
                primary_intent: Some("count_if".to_owned()),
                semantic_pipeline: Vec::new(),
            }],
        };

        let explanation = render_explanation(&report);
        assert!(explanation.contains("Primary intent: Count If (100%)"));
        assert!(explanation.contains("Observed evidence:"));
        assert!(explanation.contains("conditional [Control Flow; strength 1]"));
        assert!(explanation.contains("Average: missing division (Division)"));
        assert_eq!(explanation, render_explanation(&report));
    }
}
