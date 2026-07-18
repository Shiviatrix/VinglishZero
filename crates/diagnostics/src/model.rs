use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Hint,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiagnosticCategory {
    TypeMismatch,
    MissingDivision,
    MoveAfterUse,
    OwnershipTransfer,
    Unsupported,
    Other { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub source: String,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerDiagnostic {
    pub code: String,
    pub severity: Severity,
    pub category: DiagnosticCategory,
    pub message: String,
    pub location: Option<SourceLocation>,
    pub function_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ApplicableEvidence {
    pub provider_id: String,
    pub kind: EvidenceKind,
    pub strength: u32,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentReference {
    pub function_name: String,
    pub hypothesis_id: String,
    pub confidence: u8,
    pub evidence: Vec<ApplicableEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interpretation {
    NoMatchingIntentRule,
    AccumulatorTypeConflict,
    AverageImplementationIncomplete,
    OwnershipViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionTitle {
    ChangeAccumulatorType,
    ConvertIncomingValue,
    UseCompatibleAddition,
    AddDivision,
    RestoreOwnership,
    AvoidUseAfterMove,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuggestedFix {
    pub title: SuggestionTitle,
    pub explanation: String,
    pub rationale: String,
    pub confidence: u8,
    pub applicable_evidence: Vec<ApplicableEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticDiagnostic {
    pub version: u32,
    pub compiler_issue: CompilerDiagnostic,
    pub intent: Option<IntentReference>,
    pub interpretation: Interpretation,
    pub suggestions: Vec<SuggestedFix>,
}
