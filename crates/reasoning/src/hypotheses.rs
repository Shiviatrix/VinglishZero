//! Registry-based semantic intent hypotheses.

use std::collections::BTreeMap;

use crate::evidence::EvidenceKind;

/// A hypothesis definition can be registered without changing the engine.
pub trait HypothesisDefinition: Send + Sync {
    fn id(&self) -> &str;
    fn baseline_confidence(&self) -> i32;
    fn rules(&self) -> &[DeterministicRule];
}

/// A simple data-driven hypothesis definition suitable for extension crates.
#[derive(Debug, Clone)]
pub struct RuleBasedHypothesis {
    id: String,
    baseline_confidence: i32,
    rules: Vec<DeterministicRule>,
}

impl RuleBasedHypothesis {
    pub fn new(
        id: impl Into<String>,
        baseline_confidence: i32,
        rules: Vec<DeterministicRule>,
    ) -> Self {
        Self {
            id: id.into(),
            baseline_confidence,
            rules,
        }
    }
}

impl HypothesisDefinition for RuleBasedHypothesis {
    fn id(&self) -> &str {
        &self.id
    }

    fn baseline_confidence(&self) -> i32 {
        self.baseline_confidence
    }

    fn rules(&self) -> &[DeterministicRule] {
        &self.rules
    }
}

/// A declarative rule evaluated only against evidence presence and strength.
#[derive(Debug, Clone)]
pub enum DeterministicRule {
    Require {
        id: String,
        evidence: EvidenceKind,
    },
    Support {
        id: String,
        evidence: EvidenceKind,
        weight: i32,
    },
    Reject {
        id: String,
        evidence: EvidenceKind,
        weight: i32,
    },
}

/// Registry ownership is deterministic: keys are sorted lexicographically.
#[derive(Default)]
pub struct HypothesisRegistry {
    definitions: BTreeMap<String, Box<dyn HypothesisDefinition>>,
}

impl HypothesisRegistry {
    pub fn register(&mut self, definition: impl HypothesisDefinition + 'static) {
        self.definitions
            .insert(definition.id().to_owned(), Box::new(definition));
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn HypothesisDefinition> {
        self.definitions.values().map(Box::as_ref)
    }
}

impl Default for RuleBasedHypothesis {
    fn default() -> Self {
        Self::new("unnamed", 0, Vec::new())
    }
}

pub fn default_registry() -> HypothesisRegistry {
    let mut registry = HypothesisRegistry::default();
    registry.register(hypothesis(
        "accumulator",
        vec![
            require("loop", EvidenceKind::Loop),
            require("addition", EvidenceKind::AdditionMutation),
            support("accumulation", EvidenceKind::Accumulation, 45),
            support("numeric_return", EvidenceKind::NumericReturn, 20),
            support("return", EvidenceKind::Return, 10),
        ],
    ));
    registry.register(hypothesis(
        "average",
        vec![
            require("loop", EvidenceKind::Loop),
            require("addition", EvidenceKind::AdditionMutation),
            require("division", EvidenceKind::Division),
            require("numeric_return", EvidenceKind::NumericReturn),
            support("accumulation", EvidenceKind::Accumulation, 30),
            support("numeric_parameter", EvidenceKind::NumericParameter, 20),
            support("division", EvidenceKind::Division, 45),
            support("name", name_token("average"), 45),
        ],
    ));
    registry.register(hypothesis(
        "builder",
        vec![
            require("allocation", EvidenceKind::Allocation),
            support("mutation", EvidenceKind::Mutation, 20),
            support("return", EvidenceKind::Return, 20),
        ],
    ));
    registry.register(hypothesis(
        "cache",
        vec![
            require("conditional", EvidenceKind::Conditional),
            require("mutation", EvidenceKind::Mutation),
            support("call", EvidenceKind::Call, 20),
        ],
    ));
    registry.register(hypothesis(
        "counter",
        vec![
            require("loop", EvidenceKind::Loop),
            require("increment", EvidenceKind::IncrementMutation),
            support("mutation", EvidenceKind::Mutation, 15),
            support("return", EvidenceKind::Return, 20),
            support("name", name_token("count"), 50),
        ],
    ));
    registry.register(hypothesis(
        "event_dispatcher",
        vec![
            require("conditional", EvidenceKind::Conditional),
            require("call", EvidenceKind::Call),
            support("api", EvidenceKind::ApiUsage, 25),
        ],
    ));
    registry.register(hypothesis(
        "factory",
        vec![
            require("allocation", EvidenceKind::Allocation),
            require("return", EvidenceKind::Return),
            support("name", name_token("create"), 20),
        ],
    ));
    registry.register(hypothesis(
        "filter",
        vec![
            require("loop", EvidenceKind::Loop),
            require("conditional", EvidenceKind::Conditional),
            support("collection", EvidenceKind::CollectionIteration, 15),
            support("name", name_token("filter"), 85),
        ],
    ));
    registry.register(hypothesis(
        "iterator",
        vec![
            require("collection", EvidenceKind::CollectionIteration),
            support("loop", EvidenceKind::Loop, 25),
            support("return", EvidenceKind::Return, 15),
        ],
    ));
    registry.register(hypothesis(
        "lookup",
        vec![
            require("conditional", EvidenceKind::Conditional),
            require("return", EvidenceKind::Return),
            support("comparison", EvidenceKind::Comparison, 25),
        ],
    ));
    registry.register(hypothesis(
        "mapper",
        vec![
            require("loop", EvidenceKind::Loop),
            support("return", EvidenceKind::Return, 20),
            support("collection", EvidenceKind::CollectionIteration, 15),
            support("call", EvidenceKind::Call, 35),
            support("name", name_token("map"), 65),
            support("name", name_token("mapper"), 65),
        ],
    ));
    registry.register(hypothesis(
        "maximum",
        vec![
            require("loop", EvidenceKind::Loop),
            require("selection_update", EvidenceKind::MaximumUpdate),
            require("numeric_return", EvidenceKind::NumericReturn),
            support("selection_update", EvidenceKind::MaximumUpdate, 70),
            support("conditional", EvidenceKind::Conditional, 20),
            support("return", EvidenceKind::Return, 15),
            support("collection", EvidenceKind::CollectionIteration, 10),
        ],
    ));
    registry.register(hypothesis(
        "minimum",
        vec![
            require("loop", EvidenceKind::Loop),
            require("selection_update", EvidenceKind::MinimumUpdate),
            require("numeric_return", EvidenceKind::NumericReturn),
            support("selection_update", EvidenceKind::MinimumUpdate, 70),
            support("conditional", EvidenceKind::Conditional, 20),
            support("return", EvidenceKind::Return, 15),
            support("collection", EvidenceKind::CollectionIteration, 10),
        ],
    ));
    registry.register(hypothesis(
        "sum",
        vec![
            require("loop", EvidenceKind::Loop),
            require("addition", EvidenceKind::AdditionMutation),
            require("numeric_return", EvidenceKind::NumericReturn),
            support("sum_name", name_token("sum"), 75),
            support("accumulation", EvidenceKind::Accumulation, 30),
        ],
    ));
    registry.register(hypothesis(
        "count_if",
        vec![
            require("loop", EvidenceKind::Loop),
            require("conditional", EvidenceKind::Conditional),
            require("increment", EvidenceKind::IncrementMutation),
            support("count_name", name_token("count"), 55),
        ],
    ));
    registry.register(hypothesis(
        "any",
        vec![
            require("loop", EvidenceKind::Loop),
            require("conditional", EvidenceKind::Conditional),
            require("return", EvidenceKind::Return),
            support("boolean_return", EvidenceKind::BooleanReturn, 25),
            support("early_return", EvidenceKind::LoopReturn, 35),
            support("any_name", name_token("any"), 70),
        ],
    ));
    registry.register(hypothesis(
        "all",
        vec![
            require("loop", EvidenceKind::Loop),
            require("conditional", EvidenceKind::Conditional),
            require("return", EvidenceKind::Return),
            support("boolean_return", EvidenceKind::BooleanReturn, 25),
            support("early_return", EvidenceKind::LoopReturn, 35),
            support("all_name", name_token("all"), 70),
        ],
    ));
    registry.register(hypothesis(
        "find_first",
        vec![
            require("loop", EvidenceKind::Loop),
            require("conditional", EvidenceKind::Conditional),
            require("early_return", EvidenceKind::LoopReturn),
            support("find_name", name_token("find"), 30),
            support("first_name", name_token("first"), 60),
        ],
    ));
    registry.register(hypothesis(
        "find_last",
        vec![
            require("loop", EvidenceKind::Loop),
            require("conditional", EvidenceKind::Conditional),
            require("return", EvidenceKind::Return),
            support("find_name", name_token("find"), 30),
            support("last_name", name_token("last"), 60),
        ],
    ));
    registry.register(hypothesis(
        "reverse_traversal",
        vec![
            require("loop", EvidenceKind::Loop),
            require("return", EvidenceKind::Return),
            support("reverse_name", name_token("reverse"), 85),
        ],
    ));
    registry.register(hypothesis(
        "prefix_sum",
        vec![
            require("loop", EvidenceKind::Loop),
            require("addition", EvidenceKind::AdditionMutation),
            support("prefix_name", name_token("prefix"), 60),
            support("sum_name", name_token("sum"), 35),
        ],
    ));
    registry.register(hypothesis(
        "frequency_counter",
        vec![
            require("loop", EvidenceKind::Loop),
            require("mutation", EvidenceKind::Mutation),
            support("frequency_name", name_token("frequency"), 85),
        ],
    ));
    registry.register(hypothesis(
        "histogram",
        vec![
            require("loop", EvidenceKind::Loop),
            require("mutation", EvidenceKind::Mutation),
            support("histogram_name", name_token("histogram"), 85),
        ],
    ));
    registry.register(hypothesis(
        "max_by",
        vec![
            require("loop", EvidenceKind::Loop),
            require("selection_update", EvidenceKind::MaximumUpdate),
            support("max_name", name_token("max"), 40),
            support("by_name", name_token("by"), 45),
        ],
    ));
    registry.register(hypothesis(
        "min_by",
        vec![
            require("loop", EvidenceKind::Loop),
            require("selection_update", EvidenceKind::MinimumUpdate),
            support("min_name", name_token("min"), 40),
            support("by_name", name_token("by"), 45),
        ],
    ));
    registry.register(hypothesis(
        "group_by",
        vec![
            require("loop", EvidenceKind::Loop),
            require("return", EvidenceKind::Return),
            support("group_name", name_token("group"), 45),
            support("by_name", name_token("by"), 45),
        ],
    ));
    registry.register(hypothesis(
        "partition",
        vec![
            require("loop", EvidenceKind::Loop),
            require("conditional", EvidenceKind::Conditional),
            support("partition_name", name_token("partition"), 85),
        ],
    ));
    registry.register(hypothesis(
        "unique",
        vec![
            require("loop", EvidenceKind::Loop),
            require("conditional", EvidenceKind::Conditional),
            support("unique_name", name_token("unique"), 85),
        ],
    ));
    registry.register(hypothesis(
        "distinct",
        vec![
            require("loop", EvidenceKind::Loop),
            require("conditional", EvidenceKind::Conditional),
            support("distinct_name", name_token("distinct"), 85),
        ],
    ));
    registry.register(hypothesis(
        "zip",
        vec![
            require("loop", EvidenceKind::Loop),
            require("return", EvidenceKind::Return),
            support("zip_name", name_token("zip"), 85),
        ],
    ));
    registry.register(hypothesis(
        "flatten",
        vec![
            require("loop", EvidenceKind::Loop),
            require("nested_loop", EvidenceKind::NestedLoop),
            support("flatten_name", name_token("flatten"), 85),
        ],
    ));
    registry.register(hypothesis(
        "contains",
        vec![
            require("loop", EvidenceKind::Loop),
            require("conditional", EvidenceKind::Conditional),
            require("return", EvidenceKind::Return),
            support("boolean_return", EvidenceKind::BooleanReturn, 25),
            support("contains_name", name_token("contains"), 85),
            support("early_return", EvidenceKind::LoopReturn, 30),
        ],
    ));
    registry.register(hypothesis(
        "product",
        vec![
            require("loop", EvidenceKind::Loop),
            require("multiplication", EvidenceKind::MultiplicationMutation),
            require("numeric_return", EvidenceKind::NumericReturn),
            support("multiplication", EvidenceKind::MultiplicationMutation, 70),
            support("return", EvidenceKind::Return, 20),
            support("name", name_token("product"), 30),
        ],
    ));
    registry.register(hypothesis(
        "parser",
        vec![
            require("conditional", EvidenceKind::Conditional),
            support("name", name_token("parse"), 35),
            support("call", EvidenceKind::Call, 15),
        ],
    ));
    registry.register(hypothesis(
        "reducer",
        vec![
            require("loop", EvidenceKind::Loop),
            require("assignment", EvidenceKind::Assignment),
            require("return", EvidenceKind::Return),
            support("call", EvidenceKind::Call, 35),
            support("name", name_token("reducer"), 65),
            support("name", name_token("reduce"), 65),
        ],
    ));
    registry.register(hypothesis(
        "resource_manager",
        vec![
            require("allocation", EvidenceKind::Allocation),
            require("ownership", EvidenceKind::OwnershipTransfer),
            support("reference", EvidenceKind::Reference, 20),
        ],
    ));
    registry.register(hypothesis(
        "search",
        vec![
            require("loop", EvidenceKind::Loop),
            require("conditional", EvidenceKind::Conditional),
            require("return", EvidenceKind::Return),
            support("comparison", EvidenceKind::Comparison, 25),
        ],
    ));
    registry.register(hypothesis(
        "sort",
        vec![
            require("nested_loop", EvidenceKind::NestedLoop),
            support("comparison", EvidenceKind::Comparison, 25),
            support("name", name_token("sort"), 60),
            support("return", EvidenceKind::Return, 15),
        ],
    ));
    registry.register(hypothesis(
        "normalize",
        vec![
            require("division", EvidenceKind::Division),
            require("numeric_return", EvidenceKind::NumericReturn),
            support("name", name_token("normalize"), 70),
            support("return", EvidenceKind::Return, 15),
        ],
    ));
    registry.register(hypothesis(
        "index",
        vec![
            require("loop", EvidenceKind::Loop),
            require("return", EvidenceKind::Return),
            support("conditional", EvidenceKind::Conditional, 15),
            support("name", name_token("index"), 70),
        ],
    ));
    registry.register(hypothesis(
        "linear_search",
        vec![
            require("loop", EvidenceKind::Loop),
            require("conditional", EvidenceKind::Conditional),
            require("comparison", EvidenceKind::Comparison),
            require("early_return", EvidenceKind::LoopReturn),
            support("early_return", EvidenceKind::LoopReturn, 60),
            support("name", name_token("linear"), 40),
        ],
    ));
    registry.register(hypothesis(
        "binary_search",
        vec![
            require("loop", EvidenceKind::Loop),
            require("conditional", EvidenceKind::Conditional),
            require("comparison", EvidenceKind::Comparison),
            require("midpoint", EvidenceKind::Division),
            require("early_return", EvidenceKind::LoopReturn),
            support("midpoint", EvidenceKind::Division, 45),
            support("early_return", EvidenceKind::LoopReturn, 40),
            support("name", name_token("binary"), 30),
        ],
    ));
    registry.register(hypothesis(
        "serializer",
        vec![
            require("return", EvidenceKind::Return),
            support("name", name_token("serialize"), 35),
            support("call", EvidenceKind::Call, 15),
        ],
    ));
    registry.register(hypothesis(
        "state_machine",
        vec![
            require("conditional", EvidenceKind::Conditional),
            require("mutation", EvidenceKind::Mutation),
            support("nested_loop", EvidenceKind::NestedLoop, 15),
        ],
    ));
    registry.register(hypothesis(
        "validator",
        vec![
            require("conditional", EvidenceKind::Conditional),
            require("comparison", EvidenceKind::Comparison),
            support("return", EvidenceKind::Return, 20),
            support("name", name_token("validate"), 35),
        ],
    ));
    registry
}

fn hypothesis(id: &str, rules: Vec<DeterministicRule>) -> RuleBasedHypothesis {
    RuleBasedHypothesis::new(id, 10, rules)
}

fn require(id: &str, evidence: EvidenceKind) -> DeterministicRule {
    DeterministicRule::Require {
        id: id.to_owned(),
        evidence,
    }
}

fn support(id: &str, evidence: EvidenceKind, weight: i32) -> DeterministicRule {
    DeterministicRule::Support {
        id: id.to_owned(),
        evidence,
        weight,
    }
}

fn name_token(token: &str) -> EvidenceKind {
    EvidenceKind::NameToken {
        token: token.to_owned(),
    }
}
