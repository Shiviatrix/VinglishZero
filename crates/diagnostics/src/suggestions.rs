use crate::{
    matcher::SuggestionSet,
    model::{ApplicableEvidence, SuggestedFix, SuggestionTitle},
};

pub fn generate(
    set: SuggestionSet,
    confidence: u8,
    evidence: &[ApplicableEvidence],
) -> Vec<SuggestedFix> {
    let make = |title, explanation: &str, rationale: &str| SuggestedFix {
        title,
        explanation: explanation.to_owned(),
        rationale: rationale.to_owned(),
        confidence,
        applicable_evidence: evidence.to_vec(),
    };
    match set {
        SuggestionSet::AccumulatorType => vec![
            make(SuggestionTitle::ChangeAccumulatorType, "Use an accumulator type compatible with incoming values.", "The inferred accumulator performs addition inside iteration."),
            make(SuggestionTitle::ConvertIncomingValue, "Convert each incoming value before accumulation.", "Conversion can make the addition operands compatible."),
            make(SuggestionTitle::UseCompatibleAddition, "Use an addition operation defined for both operand types.", "The compiler reported an incompatible addition in the inferred accumulation flow."),
        ],
        SuggestionSet::AverageDivision => vec![make(SuggestionTitle::AddDivision, "Divide the accumulated value by the item count.", "Average intent requires a final division after accumulation.")],
        SuggestionSet::Ownership => vec![
            make(SuggestionTitle::RestoreOwnership, "Retain or transfer ownership before the later use.", "The inferred resource-management intent conflicts with a move-after-use diagnostic."),
            make(SuggestionTitle::AvoidUseAfterMove, "Avoid accessing the resource after it has moved.", "The diagnostic identifies a use after ownership transfer."),
        ],
    }
}
