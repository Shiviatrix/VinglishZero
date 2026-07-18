//! Language-agnostic fact extraction from Semantic IR.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use vz_common::NodeId;
use vz_semantic_ir::{node::Scope, LoopKind, SemanticGraph, SemanticNode, SemanticOp, TypeConcept};

/// Immutable facts for one complete Semantic IR graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactSet {
    pub function_count: u32,
    pub functions: Vec<FunctionFacts>,
}

/// Immutable language-neutral facts for one function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionFacts {
    pub function_name: String,
    pub parameter_count: u32,
    pub variable_count: u32,
    pub assignment_count: u32,
    pub mutation_count: u32,
    pub addition_mutation_count: u32,
    pub multiplication_mutation_count: u32,
    pub increment_mutation_count: u32,
    pub loop_count: u32,
    pub nested_loop_count: u32,
    pub collection_iteration_count: u32,
    pub conditional_count: u32,
    pub conditional_branch_count: u32,
    pub return_count: u32,
    pub boolean_return_count: u32,
    pub loop_return_count: u32,
    pub call_count: u32,
    pub recursion_count: u32,
    pub comparison_count: u32,
    pub maximum_update_count: u32,
    pub minimum_update_count: u32,
    pub division_count: u32,
    pub return_division_count: u32,
    pub allocation_count: u32,
    pub ownership_transfer_count: u32,
    pub reference_count: u32,
    pub api_usage_count: u32,
    pub has_accumulation: bool,
    pub type_relationships: Vec<TypeRelationship>,
}

impl FunctionFacts {
    fn new(name: String, parameter_count: u32) -> Self {
        Self {
            function_name: name,
            parameter_count,
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
        }
    }
}

/// A relationship between a function role and a language-neutral type category.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeRelationship {
    pub role: TypeRole,
    pub category: TypeCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeRole {
    Parameter,
    Return,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeCategory {
    Numeric,
    Boolean,
    Text,
    Unit,
    Collection,
    Mapping,
    Optional,
    Result,
    Reference,
    Pointer,
    Callable,
    Named,
    Dynamic,
    Unknown,
}

/// Extracts immutable facts without retaining references to Semantic IR.
#[derive(Debug, Default, Clone, Copy)]
pub struct FactExtractor;

impl FactExtractor {
    pub fn extract(&self, graph: &SemanticGraph) -> FactSet {
        let functions = graph
            .iter()
            .filter_map(|(_, node)| match node {
                SemanticNode::Function {
                    name,
                    params,
                    return_type,
                    body,
                    ..
                } => Some(self.function(graph, name, params, return_type.as_ref(), body)),
                _ => None,
            })
            .collect::<Vec<_>>();

        FactSet {
            function_count: functions.len() as u32,
            functions,
        }
    }

    fn function(
        &self,
        graph: &SemanticGraph,
        name: &str,
        params: &[vz_semantic_ir::node::Param],
        return_type: Option<&TypeConcept>,
        body: &[NodeId],
    ) -> FunctionFacts {
        let mut facts = FunctionFacts::new(name.to_owned(), params.len() as u32);
        let mut types = BTreeSet::new();
        for parameter in params {
            if let Some(ty) = parameter.type_concept.as_ref() {
                types.insert(TypeRelationship {
                    role: TypeRole::Parameter,
                    category: type_category(ty),
                });
            }
        }
        if let Some(ty) = return_type {
            types.insert(TypeRelationship {
                role: TypeRole::Return,
                category: type_category(ty),
            });
            if matches!(ty, TypeConcept::Boolean) {
                facts.boolean_return_count = 1;
            }
        }

        let mut visited = BTreeSet::new();
        for node in body {
            visit_node(graph, *node, 0, &mut visited, &mut facts, &mut types);
        }
        facts.type_relationships = types.into_iter().collect();
        facts.has_accumulation = facts.loop_count > 0 && facts.addition_mutation_count > 0;
        facts
    }
}

fn visit_node(
    graph: &SemanticGraph,
    id: NodeId,
    loop_depth: u32,
    visited: &mut BTreeSet<NodeId>,
    facts: &mut FunctionFacts,
    types: &mut BTreeSet<TypeRelationship>,
) {
    if !visited.insert(id) {
        return;
    }
    let Some(node) = graph.get(id) else {
        return;
    };

    match node {
        SemanticNode::Variable {
            type_concept,
            initializer_id,
            scope,
            ..
        } => {
            facts.variable_count += 1;
            if matches!(scope, Scope::Local) {
                if let Some(ty) = type_concept.as_ref() {
                    types.insert(TypeRelationship {
                        role: TypeRole::Local,
                        category: type_category(ty),
                    });
                }
            }
            if let Some(initializer) = initializer_id {
                visit_node(graph, *initializer, loop_depth, visited, facts, types);
            }
        }
        SemanticNode::Assignment {
            target, value, op, ..
        } => {
            facts.assignment_count += 1;
            if let Some(operation) = op {
                facts.mutation_count += 1;
                match operation {
                    SemanticOp::Add => {
                        facts.addition_mutation_count += 1;
                        if is_integer_literal(graph, *value, 1) {
                            facts.increment_mutation_count += 1;
                        }
                    }
                    SemanticOp::Mul => facts.multiplication_mutation_count += 1,
                    SemanticOp::Div => facts.division_count += 1,
                    _ => {}
                }
            }
            visit_node(graph, *target, loop_depth, visited, facts, types);
            visit_node(graph, *value, loop_depth, visited, facts, types);
        }
        SemanticNode::Loop { kind, body, .. } => {
            facts.loop_count += 1;
            if loop_depth > 0 {
                facts.nested_loop_count += 1;
            }
            match kind {
                LoopKind::While { condition } => {
                    visit_node(graph, *condition, loop_depth, visited, facts, types);
                }
                LoopKind::ForEach { iterable, .. } => {
                    facts.collection_iteration_count += 1;
                    visit_node(graph, *iterable, loop_depth, visited, facts, types);
                }
                LoopKind::Count { times } => {
                    visit_node(graph, *times, loop_depth, visited, facts, types);
                }
                LoopKind::Infinite => {}
            }
            for child in body {
                visit_node(graph, *child, loop_depth + 1, visited, facts, types);
            }
        }
        SemanticNode::Conditional {
            condition,
            then_body,
            else_body,
            ..
        } => {
            facts.conditional_count += 1;
            facts.conditional_branch_count += 1 + u32::from(else_body.is_some());
            if is_maximum_update(graph, *condition, then_body) {
                facts.maximum_update_count += 1;
            }
            if is_minimum_update(graph, *condition, then_body) {
                facts.minimum_update_count += 1;
            }
            visit_node(graph, *condition, loop_depth, visited, facts, types);
            for child in then_body {
                visit_node(graph, *child, loop_depth, visited, facts, types);
            }
            if let Some(else_body) = else_body {
                for child in else_body {
                    visit_node(graph, *child, loop_depth, visited, facts, types);
                }
            }
        }
        SemanticNode::Return { value, .. } => {
            facts.return_count += 1;
            if loop_depth > 0 {
                facts.loop_return_count += 1;
            }
            if let Some(value) = value {
                if is_division_expression(graph, *value) {
                    facts.return_division_count += 1;
                }
                visit_node(graph, *value, loop_depth, visited, facts, types);
            }
        }
        SemanticNode::Call {
            callee, arguments, ..
        } => {
            facts.call_count += 1;
            if matches!(graph.get(*callee), Some(SemanticNode::Identifier { name, .. }) if name == &facts.function_name)
            {
                facts.recursion_count += 1;
            }
            visit_node(graph, *callee, loop_depth, visited, facts, types);
            for argument in arguments {
                visit_node(graph, argument.value, loop_depth, visited, facts, types);
            }
        }
        SemanticNode::BinaryOp {
            op, left, right, ..
        } => {
            if matches!(
                op,
                SemanticOp::Eq
                    | SemanticOp::NotEq
                    | SemanticOp::Lt
                    | SemanticOp::LtEq
                    | SemanticOp::Gt
                    | SemanticOp::GtEq
            ) {
                facts.comparison_count += 1;
            }
            if matches!(op, SemanticOp::Div) {
                facts.division_count += 1;
            }
            visit_node(graph, *left, loop_depth, visited, facts, types);
            visit_node(graph, *right, loop_depth, visited, facts, types);
        }
        SemanticNode::UnaryOp { operand, .. }
        | SemanticNode::Reference {
            target: operand, ..
        }
        | SemanticNode::Dereference {
            target: operand, ..
        }
        | SemanticNode::Drop {
            target: operand, ..
        } => {
            if matches!(node, SemanticNode::Reference { .. }) {
                facts.reference_count += 1;
            }
            if matches!(node, SemanticNode::Drop { .. }) {
                facts.ownership_transfer_count += 1;
            }
            visit_node(graph, *operand, loop_depth, visited, facts, types);
        }
        SemanticNode::Allocation { initializer, .. } => {
            facts.allocation_count += 1;
            if let Some(initializer) = initializer {
                visit_node(graph, *initializer, loop_depth, visited, facts, types);
            }
        }
        SemanticNode::ApiUsage { call_node, .. } => {
            facts.api_usage_count += 1;
            visit_node(graph, *call_node, loop_depth, visited, facts, types);
        }
        SemanticNode::Collection { elements, .. } => {
            for element in elements {
                visit_node(graph, *element, loop_depth, visited, facts, types);
            }
        }
        SemanticNode::FieldAccess { object, .. } => {
            visit_node(graph, *object, loop_depth, visited, facts, types);
        }
        SemanticNode::IndexAccess { object, index, .. } => {
            visit_node(graph, *object, loop_depth, visited, facts, types);
            visit_node(graph, *index, loop_depth, visited, facts, types);
        }
        SemanticNode::MapLiteral { entries, .. } => {
            for (key, value) in entries {
                visit_node(graph, *key, loop_depth, visited, facts, types);
                visit_node(graph, *value, loop_depth, visited, facts, types);
            }
        }
        SemanticNode::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                visit_node(graph, *value, loop_depth, visited, facts, types);
            }
        }
        SemanticNode::Async { inner, .. } => {
            visit_node(graph, *inner, loop_depth, visited, facts, types);
        }
        SemanticNode::Transaction {
            body,
            on_commit,
            on_rollback,
            ..
        } => {
            for child in body {
                visit_node(graph, *child, loop_depth, visited, facts, types);
            }
            for child in on_commit
                .iter()
                .flatten()
                .chain(on_rollback.iter().flatten())
            {
                visit_node(graph, *child, loop_depth, visited, facts, types);
            }
        }
        SemanticNode::PatternMatch {
            subject,
            arms,
            otherwise,
            ..
        } => {
            facts.conditional_count += 1;
            facts.conditional_branch_count += arms.len() as u32 + u32::from(otherwise.is_some());
            visit_node(graph, *subject, loop_depth, visited, facts, types);
            for arm in arms {
                for child in &arm.body {
                    visit_node(graph, *child, loop_depth, visited, facts, types);
                }
            }
            if let Some(otherwise) = otherwise {
                for child in otherwise {
                    visit_node(graph, *child, loop_depth, visited, facts, types);
                }
            }
        }
        _ => {}
    }
}

/// A maximum candidate is selected by assigning a value only when a greater
/// comparison succeeds. This remains purely structural and language agnostic.
fn is_maximum_update(graph: &SemanticGraph, condition: NodeId, then_body: &[NodeId]) -> bool {
    matches!(
        graph.get(condition),
        Some(SemanticNode::BinaryOp {
            op: SemanticOp::Gt | SemanticOp::GtEq,
            ..
        })
    ) && then_body.iter().any(|id| {
        matches!(
            graph.get(*id),
            Some(SemanticNode::Assignment { op: None, .. })
        )
    })
}

/// A minimum candidate is selected by assigning a value only when a smaller
/// comparison succeeds. Like maximum selection, this is independent of source
/// syntax and adapter identity.
fn is_minimum_update(graph: &SemanticGraph, condition: NodeId, then_body: &[NodeId]) -> bool {
    matches!(
        graph.get(condition),
        Some(SemanticNode::BinaryOp {
            op: SemanticOp::Lt | SemanticOp::LtEq,
            ..
        })
    ) && then_body.iter().any(|id| {
        matches!(
            graph.get(*id),
            Some(SemanticNode::Assignment { op: None, .. })
        )
    })
}

/// Average-like calculations return the quotient itself. A division used only
/// to compute an intermediate value, such as a search midpoint, is not the
/// defining result of the function.
fn is_division_expression(graph: &SemanticGraph, id: NodeId) -> bool {
    matches!(
        graph.get(id),
        Some(SemanticNode::BinaryOp {
            op: SemanticOp::Div,
            ..
        })
    )
}

fn is_integer_literal(graph: &SemanticGraph, id: NodeId, expected: i64) -> bool {
    matches!(
        graph.get(id),
        Some(SemanticNode::Literal { value: vz_semantic_ir::node::LiteralValue::Integer(value), .. }) if *value == expected
    )
}

fn type_category(ty: &TypeConcept) -> TypeCategory {
    match ty {
        TypeConcept::Integer { .. } | TypeConcept::FloatingPoint { .. } => TypeCategory::Numeric,
        TypeConcept::Boolean => TypeCategory::Boolean,
        TypeConcept::Text => TypeCategory::Text,
        TypeConcept::Unit => TypeCategory::Unit,
        TypeConcept::Collection(_)
        | TypeConcept::Array { .. }
        | TypeConcept::Set(_)
        | TypeConcept::Tuple(_) => TypeCategory::Collection,
        TypeConcept::Map { .. } => TypeCategory::Mapping,
        TypeConcept::Optional(_) => TypeCategory::Optional,
        TypeConcept::Result { .. } => TypeCategory::Result,
        TypeConcept::Reference { .. } => TypeCategory::Reference,
        TypeConcept::RawPointer(_) => TypeCategory::Pointer,
        TypeConcept::Callable { .. } => TypeCategory::Callable,
        TypeConcept::Named { .. } => TypeCategory::Named,
        TypeConcept::Dynamic | TypeConcept::Inferred => TypeCategory::Dynamic,
        TypeConcept::Unknown | TypeConcept::Extension { .. } => TypeCategory::Unknown,
    }
}
