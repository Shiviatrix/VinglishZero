use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(pub u32);
impl NodeId {
    pub const ROOT: Self = Self(0);
    pub fn new(value: u32) -> Self {
        Self(value)
    }
}
impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "node#{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceId(pub u32);
impl SourceId {
    pub const UNKNOWN: Self = Self(0);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanguageTag {
    Vinglish,
    Python,
    Rust,
    Java,
    C,
    JavaScript,
    TypeScript,
    Custom(String),
    Unknown,
}
impl fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Vinglish => "vinglish",
                Self::Python => "python",
                Self::Rust => "rust",
                Self::Java => "java",
                Self::C => "c",
                Self::JavaScript => "javascript",
                Self::TypeScript => "typescript",
                Self::Custom(value) => value,
                Self::Unknown => "unknown",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemanticSpan {
    pub source_id: SourceId,
    pub start: u32,
    pub end: u32,
    pub language: LanguageTag,
}
impl SemanticSpan {
    pub fn new(source_id: SourceId, start: u32, end: u32, language: LanguageTag) -> Self {
        Self {
            source_id,
            start,
            end,
            language,
        }
    }
    pub fn dummy() -> Self {
        Self::new(SourceId::UNKNOWN, 0, 0, LanguageTag::Unknown)
    }
    pub fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mutability {
    Immutable,
    Mutable,
    Untracked,
}
impl fmt::Display for Mutability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Immutable => "immutable",
                Self::Mutable => "mutable",
                Self::Untracked => "untracked",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Restricted(String),
    Untracked,
}

#[derive(Debug, Default)]
pub struct NodeIdAllocator {
    next: u32,
}
impl NodeIdAllocator {
    pub fn new() -> Self {
        Self { next: 1 }
    }
    /// Allocates the next graph-local identifier.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> NodeId {
        let id = NodeId(self.next);
        self.next += 1;
        id
    }
}
