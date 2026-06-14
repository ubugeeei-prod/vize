//! Borrowed render-semantic Rendu vocabulary.

use crate::{
    ElementType, ExpressionNode, SourceLocation, TemplateChildNode, TextCallContent,
    source_atlas::{SourceAtlasCoordinate, SourceAtlasTarget, SourceAtlasTargetSet},
};

/// Source span carried by Rendu operations.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct RenduSpan {
    pub start: u32,
    pub end: u32,
}

impl RenduSpan {
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn from_location(loc: &SourceLocation) -> Self {
        Self::new(loc.start.offset, loc.end.offset)
    }
}

/// Source view used by a Rendu root.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RenduSource<'a> {
    pub filename: Option<&'a str>,
    pub source: &'a str,
}

impl<'a> RenduSource<'a> {
    pub const fn anonymous(source: &'a str) -> Self {
        Self {
            filename: None,
            source,
        }
    }

    pub const fn named(filename: &'a str, source: &'a str) -> Self {
        Self {
            filename: Some(filename),
            source,
        }
    }
}

/// Borrowed expression material that a Rendu operation may reference.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RenduExprRef<'a> {
    Relief(&'a str),
    Oxc(&'a str),
    Croquis(&'a str),
}

impl<'a> RenduExprRef<'a> {
    pub fn from_expression(expression: &'a ExpressionNode<'a>) -> Self {
        match expression {
            ExpressionNode::Simple(simple) => Self::Relief(simple.content.as_str()),
            ExpressionNode::Compound(compound) => Self::Relief(compound.loc.source.as_str()),
        }
    }

    /// The borrowed source text of this expression, regardless of origin.
    pub const fn text(self) -> &'a str {
        match self {
            Self::Relief(text) | Self::Oxc(text) | Self::Croquis(text) => text,
        }
    }
}

/// Element-like render operation kind.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum RenduElementKind {
    Element,
    Component,
    SlotOutlet,
    Template,
}

impl From<ElementType> for RenduElementKind {
    fn from(kind: ElementType) -> Self {
        match kind {
            ElementType::Element => Self::Element,
            ElementType::Component => Self::Component,
            ElementType::Slot => Self::SlotOutlet,
            ElementType::Template => Self::Template,
        }
    }
}

/// Render-semantic operation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenduOp<'a> {
    Element {
        tag: &'a str,
        kind: RenduElementKind,
        span: RenduSpan,
    },
    Text {
        content: &'a str,
        span: RenduSpan,
    },
    Comment {
        content: &'a str,
        span: RenduSpan,
    },
    Interpolation {
        expression: RenduExprRef<'a>,
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
                span: RenduSpan::from_location(&comment.loc),
            },
            TemplateChildNode::Interpolation(interpolation) => Self::Interpolation {
                expression: RenduExprRef::from_expression(&interpolation.content),
                span: RenduSpan::from_location(&interpolation.loc),
            },
            TemplateChildNode::If(node) => Self::If {
                span: RenduSpan::from_location(&node.loc),
            },
            TemplateChildNode::IfBranch(branch) => Self::IfBranch {
                condition: branch.condition.as_ref().map(RenduExprRef::from_expression),
                span: RenduSpan::from_location(&branch.loc),
            },
            TemplateChildNode::For(node) => Self::For {
                source: RenduExprRef::from_expression(&node.source),
                value: node.value_alias.as_ref().map(RenduExprRef::from_expression),
                key: node.key_alias.as_ref().map(RenduExprRef::from_expression),
                index: node
                    .object_index_alias
                    .as_ref()
                    .map(RenduExprRef::from_expression),
                span: RenduSpan::from_location(&node.loc),
            },
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
