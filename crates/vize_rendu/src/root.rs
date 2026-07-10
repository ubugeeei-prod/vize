//! Validated, owned Rendu compilation product.

use crate::{
    RenduCapabilities, RenduExpression, RenduExpressionId, RenduNode, RenduNodeId, RenduSource,
    RenduSourceId,
};

/// An owned Rendu HIR with typed source, expression, and node arenas.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenduRoot {
    pub(crate) sources: Vec<RenduSource>,
    pub(crate) expressions: Vec<RenduExpression>,
    pub(crate) nodes: Vec<RenduNode>,
    pub(crate) entry: Vec<RenduNodeId>,
    pub(crate) capabilities: RenduCapabilities,
}

impl RenduRoot {
    pub fn sources(&self) -> &[RenduSource] {
        &self.sources
    }

    pub fn expressions(&self) -> &[RenduExpression] {
        &self.expressions
    }

    pub fn nodes(&self) -> &[RenduNode] {
        &self.nodes
    }

    pub fn entry(&self) -> &[RenduNodeId] {
        &self.entry
    }

    pub const fn capabilities(&self) -> RenduCapabilities {
        self.capabilities
    }

    pub fn source(&self, id: RenduSourceId) -> Option<&RenduSource> {
        self.sources.get(id.index())
    }

    pub fn expression(&self, id: RenduExpressionId) -> Option<&RenduExpression> {
        self.expressions.get(id.index())
    }

    pub fn node(&self, id: RenduNodeId) -> Option<&RenduNode> {
        self.nodes.get(id.index())
    }
}
