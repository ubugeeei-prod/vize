//! Borrowed render-semantic Rendu vocabulary.

use super::model::{RenduElementKind, RenduExprRef, RenduModifiers, RenduSource, RenduSpan};
use vize_atlas::{SourceAtlasCoordinate, SourceAtlasTarget, SourceAtlasTargetSet};
use vize_relief::{
    AttributeNode, DirectiveNode, ElementNode, ForNode, IfBranchNode, IfNode, PropNode,
    TemplateChildNode, TextCallContent, TextNode,
};

/// Render-semantic operation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenduOp<'a> {
    Element {
        tag: &'a str,
        kind: RenduElementKind,
        span: RenduSpan,
    },
    /// A static attribute on the nearest preceding `Element` op.
    ///
    /// `name_span`/`value_span` carry the source positions of the name and
    /// value separately so a backend can source-map the emitted prop key and
    /// value literal straight from the op.
    Attribute {
        name: &'a str,
        name_span: RenduSpan,
        value: Option<&'a str>,
        value_span: Option<RenduSpan>,
        span: RenduSpan,
    },
    /// A directive (`v-bind`/`v-on`/`v-model`/custom) on the nearest preceding
    /// `Element` op, with its modifiers (`.stop`, `.camel`, ...).
    Directive {
        name: &'a str,
        arg: Option<RenduExprRef<'a>>,
        exp: Option<RenduExprRef<'a>>,
        modifiers: RenduModifiers<'a>,
        span: RenduSpan,
    },
    Text {
        content: &'a str,
        span: RenduSpan,
    },
    Comment {
        content: &'a str,
        /// Whether this is a `@vize:` directive comment (stripped from output).
        is_directive: bool,
        span: RenduSpan,
    },
    Interpolation {
        expression: RenduExprRef<'a>,
        /// Raw (unescaped) interpolation — legacy `{{{ expr }}}`; always `false`
        /// for modern Vue.
        raw: bool,
        span: RenduSpan,
    },
    If {
        span: RenduSpan,
    },
    IfBranch {
        condition: Option<RenduExprRef<'a>>,
        span: RenduSpan,
    },
    For {
        source: RenduExprRef<'a>,
        /// `v-for="value in source"` value alias, if present.
        value: Option<RenduExprRef<'a>>,
        /// `v-for="(value, key) in source"` key alias, if present.
        key: Option<RenduExprRef<'a>>,
        /// `v-for="(value, key, index) in source"` index alias, if present.
        index: Option<RenduExprRef<'a>>,
        span: RenduSpan,
    },
    TextCall {
        content: Option<RenduExprRef<'a>>,
        span: RenduSpan,
    },
    CompoundExpression {
        expression: RenduExprRef<'a>,
        span: RenduSpan,
    },
    HoistRef {
        index: usize,
    },
}

impl<'a> RenduOp<'a> {
    /// Build the render op for an element prop (static attribute or directive).
    pub fn from_prop(prop: &'a PropNode<'a>) -> Self {
        match prop {
            PropNode::Attribute(attr) => Self::from_attribute(attr),
            PropNode::Directive(directive) => Self::from_directive(directive),
        }
    }

    /// Build the element render op for an element node (tag, kind, span).
    pub fn from_element(element: &'a ElementNode<'a>) -> Self {
        Self::Element {
            tag: element.tag.as_str(),
            kind: element.tag_type.into(),
            span: RenduSpan::from_location(&element.loc),
        }
    }

    /// Build the text render op for a text node (content, span).
    pub fn from_text(text: &'a TextNode) -> Self {
        Self::Text {
            content: text.content.as_str(),
            span: RenduSpan::from_location(&text.loc),
        }
    }

    pub fn from_if(node: &'a IfNode<'a>) -> Self {
        Self::If {
            span: RenduSpan::from_location(&node.loc),
        }
    }

    pub fn from_if_branch(branch: &'a IfBranchNode<'a>) -> Self {
        Self::IfBranch {
            condition: branch.condition.as_ref().map(RenduExprRef::from_expression),
            span: RenduSpan::from_location(&branch.loc),
        }
    }

    pub fn from_for(node: &'a ForNode<'a>) -> Self {
        Self::For {
            source: RenduExprRef::from_expression(&node.source),
            value: node.value_alias.as_ref().map(RenduExprRef::from_expression),
            key: node.key_alias.as_ref().map(RenduExprRef::from_expression),
            index: node
                .object_index_alias
                .as_ref()
                .map(RenduExprRef::from_expression),
            span: RenduSpan::from_location(&node.loc),
        }
    }

    fn from_attribute(attr: &'a AttributeNode) -> Self {
        Self::Attribute {
            name: attr.name.as_str(),
            name_span: RenduSpan::from_location(&attr.name_loc),
            value: attr.value.as_ref().map(|text| text.content.as_str()),
            value_span: attr
                .value
                .as_ref()
                .map(|text| RenduSpan::from_location(&text.loc)),
            span: RenduSpan::from_location(&attr.loc),
        }
    }

    pub fn from_directive(directive: &'a DirectiveNode<'a>) -> Self {
        Self::Directive {
            name: directive.name.as_str(),
            arg: directive.arg.as_ref().map(RenduExprRef::from_expression),
            exp: directive.exp.as_ref().map(RenduExprRef::from_expression),
            modifiers: RenduModifiers::new(directive.modifiers.as_slice()),
            span: RenduSpan::from_location(&directive.loc),
        }
    }

    pub fn from_template_child(child: &'a TemplateChildNode<'a>) -> Self {
        match child {
            TemplateChildNode::Element(element) => Self::Element {
                tag: element.tag.as_str(),
                kind: element.tag_type.into(),
                span: RenduSpan::from_location(&element.loc),
            },
            TemplateChildNode::Text(text) => Self::Text {
                content: text.content.as_str(),
                span: RenduSpan::from_location(&text.loc),
            },
            TemplateChildNode::Comment(comment) => Self::Comment {
                content: comment.content.as_str(),
                is_directive: comment.directive.is_some(),
                span: RenduSpan::from_location(&comment.loc),
            },
            TemplateChildNode::Interpolation(interpolation) => Self::Interpolation {
                expression: RenduExprRef::from_expression(&interpolation.content),
                #[cfg(feature = "legacy")]
                raw: interpolation.raw,
                #[cfg(not(feature = "legacy"))]
                raw: false,
                span: RenduSpan::from_location(&interpolation.loc),
            },
            TemplateChildNode::If(node) => Self::from_if(node),
            TemplateChildNode::IfBranch(branch) => Self::from_if_branch(branch),
            TemplateChildNode::For(node) => Self::from_for(node),
            TemplateChildNode::TextCall(node) => Self::TextCall {
                content: match &node.content {
                    TextCallContent::Interpolation(interpolation) => {
                        Some(RenduExprRef::from_expression(&interpolation.content))
                    }
                    TextCallContent::Compound(compound) => {
                        Some(RenduExprRef::Relief(compound.loc.source.as_str()))
                    }
                    TextCallContent::Text(_) => None,
                },
                span: RenduSpan::from_location(&node.loc),
            },
            TemplateChildNode::CompoundExpression(compound) => Self::CompoundExpression {
                expression: RenduExprRef::Relief(compound.loc.source.as_str()),
                span: RenduSpan::from_location(&compound.loc),
            },
            TemplateChildNode::Hoisted(index) => Self::HoistRef { index: *index },
        }
    }

    /// The source span this operation covers, if it carries one.
    ///
    /// Every operation except a [`HoistRef`](Self::HoistRef) — which only points
    /// at a module-level hoist index — carries a span, so a backend can emit
    /// source-mapped output straight from the Rendu op without reaching back
    /// into the Relief node.
    pub fn span(self) -> Option<RenduSpan> {
        match self {
            Self::Element { span, .. }
            | Self::Attribute { span, .. }
            | Self::Directive { span, .. }
            | Self::Text { span, .. }
            | Self::Comment { span, .. }
            | Self::Interpolation { span, .. }
            | Self::If { span }
            | Self::IfBranch { span, .. }
            | Self::For { span, .. }
            | Self::TextCall { span, .. }
            | Self::CompoundExpression { span, .. } => Some(span),
            Self::HoistRef { .. } => None,
        }
    }

    /// The single dominant borrowed expression this operation renders, if any.
    ///
    /// Returns the one expression for ops that have a clear primary
    /// (interpolation content, a `v-for` source, an `if`-branch condition, a
    /// compound expression, a text-call expression). Ops that carry several
    /// expressions (a directive's arg and exp, a `v-for`'s aliases) or none
    /// (elements, static text) return `None`; read those fields directly.
    pub fn primary_expr(self) -> Option<RenduExprRef<'a>> {
        match self {
            Self::Interpolation { expression, .. }
            | Self::CompoundExpression { expression, .. } => Some(expression),
            Self::For { source, .. } => Some(source),
            Self::IfBranch { condition, .. } => condition,
            Self::TextCall { content, .. } => content,
            _ => None,
        }
    }

    /// Whether this operation is structured control flow (`if`, branch, `for`).
    pub const fn is_control_flow(self) -> bool {
        matches!(
            self,
            Self::If { .. } | Self::IfBranch { .. } | Self::For { .. }
        )
    }
}

/// Ordered render operations.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RenduBlock<'a> {
    pub ops: &'a [RenduOp<'a>],
}

impl<'a> RenduBlock<'a> {
    pub const fn new(ops: &'a [RenduOp<'a>]) -> Self {
        Self { ops }
    }

    pub const fn is_empty(self) -> bool {
        self.ops.is_empty()
    }

    /// Number of render operations in this block.
    pub const fn len(self) -> usize {
        self.ops.len()
    }
}

/// Target and dialect facts available while lowering Rendu.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct RenduCapabilities {
    pub targets: SourceAtlasTargetSet,
    pub coordinate: Option<SourceAtlasCoordinate>,
    pub custom_renderer: bool,
}

impl RenduCapabilities {
    pub const fn empty() -> Self {
        Self {
            targets: SourceAtlasTargetSet::empty(),
            coordinate: None,
            custom_renderer: false,
        }
    }

    pub const fn with_target(mut self, target: SourceAtlasTarget) -> Self {
        self.targets = self.targets.with(target);
        self
    }

    pub const fn with_coordinate(mut self, coordinate: SourceAtlasCoordinate) -> Self {
        self.coordinate = Some(coordinate);
        self
    }

    pub const fn with_custom_renderer(mut self, custom_renderer: bool) -> Self {
        self.custom_renderer = custom_renderer;
        self
    }
}

/// Borrowed Rendu root.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RenduRoot<'a> {
    pub source: RenduSource<'a>,
    pub entry: RenduBlock<'a>,
    pub capabilities: RenduCapabilities,
}

impl<'a> RenduRoot<'a> {
    pub const fn new(
        source: RenduSource<'a>,
        entry: RenduBlock<'a>,
        capabilities: RenduCapabilities,
    ) -> Self {
        Self {
            source,
            entry,
            capabilities,
        }
    }
}

#[cfg(test)]
#[path = "semantic_tests.rs"]
mod tests;
