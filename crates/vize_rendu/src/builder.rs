//! Builder for a validated owned Rendu root.

use crate::{
    RenduCapabilities, RenduExpression, RenduExpressionId, RenduNode, RenduNodeId, RenduRoot,
    RenduSource, RenduSourceId, RenduValidationErrors,
};

/// Producer-facing arena builder.
///
/// Frontends can add items in any order, including forward references. `finish`
/// validates every edge and source span before exposing a root to consumers.
#[derive(Debug, Clone, Default)]
pub struct RenduBuilder {
    sources: Vec<RenduSource>,
    expressions: Vec<RenduExpression>,
    nodes: Vec<RenduNode>,
    entry: Vec<RenduNodeId>,
}

impl RenduBuilder {
    pub const fn new() -> Self {
        Self {
            sources: Vec::new(),
            expressions: Vec::new(),
            nodes: Vec::new(),
            entry: Vec::new(),
        }
    }

    pub fn add_source(&mut self, source: RenduSource) -> RenduSourceId {
        let id = RenduSourceId::from_index(self.sources.len());
        self.sources.push(source);
        id
    }

    pub fn add_expression(&mut self, expression: RenduExpression) -> RenduExpressionId {
        let id = RenduExpressionId::from_index(self.expressions.len());
        self.expressions.push(expression);
        id
    }

    pub fn add_node(&mut self, node: RenduNode) -> RenduNodeId {
        let id = RenduNodeId::from_index(self.nodes.len());
        self.nodes.push(node);
        id
    }

    pub fn push_entry(&mut self, node: RenduNodeId) {
        self.entry.push(node);
    }

    pub fn set_entry(&mut self, nodes: impl IntoIterator<Item = RenduNodeId>) {
        self.entry.clear();
        self.entry.extend(nodes);
    }

    pub fn finish(self) -> Result<RenduRoot, RenduValidationErrors> {
        let mut root = RenduRoot {
            sources: self.sources,
            expressions: self.expressions,
            nodes: self.nodes,
            entry: self.entry,
            capabilities: RenduCapabilities::empty(),
            component_scope_id: None,
        };
        root.validate()?;
        root.capabilities = RenduCapabilities::infer(&root);
        Ok(root)
    }
}
