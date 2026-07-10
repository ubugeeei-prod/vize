use vize_carton::{String, directive::DirectiveKind};

use crate::{CommentKind, ElementType, Namespace, NodeType, SourceLocation};

use super::{
    ReliefSnapshotNodeId, SnapshotCompoundExpression, SnapshotExpression, SnapshotForParseResult,
    SnapshotProp,
};

/// Exact Relief template-child variant retained by a snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReliefSnapshotNodeKind {
    Element,
    Text,
    Comment,
    Interpolation,
    If,
    IfBranch,
    For,
    TextCall,
    CompoundExpression,
    Hoisted,
}

/// Owned node in a [`crate::ReliefSnapshot`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReliefSnapshotNode {
    Element(SnapshotElement),
    Text(SnapshotText),
    Comment(SnapshotComment),
    Interpolation(SnapshotInterpolation),
    If(SnapshotIf),
    IfBranch(SnapshotIfBranch),
    For(Box<SnapshotFor>),
    TextCall(SnapshotTextCall),
    CompoundExpression(SnapshotCompoundExpression),
    Hoisted(SnapshotHoisted),
}

impl ReliefSnapshotNode {
    /// Exact template-child variant, including Relief's hoisted reference.
    pub const fn kind(&self) -> ReliefSnapshotNodeKind {
        match self {
            Self::Element(_) => ReliefSnapshotNodeKind::Element,
            Self::Text(_) => ReliefSnapshotNodeKind::Text,
            Self::Comment(_) => ReliefSnapshotNodeKind::Comment,
            Self::Interpolation(_) => ReliefSnapshotNodeKind::Interpolation,
            Self::If(_) => ReliefSnapshotNodeKind::If,
            Self::IfBranch(_) => ReliefSnapshotNodeKind::IfBranch,
            Self::For(_) => ReliefSnapshotNodeKind::For,
            Self::TextCall(_) => ReliefSnapshotNodeKind::TextCall,
            Self::CompoundExpression(_) => ReliefSnapshotNodeKind::CompoundExpression,
            Self::Hoisted(_) => ReliefSnapshotNodeKind::Hoisted,
        }
    }

    /// Relief's original `NodeType` discriminant.
    pub const fn node_type(&self) -> NodeType {
        match self {
            Self::Element(_) => NodeType::Element,
            Self::Text(_) => NodeType::Text,
            Self::Comment(_) => NodeType::Comment,
            Self::Interpolation(_) => NodeType::Interpolation,
            Self::If(_) => NodeType::If,
            Self::IfBranch(_) => NodeType::IfBranch,
            Self::For(_) => NodeType::For,
            Self::TextCall(_) => NodeType::TextCall,
            Self::CompoundExpression(_) => NodeType::CompoundExpression,
            // This mirrors `TemplateChildNode::node_type`.
            Self::Hoisted(_) => NodeType::SimpleExpression,
        }
    }

    /// Source span retained by the original Relief node.
    pub const fn location(&self) -> &SourceLocation {
        match self {
            Self::Element(node) => &node.location,
            Self::Text(node) => &node.location,
            Self::Comment(node) => &node.location,
            Self::Interpolation(node) => &node.location,
            Self::If(node) => &node.location,
            Self::IfBranch(node) => &node.location,
            Self::For(node) => &node.location,
            Self::TextCall(node) => &node.location,
            Self::CompoundExpression(node) => &node.location,
            Self::Hoisted(node) => &node.location,
        }
    }

    /// Direct syntax children in original source order.
    pub fn children(&self) -> &[ReliefSnapshotNodeId] {
        match self {
            Self::Element(node) => &node.children,
            Self::If(node) => &node.branches,
            Self::IfBranch(node) => &node.children,
            Self::For(node) => &node.children,
            Self::Text(_)
            | Self::Comment(_)
            | Self::Interpolation(_)
            | Self::TextCall(_)
            | Self::CompoundExpression(_)
            | Self::Hoisted(_) => &[],
        }
    }
}

/// Owned Relief element syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotElement {
    pub namespace: Namespace,
    pub tag: String,
    pub tag_type: ElementType,
    pub props: Vec<SnapshotProp>,
    pub(crate) children: Vec<ReliefSnapshotNodeId>,
    pub is_self_closing: bool,
    pub location: SourceLocation,
    pub inner_location: Option<SourceLocation>,
    pub hoisted_props_index: Option<usize>,
}

impl SnapshotElement {
    pub fn children(&self) -> &[ReliefSnapshotNodeId] {
        &self.children
    }
}

/// Owned text syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotText {
    pub content: String,
    pub location: SourceLocation,
}

/// Owned comment syntax, including comments written inside a start tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotComment {
    pub content: String,
    pub location: SourceLocation,
    pub kind: CommentKind,
    pub directive: Option<DirectiveKind>,
}

/// Owned interpolation syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotInterpolation {
    pub content: SnapshotExpression,
    pub location: SourceLocation,
    #[cfg(feature = "_legacy")]
    pub raw: bool,
}

/// Owned `v-if` node whose branch IDs retain branch order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotIf {
    pub(crate) branches: Vec<ReliefSnapshotNodeId>,
    pub location: SourceLocation,
}

impl SnapshotIf {
    pub fn branches(&self) -> &[ReliefSnapshotNodeId] {
        &self.branches
    }
}

/// Owned `v-if`/`v-else-if`/`v-else` branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotIfBranch {
    pub condition: Option<SnapshotExpression>,
    pub(crate) children: Vec<ReliefSnapshotNodeId>,
    pub user_key: Option<SnapshotProp>,
    pub is_template_if: bool,
    pub location: SourceLocation,
}

impl SnapshotIfBranch {
    pub fn children(&self) -> &[ReliefSnapshotNodeId] {
        &self.children
    }
}

/// Owned `v-for` syntax and body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotFor {
    pub source: SnapshotExpression,
    pub value_alias: Option<SnapshotExpression>,
    pub key_alias: Option<SnapshotExpression>,
    pub object_index_alias: Option<SnapshotExpression>,
    pub parse_result: SnapshotForParseResult,
    pub(crate) children: Vec<ReliefSnapshotNodeId>,
    pub location: SourceLocation,
}

impl SnapshotFor {
    pub fn children(&self) -> &[ReliefSnapshotNodeId] {
        &self.children
    }
}

/// Owned transformed text-call syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotTextCall {
    pub content: SnapshotTextCallContent,
    pub location: SourceLocation,
}

/// Original content carried by a Relief text-call node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotTextCallContent {
    Text(SnapshotText),
    Interpolation(SnapshotInterpolation),
    Compound(SnapshotCompoundExpression),
}

/// Owned Relief hoist reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotHoisted {
    pub index: usize,
    pub location: SourceLocation,
}
