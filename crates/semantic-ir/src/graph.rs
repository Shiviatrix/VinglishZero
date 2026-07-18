//! Semantic graph storage.
//!
//! The graph owns semantic nodes and keeps the IR detached from any specific
//! parser, AST, or lowering pipeline.

use std::collections::BTreeMap;

use vz_common::NodeId;

use crate::{Metadata, SemanticNode};

/// A collection of semantic nodes indexed by stable node identifiers.
#[derive(Debug, Clone, Default)]
pub struct SemanticGraph {
    nodes: BTreeMap<NodeId, SemanticNode>,
    metadata: Metadata,
}

impl SemanticGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_metadata(metadata: Metadata) -> Self {
        Self {
            nodes: BTreeMap::new(),
            metadata,
        }
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    pub fn insert(&mut self, node: SemanticNode) -> NodeId {
        let id = node.id();
        self.nodes.insert(id, node);
        id
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
}
