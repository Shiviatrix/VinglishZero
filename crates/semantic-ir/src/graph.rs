//! Semantic graph storage.
//!
//! The graph owns semantic nodes and keeps the IR detached from any specific
//! parser, AST, or lowering pipeline.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use vz_common::NodeId;

use crate::{Metadata, SemanticNode};

/// A collection of semantic nodes indexed by stable node identifiers.
#[derive(Debug, Clone, Default)]
pub struct SemanticGraph {
    nodes: BTreeMap<NodeId, SemanticNode>,
    duplicate_ids: BTreeSet<NodeId>,
    metadata: Metadata,
}

impl SemanticGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_metadata(metadata: Metadata) -> Self {
        Self {
            nodes: BTreeMap::new(),
            duplicate_ids: BTreeSet::new(),
            metadata,
        }
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    /// Inserts a node while retaining duplicate-insertion evidence for
    /// [`Self::validate`]. This compatibility API preserves existing producer
    /// call sites; new producers should use [`Self::try_insert`] when they can
    /// report the error at its origin.
    pub fn insert(&mut self, node: SemanticNode) -> NodeId {
        let id = node.id();
        if self.nodes.insert(id, node).is_some() {
            self.duplicate_ids.insert(id);
        }
        id
    }

    /// Inserts a node only when its graph-local identifier is unused.
    ///
    /// New producers should prefer this method over [`Self::insert`] so a
    /// malformed lowering is rejected at the insertion site.
    pub fn try_insert(&mut self, node: SemanticNode) -> Result<NodeId, GraphValidationError> {
        let id = node.id();
        if self.nodes.contains_key(&id) {
            return Err(GraphValidationError::DuplicateNodeId { id });
        }
        self.nodes.insert(id, node);
        Ok(id)
    }

    pub fn get(&self, id: NodeId) -> Option<&SemanticNode> {
        self.nodes.get(&id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut SemanticNode> {
        self.nodes.get_mut(&id)
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&NodeId, &SemanticNode)> {
        self.nodes.iter()
    }

    /// Verifies graph-local identifiers and every referenced node.
    ///
    /// The graph intentionally permits multiple parents and cycles, because
    /// semantic ownership and control flow are not trees. Validation only
    /// rejects malformed relationships that would make downstream consumers
    /// observe incomplete semantic state.
    pub fn validate(&self) -> Result<(), GraphValidationError> {
        if let Some(id) = self.duplicate_ids.iter().next() {
            return Err(GraphValidationError::DuplicateNodeId { id: *id });
        }
        for (stored_id, node) in &self.nodes {
            if node.id() != *stored_id {
                return Err(GraphValidationError::StoredNodeIdMismatch {
                    stored: *stored_id,
                    declared: node.id(),
                });
            }
            for referenced in node.referenced_ids() {
                if !self.nodes.contains_key(&referenced) {
                    return Err(GraphValidationError::DanglingReference {
                        from: *stored_id,
                        to: referenced,
                    });
                }
            }
        }
        Ok(())
    }
}

/// A deterministic semantic graph integrity failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphValidationError {
    DuplicateNodeId { id: NodeId },
    StoredNodeIdMismatch { stored: NodeId, declared: NodeId },
    DanglingReference { from: NodeId, to: NodeId },
}

impl fmt::Display for GraphValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateNodeId { id } => {
                write!(formatter, "duplicate semantic node identifier {id}")
            }
            Self::StoredNodeIdMismatch { stored, declared } => write!(
                formatter,
                "semantic graph stored {stored} under a node declaring {declared}"
            ),
            Self::DanglingReference { from, to } => {
                write!(
                    formatter,
                    "semantic node {from} references missing node {to}"
                )
            }
        }
    }
}

impl std::error::Error for GraphValidationError {}

#[cfg(test)]
mod tests {
    use vz_common::{LanguageTag, NodeId, SemanticSpan};

    use crate::{Metadata, SemanticNode};

    use super::{GraphValidationError, SemanticGraph};

    #[test]
    fn checked_insertion_rejects_duplicate_identifiers() {
        let mut graph = SemanticGraph::new();
        let node = SemanticNode::Import {
            id: NodeId::new(1),
            path: vec!["example".to_owned()],
            alias: None,
            kind: crate::node::ImportKind::Named,
            span: SemanticSpan::dummy(),
            metadata: Metadata::default(),
        };
        graph.try_insert(node.clone()).unwrap();
        assert_eq!(
            graph.try_insert(node),
            Err(GraphValidationError::DuplicateNodeId { id: NodeId::new(1) })
        );
    }

    #[test]
    fn validation_detects_legacy_duplicate_insertions() {
        let mut graph = SemanticGraph::new();
        let node = SemanticNode::Import {
            id: NodeId::new(1),
            path: vec!["example".to_owned()],
            alias: None,
            kind: crate::node::ImportKind::Named,
            span: SemanticSpan::dummy(),
            metadata: Metadata::default(),
        };
        graph.insert(node.clone());
        graph.insert(node);

        assert_eq!(
            graph.validate(),
            Err(GraphValidationError::DuplicateNodeId { id: NodeId::new(1) })
        );
    }

    #[test]
    fn validation_rejects_dangling_references() {
        let mut graph = SemanticGraph::new();
        graph.insert(SemanticNode::Module {
            id: NodeId::new(1),
            name: "example".to_owned(),
            language: LanguageTag::Unknown,
            children: vec![NodeId::new(2)],
            span: SemanticSpan::dummy(),
            metadata: Metadata::default(),
        });
        assert_eq!(
            graph.validate(),
            Err(GraphValidationError::DanglingReference {
                from: NodeId::new(1),
                to: NodeId::new(2),
            })
        );
    }
}
