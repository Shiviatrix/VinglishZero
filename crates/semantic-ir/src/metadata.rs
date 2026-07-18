//! Metadata attached to semantic concepts.
//!
//! This is intentionally a light wrapper around a string-keyed map so adapters
//! can attach language-specific annotations without changing the shared IR.

use std::collections::BTreeMap;

/// A small, dependency-free value model for metadata and extension payloads.
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    Text(String),
    List(Vec<SemanticValue>),
    Map(BTreeMap<String, SemanticValue>),
}

impl From<&str> for SemanticValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<String> for SemanticValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<bool> for SemanticValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for SemanticValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for SemanticValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

/// Arbitrary metadata attached to a semantic node or relationship.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Metadata(pub BTreeMap<String, SemanticValue>);

impl Metadata {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<SemanticValue>) {
        self.0.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&SemanticValue> {
        self.0.get(key)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
