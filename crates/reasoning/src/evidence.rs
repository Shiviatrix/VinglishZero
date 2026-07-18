//! Independent evidence providers.
//!
//! Providers report observations only. They never mention a hypothesis or make
//! an intent decision; that responsibility belongs to the rule evaluator.

use serde::{Deserialize, Serialize};

use crate::facts::FunctionFacts;

/// A language-neutral observation that can be consumed by any hypothesis rule.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Evidence {
    pub provider_id: String,
    pub kind: EvidenceKind,
    pub strength: u32,
}

/// The stable vocabulary emitted by evidence providers in the first reasoning version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EvidenceKind {
    NameToken { token: String },
    Loop,
    NestedLoop,
    CollectionIteration,
    Conditional,
    Return,
    BooleanReturn,
    Call,
    Recursion,
    Assignment,
    Mutation,
    AdditionMutation,
    MultiplicationMutation,
    IncrementMutation,
    Division,
    Comparison,
    MaximumUpdate,
    MinimumUpdate,
    LoopReturn,
    NumericParameter,
    NumericReturn,
    Allocation,
    OwnershipTransfer,
    Reference,
    ApiUsage,
    Accumulation,
}

/// A pluggable source of observations derived from immutable facts.
pub trait EvidenceProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn collect(&self, facts: &FunctionFacts) -> Vec<Evidence>;
}

#[derive(Debug, Default)]
pub struct NamingEvidence;

impl EvidenceProvider for NamingEvidence {
    fn id(&self) -> &'static str {
        "naming"
    }

    fn collect(&self, facts: &FunctionFacts) -> Vec<Evidence> {
        facts
            .function_name
            .split(|character: char| !character.is_ascii_alphanumeric())
            .filter(|token| !token.is_empty())
            .map(|token| Evidence {
                provider_id: self.id().to_owned(),
                kind: EvidenceKind::NameToken {
                    token: token.to_ascii_lowercase(),
                },
                strength: 1,
            })
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct ControlFlowEvidence;

impl EvidenceProvider for ControlFlowEvidence {
    fn id(&self) -> &'static str {
        "control_flow"
    }

    fn collect(&self, facts: &FunctionFacts) -> Vec<Evidence> {
        let mut evidence = Vec::new();
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::Loop,
            facts.loop_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::NestedLoop,
            facts.nested_loop_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::CollectionIteration,
            facts.collection_iteration_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::Conditional,
            facts.conditional_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::Return,
            facts.return_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::BooleanReturn,
            facts.boolean_return_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::Call,
            facts.call_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::Recursion,
            facts.recursion_count,
        );
        evidence
    }
}

#[derive(Debug, Default)]
pub struct MutationEvidence;

impl EvidenceProvider for MutationEvidence {
    fn id(&self) -> &'static str {
        "mutation"
    }

    fn collect(&self, facts: &FunctionFacts) -> Vec<Evidence> {
        let mut evidence = Vec::new();
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::Assignment,
            facts.assignment_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::Mutation,
            facts.mutation_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::AdditionMutation,
            facts.addition_mutation_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::IncrementMutation,
            facts.increment_mutation_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::MultiplicationMutation,
            facts.multiplication_mutation_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::Division,
            facts.division_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::Accumulation,
            u32::from(facts.has_accumulation),
        );
        evidence
    }
}

#[derive(Debug, Default)]
pub struct TypeEvidence;

impl EvidenceProvider for TypeEvidence {
    fn id(&self) -> &'static str {
        "type"
    }

    fn collect(&self, facts: &FunctionFacts) -> Vec<Evidence> {
        use crate::facts::{TypeCategory, TypeRole};

        let numeric_parameters = facts
            .type_relationships
            .iter()
            .filter(|relationship| {
                relationship.role == TypeRole::Parameter
                    && relationship.category == TypeCategory::Numeric
            })
            .count() as u32;
        let numeric_returns = facts
            .type_relationships
            .iter()
            .filter(|relationship| {
                relationship.role == TypeRole::Return
                    && relationship.category == TypeCategory::Numeric
            })
            .count() as u32;
        let mut evidence = Vec::new();
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::NumericParameter,
            numeric_parameters,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::NumericReturn,
            numeric_returns,
        );
        evidence
    }
}

#[derive(Debug, Default)]
pub struct OwnershipEvidence;

impl EvidenceProvider for OwnershipEvidence {
    fn id(&self) -> &'static str {
        "ownership"
    }

    fn collect(&self, facts: &FunctionFacts) -> Vec<Evidence> {
        let mut evidence = Vec::new();
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::Allocation,
            facts.allocation_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::OwnershipTransfer,
            facts.ownership_transfer_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::Reference,
            facts.reference_count,
        );
        evidence
    }
}

#[derive(Debug, Default)]
pub struct DataFlowEvidence;

impl EvidenceProvider for DataFlowEvidence {
    fn id(&self) -> &'static str {
        "data_flow"
    }

    fn collect(&self, facts: &FunctionFacts) -> Vec<Evidence> {
        let mut evidence = Vec::new();
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::Comparison,
            facts.comparison_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::MaximumUpdate,
            facts.maximum_update_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::MinimumUpdate,
            facts.minimum_update_count,
        );
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::LoopReturn,
            facts.loop_return_count,
        );
        evidence
    }
}

#[derive(Debug, Default)]
pub struct ApiUsageEvidence;

impl EvidenceProvider for ApiUsageEvidence {
    fn id(&self) -> &'static str {
        "api_usage"
    }

    fn collect(&self, facts: &FunctionFacts) -> Vec<Evidence> {
        let mut evidence = Vec::new();
        push(
            &mut evidence,
            self.id(),
            EvidenceKind::ApiUsage,
            facts.api_usage_count,
        );
        evidence
    }
}

pub fn default_providers() -> Vec<Box<dyn EvidenceProvider>> {
    vec![
        Box::new(NamingEvidence),
        Box::new(ControlFlowEvidence),
        Box::new(MutationEvidence),
        Box::new(TypeEvidence),
        Box::new(OwnershipEvidence),
        Box::new(DataFlowEvidence),
        Box::new(ApiUsageEvidence),
    ]
}

fn push(evidence: &mut Vec<Evidence>, provider_id: &str, kind: EvidenceKind, strength: u32) {
    if strength > 0 {
        evidence.push(Evidence {
            provider_id: provider_id.to_owned(),
            kind,
            strength,
        });
    }
}
