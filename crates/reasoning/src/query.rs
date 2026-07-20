//! Deterministic semantic queries over [`IntentReport`] values.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    evidence::EvidenceKind,
    report::{FunctionIntentReport, HypothesisStatus, IntentReport},
};

/// A primary intent or pipeline stage is always queryable. A non-primary
/// hypothesis is a useful repository match only once deterministic rule
/// scoring has reached this confidence floor; otherwise it remains visible in
/// the detailed report without flooding a semantic search with weak aliases.
const QUERY_HYPOTHESIS_CONFIDENCE_FLOOR: u8 = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryExpression {
    Term(String),
    Not(Box<QueryExpression>),
    And(Box<QueryExpression>, Box<QueryExpression>),
    Or(Box<QueryExpression>, Box<QueryExpression>),
    Then(Box<QueryExpression>, Box<QueryExpression>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryMatch {
    pub source: String,
    pub function_name: String,
    pub primary_intent: Option<String>,
    pub semantic_pipeline: Vec<String>,
    pub score: u16,
    pub matched_fields: Vec<QueryMatchField>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMatchField {
    PrimaryIntent { id: String },
    PipelineStage { id: String },
    Hypothesis { id: String },
    Evidence { kind: String },
    FunctionMetadata { token: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryError {
    Empty,
    MissingOperand { operator: String },
    UnsupportedToken { token: String },
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("query expression is empty"),
            Self::MissingOperand { operator } => write!(f, "missing operand for {operator}"),
            Self::UnsupportedToken { token } => write!(f, "unsupported query token: {token}"),
        }
    }
}

impl std::error::Error for QueryError {}

pub fn parse(input: &str) -> Result<QueryExpression, QueryError> {
    let normalized = input
        .trim()
        .to_ascii_lowercase()
        .replace("maximum finder", "maximum")
        .replace("minimum finder", "minimum")
        .replace("max by", "max_by")
        .replace("min by", "min_by")
        .replace("binary search", "binary_search")
        .replace("group by", "group_by")
        .replace("count if", "count_if")
        .replace("prefix sum", "prefix_sum")
        .replace("functions that ", "")
        .replace("function that ", "");
    if normalized.is_empty() {
        return Err(QueryError::Empty);
    }
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    if tokens
        .iter()
        .any(|token| token.contains('(') || token.contains(')'))
    {
        return Err(QueryError::UnsupportedToken {
            token: "parentheses".to_owned(),
        });
    }
    parse_or(&tokens)
}

fn parse_or(tokens: &[&str]) -> Result<QueryExpression, QueryError> {
    let groups = split(tokens, "or");
    let mut expression = parse_and(groups.first().copied().unwrap_or_default())?;
    for group in groups.iter().skip(1) {
        expression = QueryExpression::Or(Box::new(expression), Box::new(parse_and(group)?));
    }
    Ok(expression)
}

fn parse_and(tokens: &[&str]) -> Result<QueryExpression, QueryError> {
    if tokens.is_empty() {
        return Err(QueryError::MissingOperand {
            operator: "OR".to_owned(),
        });
    }
    let mut index = 0;
    let mut expression = parse_unary(tokens, &mut index)?;
    while index < tokens.len() {
        let operator = tokens[index];
        if operator == "and" {
            index += 1;
            expression = QueryExpression::And(
                Box::new(expression),
                Box::new(parse_unary(tokens, &mut index)?),
            );
        } else if operator == "then" {
            index += 1;
            expression = QueryExpression::Then(
                Box::new(expression),
                Box::new(parse_unary(tokens, &mut index)?),
            );
        } else {
            expression = QueryExpression::And(
                Box::new(expression),
                Box::new(parse_unary(tokens, &mut index)?),
            );
        }
    }
    Ok(expression)
}

fn parse_unary(tokens: &[&str], index: &mut usize) -> Result<QueryExpression, QueryError> {
    let Some(token) = tokens.get(*index) else {
        return Err(QueryError::MissingOperand {
            operator: "AND".to_owned(),
        });
    };
    *index += 1;
    if *token == "not" {
        return Ok(QueryExpression::Not(Box::new(parse_unary(tokens, index)?)));
    }
    if matches!(*token, "and" | "or" | "then") {
        return Err(QueryError::MissingOperand {
            operator: token.to_ascii_uppercase(),
        });
    }
    Ok(QueryExpression::Term(canonical(token)))
}

fn split<'a>(tokens: &'a [&'a str], operator: &str) -> Vec<&'a [&'a str]> {
    let mut groups = Vec::new();
    let mut start = 0;
    for (index, token) in tokens.iter().enumerate() {
        if *token == operator {
            groups.push(&tokens[start..index]);
            start = index + 1;
        }
    }
    groups.push(&tokens[start..]);
    groups
}

fn canonical(token: &str) -> String {
    match token
        .trim_matches(|character: char| !character.is_ascii_alphanumeric() && character != '_')
    {
        "reducers" | "reduce" => "reducer".to_owned(),
        "map" | "maps" | "mapping" => "mapper".to_owned(),
        "filters" | "filtering" => "filter".to_owned(),
        "recursive" => "recursion".to_owned(),
        "normalization" | "normalizes" => "normalize".to_owned(),
        other => other.to_owned(),
    }
}

pub fn search(
    source: &str,
    report: &IntentReport,
    expression: &QueryExpression,
) -> Vec<QueryMatch> {
    let mut matches = report
        .function_reports
        .iter()
        .filter_map(|function| {
            evaluate(expression, function).map(|evaluation| QueryMatch {
                source: source.to_owned(),
                function_name: function.function_name.clone(),
                primary_intent: function.primary_intent.clone(),
                semantic_pipeline: function
                    .semantic_pipeline
                    .iter()
                    .map(|stage| stage.hypothesis_id.clone())
                    .collect(),
                score: evaluation.score,
                matched_fields: evaluation.fields.into_iter().collect(),
            })
        })
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.source.cmp(&right.source))
            .then_with(|| left.function_name.cmp(&right.function_name))
    });
    matches
}

#[derive(Default)]
struct Evaluation {
    score: u16,
    fields: BTreeSet<QueryMatchField>,
}

fn evaluate(expression: &QueryExpression, function: &FunctionIntentReport) -> Option<Evaluation> {
    match expression {
        QueryExpression::Term(term) => evaluate_term(term, function),
        QueryExpression::Not(inner) => evaluate(inner, function).is_none().then_some(Evaluation {
            score: 1,
            fields: BTreeSet::new(),
        }),
        QueryExpression::And(left, right) => {
            combine(evaluate(left, function)?, evaluate(right, function)?)
        }
        QueryExpression::Or(left, right) => {
            match (evaluate(left, function), evaluate(right, function)) {
                (Some(left), Some(right)) => Some(best(left, right)),
                (Some(value), None) | (None, Some(value)) => Some(value),
                (None, None) => None,
            }
        }
        QueryExpression::Then(left, right) => evaluate_then(left, right, function),
    }
}

fn evaluate_then(
    left: &QueryExpression,
    right: &QueryExpression,
    function: &FunctionIntentReport,
) -> Option<Evaluation> {
    let left = pipeline_term(left)?;
    let right = pipeline_term(right)?;
    let stages = function
        .semantic_pipeline
        .iter()
        .map(|stage| stage.hypothesis_id.as_str())
        .collect::<Vec<_>>();
    let left_index = stages.iter().position(|stage| *stage == left)?;
    let right_index = stages
        .iter()
        .skip(left_index + 1)
        .position(|stage| *stage == right)?;
    let mut fields = BTreeSet::new();
    fields.insert(QueryMatchField::PipelineStage {
        id: left.to_owned(),
    });
    fields.insert(QueryMatchField::PipelineStage {
        id: right.to_owned(),
    });
    Some(Evaluation {
        score: 180 + right_index as u16,
        fields,
    })
}

fn pipeline_term(expression: &QueryExpression) -> Option<&str> {
    match expression {
        QueryExpression::Term(term) => Some(term),
        _ => None,
    }
}

fn evaluate_term(term: &str, function: &FunctionIntentReport) -> Option<Evaluation> {
    let mut result = Evaluation::default();
    if function.primary_intent.as_deref() == Some(term) {
        result.score += 100;
        result.fields.insert(QueryMatchField::PrimaryIntent {
            id: term.to_owned(),
        });
    }
    for stage in &function.semantic_pipeline {
        if stage.hypothesis_id == term {
            result.score += 80;
            result.fields.insert(QueryMatchField::PipelineStage {
                id: term.to_owned(),
            });
        }
    }
    for hypothesis in &function.hypotheses {
        if hypothesis.id == term
            && hypothesis.status == HypothesisStatus::Active
            && hypothesis.confidence >= QUERY_HYPOTHESIS_CONFIDENCE_FLOOR
        {
            result.score += 60;
            result.fields.insert(QueryMatchField::Hypothesis {
                id: term.to_owned(),
            });
        }
    }
    for evidence in &function.evidence {
        if evidence_term(&evidence.kind) == term {
            result.score += 40;
            result.fields.insert(QueryMatchField::Evidence {
                kind: term.to_owned(),
            });
        }
    }
    for token in function
        .function_name
        .split(|character: char| !character.is_ascii_alphanumeric())
        .map(|token| token.to_ascii_lowercase())
    {
        if token == term {
            result.score += 20;
            result
                .fields
                .insert(QueryMatchField::FunctionMetadata { token });
        }
    }
    (result.score > 0).then_some(result)
}

fn evidence_term(kind: &EvidenceKind) -> String {
    match kind {
        EvidenceKind::NameToken { token } => token.clone(),
        other => format!("{other:?}").to_ascii_lowercase(),
    }
}

fn combine(mut left: Evaluation, right: Evaluation) -> Option<Evaluation> {
    left.score = left.score.saturating_add(right.score);
    left.fields.extend(right.fields);
    Some(left)
}

fn best(left: Evaluation, right: Evaluation) -> Evaluation {
    if left.score >= right.score {
        left
    } else {
        right
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{HypothesisReport, SemanticStage};

    fn report() -> IntentReport {
        IntentReport {
            version: 1,
            function_reports: vec![FunctionIntentReport {
                function_name: "filter_map_reduce".to_owned(),
                evidence: Vec::new(),
                hypotheses: ["filter", "mapper", "reducer"]
                    .into_iter()
                    .map(|id| HypothesisReport {
                        id: id.to_owned(),
                        confidence: 90,
                        status: HypothesisStatus::Active,
                        supporting_evidence: Vec::new(),
                        rejected_evidence: Vec::new(),
                    })
                    .collect(),
                primary_intent: Some("filter".to_owned()),
                semantic_pipeline: ["filter", "mapper", "reducer"]
                    .into_iter()
                    .map(|id| SemanticStage {
                        hypothesis_id: id.to_owned(),
                        confidence: 90,
                    })
                    .collect(),
            }],
        }
    }

    #[test]
    fn parses_boolean_and_pipeline_operators() {
        assert!(matches!(
            parse("filter THEN reduce").unwrap(),
            QueryExpression::Then(_, _)
        ));
        assert!(matches!(
            parse("maximum OR minimum").unwrap(),
            QueryExpression::Or(_, _)
        ));
        assert!(matches!(
            parse("NOT recursive").unwrap(),
            QueryExpression::Not(_)
        ));
        assert_eq!(
            parse("filter THEN map").unwrap(),
            QueryExpression::Then(
                Box::new(QueryExpression::Term("filter".to_owned())),
                Box::new(QueryExpression::Term("mapper".to_owned()))
            )
        );
        assert_eq!(
            parse("max by").unwrap(),
            QueryExpression::Term("max_by".to_owned())
        );
    }

    #[test]
    fn pipeline_queries_match_in_declared_order() {
        let query = parse("filter THEN reduce").unwrap();
        let matches = search("fixture", &report(), &query);
        assert_eq!(matches.len(), 1);
        assert_eq!(
            matches[0].semantic_pipeline,
            ["filter", "mapper", "reducer"]
        );
    }

    #[test]
    fn weak_alternative_hypotheses_do_not_pollute_repository_queries() {
        let function = FunctionIntentReport {
            function_name: "calculate".to_owned(),
            evidence: Vec::new(),
            hypotheses: vec![HypothesisReport {
                id: "histogram".to_owned(),
                confidence: QUERY_HYPOTHESIS_CONFIDENCE_FLOOR - 1,
                status: HypothesisStatus::Active,
                supporting_evidence: Vec::new(),
                rejected_evidence: Vec::new(),
            }],
            primary_intent: None,
            semantic_pipeline: Vec::new(),
        };
        let report = IntentReport {
            version: 1,
            function_reports: vec![function],
        };

        assert!(search("fixture", &report, &parse("histogram").unwrap()).is_empty());
    }
}
