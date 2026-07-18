use vz_common::{LanguageTag, Mutability, NodeId, SemanticSpan, Visibility};
use vz_reasoning::{
    Evidence, EvidenceKind, EvidenceProvider, FactExtractor, ReasoningEngine, RuleBasedHypothesis,
};
use vz_semantic_ir::{
    node::Param, LoopKind, Metadata, SemanticGraph, SemanticNode, SemanticOp, TypeConcept,
};

fn graph_with_accumulator() -> SemanticGraph {
    let mut graph = SemanticGraph::new();
    let span = SemanticSpan::new(vz_common::SourceId::UNKNOWN, 0, 0, LanguageTag::Unknown);

    graph.insert(SemanticNode::Identifier {
        id: NodeId::new(1),
        name: "ready".to_owned(),
        resolved_to: None,
        type_concept: Some(TypeConcept::Boolean),
        span: span.clone(),
        metadata: Metadata::default(),
    });
    graph.insert(SemanticNode::Identifier {
        id: NodeId::new(2),
        name: "total".to_owned(),
        resolved_to: None,
        type_concept: Some(TypeConcept::Integer {
            bits: None,
            signed: true,
        }),
        span: span.clone(),
        metadata: Metadata::default(),
    });
    graph.insert(SemanticNode::Identifier {
        id: NodeId::new(3),
        name: "value".to_owned(),
        resolved_to: None,
        type_concept: Some(TypeConcept::Integer {
            bits: None,
            signed: true,
        }),
        span: span.clone(),
        metadata: Metadata::default(),
    });
    graph.insert(SemanticNode::Assignment {
        id: NodeId::new(4),
        target: NodeId::new(2),
        value: NodeId::new(3),
        op: Some(SemanticOp::Add),
        span: span.clone(),
        metadata: Metadata::default(),
    });
    graph.insert(SemanticNode::Loop {
        id: NodeId::new(5),
        kind: LoopKind::While {
            condition: NodeId::new(1),
        },
        body: vec![NodeId::new(4)],
        span: span.clone(),
        metadata: Metadata::default(),
    });
    graph.insert(SemanticNode::Return {
        id: NodeId::new(6),
        value: Some(NodeId::new(2)),
        span: span.clone(),
        metadata: Metadata::default(),
    });
    graph.insert(SemanticNode::Function {
        id: NodeId::new(7),
        name: "calculate".to_owned(),
        params: vec![Param {
            name: "values".to_owned(),
            type_concept: Some(TypeConcept::Collection(Box::new(TypeConcept::Integer {
                bits: None,
                signed: true,
            }))),
            mutability: Mutability::Immutable,
            has_default: false,
            span,
        }],
        return_type: Some(TypeConcept::Integer {
            bits: None,
            signed: true,
        }),
        body: vec![NodeId::new(5), NodeId::new(6)],
        visibility: Visibility::Untracked,
        is_async: false,
        is_foreign: false,
        generic_params: Vec::new(),
        span: SemanticSpan::dummy(),
        metadata: Metadata::default(),
    });
    graph
}

#[test]
fn fact_extraction_is_language_agnostic_and_immutable() {
    let facts = FactExtractor.extract(&graph_with_accumulator());
    let function = &facts.functions[0];

    assert_eq!(facts.function_count, 1);
    assert_eq!(function.loop_count, 1);
    assert_eq!(function.mutation_count, 1);
    assert_eq!(function.addition_mutation_count, 1);
    assert!(function.has_accumulation);
}

#[test]
fn deterministic_rules_activate_accumulator_and_eliminate_average_without_division() {
    let report = ReasoningEngine::new().analyze(&graph_with_accumulator());
    let hypotheses = &report.function_reports[0].hypotheses;
    let accumulator = hypotheses
        .iter()
        .find(|hypothesis| hypothesis.id == "accumulator")
        .unwrap();
    let average = hypotheses
        .iter()
        .find(|hypothesis| hypothesis.id == "average")
        .unwrap();

    assert_eq!(
        accumulator.status,
        vz_reasoning::report::HypothesisStatus::Active
    );
    assert!(accumulator.confidence > 0);
    assert_eq!(
        average.status,
        vz_reasoning::report::HypothesisStatus::Eliminated
    );
    assert_eq!(average.confidence, 0);
    assert!(average.rejected_evidence.iter().any(|rejection| matches!(
        rejection,
        vz_reasoning::report::Rejection::MissingRequiredEvidence {
            evidence: vz_reasoning::EvidenceKind::Division,
            ..
        }
    )));
}

#[test]
fn identical_semantic_ir_produces_identical_serialized_reports() {
    let engine = ReasoningEngine::new();
    let first = serde_json::to_string(&engine.analyze(&graph_with_accumulator())).unwrap();
    let second = serde_json::to_string(&engine.analyze(&graph_with_accumulator())).unwrap();

    assert_eq!(first, second);
}

struct AllocationObservation;

impl EvidenceProvider for AllocationObservation {
    fn id(&self) -> &'static str {
        "test_allocation_observation"
    }

    fn collect(&self, _: &vz_reasoning::FunctionFacts) -> Vec<Evidence> {
        vec![Evidence {
            provider_id: self.id().to_owned(),
            kind: EvidenceKind::Allocation,
            strength: 1,
        }]
    }
}

#[test]
fn providers_and_hypotheses_are_independently_extensible() {
    use vz_reasoning::hypotheses::DeterministicRule;

    let mut engine = ReasoningEngine::new();
    engine.add_provider(AllocationObservation);
    engine.register_hypothesis(RuleBasedHypothesis::new(
        "allocation_observer",
        10,
        vec![DeterministicRule::Support {
            id: "allocation".to_owned(),
            evidence: EvidenceKind::Allocation,
            weight: 25,
        }],
    ));

    let report = engine.analyze(&graph_with_accumulator());
    let hypothesis = report.function_reports[0]
        .hypotheses
        .iter()
        .find(|hypothesis| hypothesis.id == "allocation_observer")
        .unwrap();
    assert_eq!(hypothesis.confidence, 35);
}

#[derive(Clone, Copy)]
enum CollectionPatternShape {
    Plain,
    Addition { increment: bool },
    Conditional { early_return: bool },
    ConditionalIncrement,
    Maximum,
    Minimum,
    NestedLoop,
}

fn graph_with_collection_pattern(
    name: &str,
    returns_boolean: bool,
    shape: CollectionPatternShape,
) -> SemanticGraph {
    let mut graph = SemanticGraph::new();
    let span = SemanticSpan::new(vz_common::SourceId::UNKNOWN, 0, 0, LanguageTag::Unknown);
    let mut next = 1_u32;
    let mut id = || {
        let current = NodeId::new(next);
        next += 1;
        current
    };
    let condition = id();
    graph.insert(SemanticNode::Identifier {
        id: condition,
        name: "has_more".to_owned(),
        resolved_to: None,
        type_concept: Some(TypeConcept::Boolean),
        span: span.clone(),
        metadata: Metadata::default(),
    });

    let mut loop_body = Vec::new();
    match shape {
        CollectionPatternShape::Plain => {}
        CollectionPatternShape::Addition { increment } => {
            let target = id();
            let value = id();
            graph.insert(SemanticNode::Identifier {
                id: target,
                name: "total".to_owned(),
                resolved_to: None,
                type_concept: Some(integer()),
                span: span.clone(),
                metadata: Metadata::default(),
            });
            graph.insert(if increment {
                SemanticNode::Literal {
                    id: value,
                    value: vz_semantic_ir::node::LiteralValue::Integer(1),
                    type_concept: integer(),
                    span: span.clone(),
                    metadata: Metadata::default(),
                }
            } else {
                SemanticNode::Identifier {
                    id: value,
                    name: "value".to_owned(),
                    resolved_to: None,
                    type_concept: Some(integer()),
                    span: span.clone(),
                    metadata: Metadata::default(),
                }
            });
            let mutation = id();
            graph.insert(SemanticNode::Assignment {
                id: mutation,
                target,
                value,
                op: Some(SemanticOp::Add),
                span: span.clone(),
                metadata: Metadata::default(),
            });
            loop_body.push(mutation);
        }
        CollectionPatternShape::Conditional { early_return } => {
            let matches = id();
            graph.insert(SemanticNode::Identifier {
                id: matches,
                name: "matches".to_owned(),
                resolved_to: None,
                type_concept: Some(TypeConcept::Boolean),
                span: span.clone(),
                metadata: Metadata::default(),
            });
            let then_body = if early_return {
                let returned = id();
                graph.insert(SemanticNode::Return {
                    id: returned,
                    value: None,
                    span: span.clone(),
                    metadata: Metadata::default(),
                });
                vec![returned]
            } else {
                Vec::new()
            };
            let conditional = id();
            graph.insert(SemanticNode::Conditional {
                id: conditional,
                condition: matches,
                then_body,
                else_body: None,
                span: span.clone(),
                metadata: Metadata::default(),
            });
            loop_body.push(conditional);
        }
        CollectionPatternShape::ConditionalIncrement => {
            let matches = id();
            let count = id();
            let one = id();
            graph.insert(SemanticNode::Identifier {
                id: matches,
                name: "matches".to_owned(),
                resolved_to: None,
                type_concept: Some(TypeConcept::Boolean),
                span: span.clone(),
                metadata: Metadata::default(),
            });
            graph.insert(SemanticNode::Identifier {
                id: count,
                name: "count".to_owned(),
                resolved_to: None,
                type_concept: Some(integer()),
                span: span.clone(),
                metadata: Metadata::default(),
            });
            graph.insert(SemanticNode::Literal {
                id: one,
                value: vz_semantic_ir::node::LiteralValue::Integer(1),
                type_concept: integer(),
                span: span.clone(),
                metadata: Metadata::default(),
            });
            let mutation = id();
            graph.insert(SemanticNode::Assignment {
                id: mutation,
                target: count,
                value: one,
                op: Some(SemanticOp::Add),
                span: span.clone(),
                metadata: Metadata::default(),
            });
            let conditional = id();
            graph.insert(SemanticNode::Conditional {
                id: conditional,
                condition: matches,
                then_body: vec![mutation],
                else_body: None,
                span: span.clone(),
                metadata: Metadata::default(),
            });
            loop_body.push(conditional);
        }
        CollectionPatternShape::Maximum | CollectionPatternShape::Minimum => {
            let candidate = id();
            let selected = id();
            graph.insert(SemanticNode::Identifier {
                id: candidate,
                name: "value".to_owned(),
                resolved_to: None,
                type_concept: Some(integer()),
                span: span.clone(),
                metadata: Metadata::default(),
            });
            graph.insert(SemanticNode::Identifier {
                id: selected,
                name: "best".to_owned(),
                resolved_to: None,
                type_concept: Some(integer()),
                span: span.clone(),
                metadata: Metadata::default(),
            });
            let comparison = id();
            graph.insert(SemanticNode::BinaryOp {
                id: comparison,
                op: if matches!(shape, CollectionPatternShape::Maximum) {
                    SemanticOp::Gt
                } else {
                    SemanticOp::Lt
                },
                left: candidate,
                right: selected,
                result_type: Some(TypeConcept::Boolean),
                span: span.clone(),
                metadata: Metadata::default(),
            });
            let assignment = id();
            graph.insert(SemanticNode::Assignment {
                id: assignment,
                target: selected,
                value: candidate,
                op: None,
                span: span.clone(),
                metadata: Metadata::default(),
            });
            let conditional = id();
            graph.insert(SemanticNode::Conditional {
                id: conditional,
                condition: comparison,
                then_body: vec![assignment],
                else_body: None,
                span: span.clone(),
                metadata: Metadata::default(),
            });
            loop_body.push(conditional);
        }
        CollectionPatternShape::NestedLoop => {
            let inner = id();
            graph.insert(SemanticNode::Loop {
                id: inner,
                kind: LoopKind::Infinite,
                body: Vec::new(),
                span: span.clone(),
                metadata: Metadata::default(),
            });
            loop_body.push(inner);
        }
    }

    let loop_id = id();
    graph.insert(SemanticNode::Loop {
        id: loop_id,
        kind: LoopKind::While { condition },
        body: loop_body,
        span: span.clone(),
        metadata: Metadata::default(),
    });
    let return_id = id();
    graph.insert(SemanticNode::Return {
        id: return_id,
        value: None,
        span: span.clone(),
        metadata: Metadata::default(),
    });
    let function = id();
    graph.insert(SemanticNode::Function {
        id: function,
        name: name.to_owned(),
        params: Vec::new(),
        return_type: Some(if returns_boolean {
            TypeConcept::Boolean
        } else {
            integer()
        }),
        body: vec![loop_id, return_id],
        visibility: Visibility::Untracked,
        is_async: false,
        is_foreign: false,
        generic_params: Vec::new(),
        span,
        metadata: Metadata::default(),
    });
    graph
}

fn integer() -> TypeConcept {
    TypeConcept::Integer {
        bits: None,
        signed: true,
    }
}

fn assert_active(name: &str, returns_boolean: bool, shape: CollectionPatternShape, expected: &str) {
    let report = ReasoningEngine::new().analyze(&graph_with_collection_pattern(
        name,
        returns_boolean,
        shape,
    ));
    assert!(report.function_reports[0]
        .hypotheses
        .iter()
        .any(|hypothesis| {
            hypothesis.id == expected
                && hypothesis.status == vz_reasoning::report::HypothesisStatus::Active
        }));
}

#[test]
fn collection_pattern_hypotheses_are_deterministic_semantic_rules() {
    use CollectionPatternShape::{
        Addition, Conditional, ConditionalIncrement, Maximum, Minimum, NestedLoop, Plain,
    };

    assert_active("sum_values", false, Addition { increment: false }, "sum");
    assert_active("find_minimum", false, Minimum, "minimum");
    assert_active("count_if", false, ConditionalIncrement, "count_if");
    assert_active("count_if", false, Addition { increment: true }, "counter");
    assert_active("any_match", true, Conditional { early_return: true }, "any");
    assert_active(
        "all_positive",
        true,
        Conditional { early_return: true },
        "all",
    );
    assert_active(
        "find_first",
        false,
        Conditional { early_return: true },
        "find_first",
    );
    assert_active(
        "find_last",
        false,
        Conditional {
            early_return: false,
        },
        "find_last",
    );
    assert_active("reverse_traversal", false, Plain, "reverse_traversal");
    assert_active(
        "prefix_sum",
        false,
        Addition { increment: false },
        "prefix_sum",
    );
    assert_active(
        "frequency_counter",
        false,
        Addition { increment: true },
        "frequency_counter",
    );
    assert_active(
        "histogram",
        false,
        Addition { increment: true },
        "histogram",
    );
    assert_active("max_by", false, Maximum, "max_by");
    assert_active("min_by", false, Minimum, "min_by");
    assert_active("group_by", false, Plain, "group_by");
    assert_active(
        "partition",
        false,
        Conditional {
            early_return: false,
        },
        "partition",
    );
    assert_active(
        "unique",
        false,
        Conditional {
            early_return: false,
        },
        "unique",
    );
    assert_active(
        "distinct",
        false,
        Conditional {
            early_return: false,
        },
        "distinct",
    );
    assert_active("zip_items", false, Plain, "zip");
    assert_active("flatten", false, NestedLoop, "flatten");
    assert_active(
        "contains",
        true,
        Conditional { early_return: true },
        "contains",
    );
}

#[test]
fn engine_adds_a_pipeline_without_changing_the_primary_hypothesis_reports() {
    let mut engine = ReasoningEngine::new();
    for id in ["filter", "mapper", "reducer"] {
        engine.register_hypothesis(RuleBasedHypothesis::new(id, 10, Vec::new()));
    }

    let mut graph = graph_with_accumulator();
    let Some(SemanticNode::Function { name, .. }) = graph.get_mut(NodeId::new(7)) else {
        panic!("test graph must contain its function node");
    };
    *name = "filter_map_reduce".to_owned();

    let report = engine.analyze(&graph);
    let function = &report.function_reports[0];
    assert_eq!(function.primary_intent.as_deref(), Some("accumulator"));
    assert!(function
        .hypotheses
        .iter()
        .any(|hypothesis| hypothesis.id == "accumulator"));
    assert_eq!(
        function
            .semantic_pipeline
            .iter()
            .map(|stage| stage.hypothesis_id.as_str())
            .collect::<Vec<_>>(),
        vec!["filter", "mapper", "reducer"]
    );
}
