//! Deterministic composition of already-evaluated semantic hypotheses.
//!
//! Composition deliberately runs after constraint evaluation. It neither reads
//! source nor creates language-specific evidence: a pipeline exists only when
//! every stage is an active, independently supported hypothesis.

use std::collections::BTreeMap;

use crate::{
    evidence::{Evidence, EvidenceKind},
    report::{HypothesisReport, HypothesisStatus, SemanticStage},
};

#[derive(Debug, Clone, Copy)]
struct PipelineDefinition {
    id: &'static str,
    stages: &'static [&'static str],
    name_tokens: &'static [&'static str],
}

const DEFINITIONS: &[PipelineDefinition] = &[
    PipelineDefinition {
        id: "filter_map_reduce",
        stages: &["filter", "mapper", "reducer"],
        name_tokens: &["filter", "map", "reduce"],
    },
    PipelineDefinition {
        id: "histogram_maximum",
        stages: &["histogram", "maximum"],
        name_tokens: &["histogram", "maximum"],
    },
    PipelineDefinition {
        id: "group_by_count",
        stages: &["group_by", "counter"],
        name_tokens: &["group", "by", "count"],
    },
    PipelineDefinition {
        id: "prefix_sum_search",
        stages: &["prefix_sum", "search"],
        name_tokens: &["prefix", "sum", "search"],
    },
    PipelineDefinition {
        id: "partition_sort",
        stages: &["partition", "sort"],
        name_tokens: &["partition", "sort"],
    },
    PipelineDefinition {
        id: "flatten_filter",
        stages: &["flatten", "filter"],
        name_tokens: &["flatten", "filter"],
    },
    PipelineDefinition {
        id: "unique_sort",
        stages: &["unique", "sort"],
        name_tokens: &["unique", "sort"],
    },
    PipelineDefinition {
        id: "binary_search_validation",
        stages: &["binary_search", "validator"],
        name_tokens: &["binary", "search", "validate"],
    },
    PipelineDefinition {
        id: "count_if_average",
        stages: &["count_if", "average"],
        name_tokens: &["count", "if", "average"],
    },
    PipelineDefinition {
        id: "sum_normalize",
        stages: &["sum", "normalize"],
        name_tokens: &["sum", "normalize"],
    },
    PipelineDefinition {
        id: "minimum_index",
        stages: &["minimum", "index"],
        name_tokens: &["minimum", "index"],
    },
    PipelineDefinition {
        id: "maximum_index",
        stages: &["maximum", "index"],
        name_tokens: &["maximum", "index"],
    },
    PipelineDefinition {
        id: "filter_map",
        stages: &["filter", "mapper"],
        name_tokens: &["filter", "map"],
    },
    PipelineDefinition {
        id: "map_reduce",
        stages: &["mapper", "reducer"],
        name_tokens: &["map", "reduce"],
    },
    PipelineDefinition {
        id: "filter_reduce",
        stages: &["filter", "reducer"],
        name_tokens: &["filter", "reduce"],
    },
];

/// Returns the most specific matching pipeline. Definition ordering is stable:
/// longer pipelines win, then identifiers break ties lexicographically.
pub fn infer(hypotheses: &[HypothesisReport], evidence: &[Evidence]) -> Vec<SemanticStage> {
    let active = hypotheses
        .iter()
        .filter(|hypothesis| hypothesis.status == HypothesisStatus::Active)
        .map(|hypothesis| (hypothesis.id.as_str(), hypothesis.confidence))
        .collect::<BTreeMap<_, _>>();

    let mut definitions = DEFINITIONS.to_vec();
    definitions.sort_by(|left, right| {
        right
            .stages
            .len()
            .cmp(&left.stages.len())
            .then_with(|| left.id.cmp(right.id))
    });
    definitions
        .into_iter()
        .find(|definition| {
            definition
                .stages
                .iter()
                .all(|stage| active.contains_key(stage))
                && definition.name_tokens.iter().all(|token| {
                    evidence.iter().any(|item| {
                        matches!(&item.kind, EvidenceKind::NameToken { token: observed } if observed == token)
                    })
                })
        })
        .map(|definition| {
            definition
                .stages
                .iter()
                .map(|stage| SemanticStage {
                    hypothesis_id: (*stage).to_owned(),
                    confidence: active[stage],
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::Rejection;

    fn hypothesis(id: &str, confidence: u8) -> HypothesisReport {
        HypothesisReport {
            id: id.to_owned(),
            confidence,
            status: HypothesisStatus::Active,
            supporting_evidence: Vec::new(),
            rejected_evidence: Vec::<Rejection>::new(),
        }
    }

    fn names(tokens: &[&str]) -> Vec<Evidence> {
        tokens
            .iter()
            .map(|token| Evidence {
                provider_id: "test".to_owned(),
                kind: EvidenceKind::NameToken {
                    token: (*token).to_owned(),
                },
                strength: 1,
            })
            .collect()
    }

    #[test]
    fn selects_the_longest_matching_composition_deterministically() {
        let stages = infer(
            &[
                hypothesis("filter", 80),
                hypothesis("mapper", 70),
                hypothesis("reducer", 90),
            ],
            &names(&["filter", "map", "reduce"]),
        );
        assert_eq!(
            stages
                .iter()
                .map(|stage| stage.hypothesis_id.as_str())
                .collect::<Vec<_>>(),
            vec!["filter", "mapper", "reducer"]
        );
    }

    #[test]
    fn every_registered_composition_has_a_stable_stage_order() {
        for definition in DEFINITIONS {
            let hypotheses = definition
                .stages
                .iter()
                .enumerate()
                .map(|(index, stage)| hypothesis(stage, 100 - index as u8))
                .collect::<Vec<_>>();
            let stages = infer(&hypotheses, &names(definition.name_tokens));
            assert_eq!(stages.len(), definition.stages.len());
            assert_eq!(
                stages
                    .iter()
                    .map(|stage| stage.hypothesis_id.as_str())
                    .collect::<Vec<_>>(),
                definition.stages
            );
        }
    }
}
