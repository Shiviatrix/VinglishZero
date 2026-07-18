//! Structured diagnostics for semantic reasoning.
//!
//! Diagnostics describe what the semantic engine discovered without binding
//! the result to any one language front end.

use vz_common::{SemanticSpan, SourceId};

pub mod engine;
pub mod matcher;
pub mod model;
pub mod suggestions;

pub use engine::{DiagnosticEngineError, SemanticDiagnosticEngine};
pub use model::{CompilerDiagnostic, DiagnosticCategory, SemanticDiagnostic, Severity};

/// A single semantic finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: String,
    pub severity: model::Severity,
    pub message: String,
    pub source_id: Option<SourceId>,
    pub span: Option<SemanticSpan>,
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn new(
        code: impl Into<String>,
        severity: model::Severity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            source_id: None,
            span: None,
            notes: Vec::new(),
        }
    }
}
