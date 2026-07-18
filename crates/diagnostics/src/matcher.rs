use crate::model::{DiagnosticCategory, Interpretation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionSet {
    AccumulatorType,
    AverageDivision,
    Ownership,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticRule {
    pub id: &'static str,
    pub category: DiagnosticCategory,
    pub hypothesis_id: &'static str,
    pub interpretation: Interpretation,
    pub suggestion_set: SuggestionSet,
}

#[derive(Debug, Default)]
pub struct DiagnosticRuleRegistry {
    rules: Vec<DiagnosticRule>,
}
impl DiagnosticRuleRegistry {
    pub fn register(&mut self, rule: DiagnosticRule) {
        self.rules.push(rule);
        self.rules.sort_by(|left, right| left.id.cmp(right.id));
    }
    pub fn matching(
        &self,
        category: &DiagnosticCategory,
        hypothesis_id: &str,
    ) -> Vec<&DiagnosticRule> {
        self.rules
            .iter()
            .filter(|rule| &rule.category == category && rule.hypothesis_id == hypothesis_id)
            .collect()
    }
}
pub fn default_rules() -> DiagnosticRuleRegistry {
    let mut rules = DiagnosticRuleRegistry::default();
    rules.register(DiagnosticRule {
        id: "accumulator-type-mismatch",
        category: DiagnosticCategory::TypeMismatch,
        hypothesis_id: "accumulator",
        interpretation: Interpretation::AccumulatorTypeConflict,
        suggestion_set: SuggestionSet::AccumulatorType,
    });
    rules.register(DiagnosticRule {
        id: "average-missing-division",
        category: DiagnosticCategory::MissingDivision,
        hypothesis_id: "average",
        interpretation: Interpretation::AverageImplementationIncomplete,
        suggestion_set: SuggestionSet::AverageDivision,
    });
    rules.register(DiagnosticRule {
        id: "resource-move-after-use",
        category: DiagnosticCategory::MoveAfterUse,
        hypothesis_id: "resource_manager",
        interpretation: Interpretation::OwnershipViolation,
        suggestion_set: SuggestionSet::Ownership,
    });
    rules
}
