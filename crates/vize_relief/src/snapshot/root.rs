use vize_carton::String;

use crate::{NodeType, RuntimeHelper, SourceLocation};

use super::{
    ReliefSnapshotNode, ReliefSnapshotNodeId, ReliefSnapshotWalker, SnapshotComment,
    SnapshotSimpleExpression,
};

/// Owned import retained from a Relief root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotImport {
    pub expression: SnapshotSimpleExpression,
    pub path: String,
}

/// Owned, source-faithful cache product copied from an arena Relief root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReliefSnapshot {
    pub(crate) source: String,
    pub(crate) location: SourceLocation,
    pub(crate) nodes: Vec<ReliefSnapshotNode>,
    pub(crate) children: Vec<ReliefSnapshotNodeId>,
    pub(crate) comments: Vec<SnapshotComment>,
    pub(crate) helpers: Vec<RuntimeHelper>,
    pub(crate) components: Vec<String>,
    pub(crate) directives: Vec<String>,
    #[cfg(feature = "_legacy")]
    pub(crate) filters: Vec<String>,
    pub(crate) imports: Vec<SnapshotImport>,
    pub(crate) temps: u32,
    pub(crate) transformed: bool,
}

impl ReliefSnapshot {
    /// Relief root discriminant.
    pub const fn node_type(&self) -> NodeType {
        NodeType::Root
    }

    /// Complete template source retained by the Relief root.
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// Root source location.
    pub const fn location(&self) -> &SourceLocation {
        &self.location
    }

    /// Top-level syntax nodes in original source order.
    pub fn children(&self) -> &[ReliefSnapshotNodeId] {
        &self.children
    }

    /// All nodes in stable pre-order allocation order.
    pub fn nodes(&self) -> &[ReliefSnapshotNode] {
        &self.nodes
    }

    /// Look up a node by its snapshot-local ID.
    pub fn node(&self, id: ReliefSnapshotNodeId) -> Option<&ReliefSnapshotNode> {
        self.nodes.get(id.index())
    }

    /// Direct children of a root or node. Unknown IDs return `None`.
    pub fn children_of(
        &self,
        parent: Option<ReliefSnapshotNodeId>,
    ) -> Option<&[ReliefSnapshotNodeId]> {
        match parent {
            None => Some(&self.children),
            Some(id) => self.node(id).map(ReliefSnapshotNode::children),
        }
    }

    /// Comments collected separately by the original Relief root.
    ///
    /// A comment may also be present as a syntax child; both original
    /// collections are retained without deduplication.
    pub fn comments(&self) -> &[SnapshotComment] {
        &self.comments
    }

    pub fn helpers(&self) -> &[RuntimeHelper] {
        &self.helpers
    }

    pub fn components(&self) -> &[String] {
        &self.components
    }

    pub fn directives(&self) -> &[String] {
        &self.directives
    }

    #[cfg(feature = "_legacy")]
    pub fn filters(&self) -> &[String] {
        &self.filters
    }

    pub fn imports(&self) -> &[SnapshotImport] {
        &self.imports
    }

    pub const fn temps(&self) -> u32 {
        self.temps
    }

    pub const fn transformed(&self) -> bool {
        self.transformed
    }

    /// Depth-first source-order traversal from the root.
    pub fn walk(&self) -> ReliefSnapshotWalker<'_> {
        ReliefSnapshotWalker::from_roots(self, &self.children)
    }

    /// Depth-first traversal rooted at one node.
    pub fn walk_from(&self, root: ReliefSnapshotNodeId) -> ReliefSnapshotWalker<'_> {
        if self.node(root).is_some() {
            ReliefSnapshotWalker::from_one(self, root)
        } else {
            ReliefSnapshotWalker::empty(self)
        }
    }

    /// Nodes whose half-open source ranges contain `offset`, in traversal order.
    pub fn nodes_at_offset(
        &self,
        offset: u32,
    ) -> impl Iterator<Item = (ReliefSnapshotNodeId, &ReliefSnapshotNode)> {
        self.walk().filter_map(move |visit| {
            let location = visit.node.location();
            (location.start.offset <= offset && offset < location.end.offset)
                .then_some((visit.id, visit.node))
        })
    }

    /// Smallest source-backed node containing `offset`.
    pub fn node_at_offset(
        &self,
        offset: u32,
    ) -> Option<(ReliefSnapshotNodeId, &ReliefSnapshotNode)> {
        self.nodes_at_offset(offset).min_by_key(|(_, node)| {
            node.location()
                .end
                .offset
                .saturating_sub(node.location().start.offset)
        })
    }
}
