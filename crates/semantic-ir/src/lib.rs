//! Shared language-agnostic semantic representation.

pub mod graph;
pub mod metadata;
pub mod node;
pub mod types;

pub use graph::{GraphValidationError, SemanticGraph};
pub use metadata::Metadata;
pub use node::{
    AllocKind, AsyncKind, CollectionKind, LoopKind, SemanticNode, SemanticOp, SemanticUnOp,
};
pub use types::TypeConcept;
pub use vz_common::{LanguageTag, Mutability, NodeId, SemanticSpan, SourceId, Visibility};
