//! Semantic engine boundaries.
//!
//! The engine consumes Semantic IR and produces language-agnostic outputs.
//! It does not parse source code and it does not know which language produced
//! a given IR graph.

/// Intent inference boundaries.
pub mod intent {
    /// Placeholder for future intent inference requests.
    #[derive(Debug, Clone, Default)]
    pub struct Request;
}

/// Deterministic, language-agnostic code explanation.
pub mod explanation {
    use vz_common::NodeId;
    use vz_semantic_ir::{LoopKind, SemanticGraph, SemanticNode, SemanticOp};

    /// Produces a stable, human-readable explanation from Semantic IR alone.
    ///
    /// This module deliberately does not know which adapter produced the graph.
    pub fn explain(graph: &SemanticGraph) -> String {
        let functions: Vec<_> = graph
            .iter()
            .filter_map(|(_, node)| match node {
                SemanticNode::Function {
                    name,
                    params,
                    return_type,
                    body,
                    ..
                } => Some((name, params, return_type, body)),
                _ => None,
            })
            .collect();

        functions
            .into_iter()
            .map(|(name, params, return_type, body)| {
                let operations = operations(graph, body);
                let purpose = purpose(&operations);
                let inputs = if params.is_empty() {
                    "none".to_owned()
                } else {
                    params
                        .iter()
                        .map(|parameter| parameter.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                let produces = return_type
                    .as_ref()
                    .map(|ty| ty.describe())
                    .unwrap_or_else(|| "an unspecified value".to_owned());
                let operation_lines = if operations.is_empty() {
                    "- No supported operations\n".to_owned()
                } else {
                    operations
                        .iter()
                        .map(|operation| format!("- {operation}\n"))
                        .collect()
                };

                format!(
                    "Function \"{name}\"\n\nPurpose:\n{purpose}\n\nInputs:\n{inputs}\n\nProduces:\n{produces}\n\nOperations:\n\n{operation_lines}"
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn purpose(operations: &[String]) -> &'static str {
        let loops = operations
            .iter()
            .any(|operation| operation.starts_with("Iterate"));
        let additions = operations.iter().any(|operation| operation == "Add values");
        if loops && additions {
            "Accumulates values inside a loop."
        } else if loops {
            "Performs repeated work inside a loop."
        } else if operations
            .iter()
            .any(|operation| operation == "Return result")
        {
            "Computes and returns a result."
        } else {
            "Performs a sequence of semantic operations."
        }
    }

    fn operations(graph: &SemanticGraph, body: &[NodeId]) -> Vec<String> {
        let mut operations = Vec::new();
        for id in body {
            collect_operations(graph, *id, &mut operations);
        }
        operations
    }

    fn collect_operations(graph: &SemanticGraph, id: NodeId, operations: &mut Vec<String>) {
        let Some(node) = graph.get(id) else {
            return;
        };
        match node {
            SemanticNode::Loop { kind, body, .. } => {
                operations.push(match kind {
                    LoopKind::While { .. } => "Iterate while condition".to_owned(),
                    LoopKind::ForEach { .. } => "Iterate collection".to_owned(),
                    LoopKind::Count { .. } => "Repeat a fixed number of times".to_owned(),
                    LoopKind::Infinite => "Repeat until an explicit exit".to_owned(),
                });
                for child in body {
                    collect_operations(graph, *child, operations);
                }
            }
            SemanticNode::Conditional {
                then_body,
                else_body,
                ..
            } => {
                operations.push("Evaluate a conditional branch".to_owned());
                for child in then_body {
                    collect_operations(graph, *child, operations);
                }
                if let Some(else_body) = else_body {
                    for child in else_body {
                        collect_operations(graph, *child, operations);
                    }
                }
            }
            SemanticNode::Assignment { op, .. } => operations.push(match op {
                Some(SemanticOp::Add) => "Add values".to_owned(),
                Some(SemanticOp::Sub) => "Subtract values".to_owned(),
                Some(SemanticOp::Mul) => "Multiply values".to_owned(),
                Some(SemanticOp::Div) => "Divide values".to_owned(),
                Some(_) => "Mutate a value".to_owned(),
                None => "Assign a value".to_owned(),
            }),
            SemanticNode::Call { callee, .. } => {
                let name = match graph.get(*callee) {
                    Some(SemanticNode::Identifier { name, .. }) => name.as_str(),
                    _ => "a callable",
                };
                operations.push(format!("Call {name}"));
            }
            SemanticNode::Return { .. } => operations.push("Return result".to_owned()),
            _ => {}
        }
    }
}

/// Bug detection boundaries.
pub mod bug_detection {
    /// Placeholder for future bug detection requests.
    #[derive(Debug, Clone, Default)]
    pub struct Request;
}

/// Semantic search boundaries.
pub mod semantic_search {
    /// Placeholder for future semantic search requests.
    #[derive(Debug, Clone, Default)]
    pub struct Request;
}

/// Automatic repair boundaries.
pub mod repair {
    /// Placeholder for future repair requests.
    #[derive(Debug, Clone, Default)]
    pub struct Request;
}

/// Architecture reasoning boundaries.
pub mod architecture {
    /// Placeholder for future architecture reasoning requests.
    #[derive(Debug, Clone, Default)]
    pub struct Request;
}

/// API reasoning boundaries.
pub mod api {
    /// Placeholder for future API reasoning requests.
    #[derive(Debug, Clone, Default)]
    pub struct Request;
}

/// Documentation generation boundaries.
pub mod documentation {
    /// Placeholder for future documentation generation requests.
    #[derive(Debug, Clone, Default)]
    pub struct Request;
}

/// Test generation boundaries.
pub mod test_generation {
    /// Placeholder for future test generation requests.
    #[derive(Debug, Clone, Default)]
    pub struct Request;
}

/// AI interaction boundaries.
pub mod ai_interaction {
    /// Placeholder for future AI interaction requests.
    #[derive(Debug, Clone, Default)]
    pub struct Request;
}
