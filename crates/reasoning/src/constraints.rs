//! Deterministic constraint evaluation and integer confidence scoring.

use std::collections::BTreeSet;

use crate::{
    evidence::Evidence,
    hypotheses::{DeterministicRule, HypothesisDefinition},
    report::{HypothesisReport, HypothesisStatus, Rejection},
};

/// Evaluates declarative rules with no probabilistic or external inputs.
#[derive(Debug, Default, Clone, Copy)]
pub struct ConstraintEngine;

impl ConstraintEngine {
    pub fn evaluate(
        &self,
        definition: &dyn HypothesisDefinition,
        evidence: &[Evidence],
    ) -> HypothesisReport {
        let mut score = definition.baseline_confidence();
        let mut eliminated = false;
        let mut supporting = BTreeSet::new();
        let mut rejected = BTreeSet::new();

        for rule in definition.rules() {
            match rule {
                DeterministicRule::Require {
                    id,
                    evidence: required,
                } => {
                    if !evidence.iter().any(|item| &item.kind == required) {
                        eliminated = true;
                        rejected.insert(Rejection::MissingRequiredEvidence {
                            rule_id: id.clone(),
                            evidence: required.clone(),
                        });
                    }
                }
                DeterministicRule::Support {
                    evidence: expected,
                    weight,
                    ..
                } => {
                    for item in evidence.iter().filter(|item| &item.kind == expected) {
                        score += weighted_score(*weight, item.strength);
                        supporting.insert(item.clone());
                    }
                }
                DeterministicRule::Reject {
                    id,
                    evidence: contradicting,
                    weight,
                } => {
                    for item in evidence.iter().filter(|item| &item.kind == contradicting) {
                        score -= weighted_score(*weight, item.strength);
                        rejected.insert(Rejection::ContradictingEvidence {
                            rule_id: id.clone(),
                            evidence: item.clone(),
                        });
                    }
                }
            }
        }

        HypothesisReport {
            id: definition.id().to_owned(),
            confidence: if eliminated {
                0
            } else {
                clamp_confidence(score)
            },
            status: if eliminated {
                HypothesisStatus::Eliminated
            } else {
                HypothesisStatus::Active
            },
            supporting_evidence: supporting.into_iter().collect(),
            rejected_evidence: rejected.into_iter().collect(),
        }
    }
}

/// Integer-only score calculation. Strength is capped to avoid a node count
/// overwhelming a rule's intentional, fixed weight.
fn weighted_score(weight: i32, strength: u32) -> i32 {
    weight.saturating_mul(strength.clamp(1, 3) as i32)
}

fn clamp_confidence(score: i32) -> u8 {
    score.clamp(0, 100) as u8
}
