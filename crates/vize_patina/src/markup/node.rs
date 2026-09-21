//! [`MarkupText`] and the [`MarkupNode`] child view.

use super::element::MarkupElement;
use super::jsx_names::jsx_text_ref;
use super::{loc_to_range, span_to_range};
use crate::ir::ByteRange;
use oxc_ast::ast::JSXText;
use std::marker::PhantomData;
use vize_relief::{TemplateChildNode, TextNode};

#[derive(Clone, Copy)]
enum MarkupTextInner<'a> {
    Relief(&'a TextNode<'a>),
    Jsx {
        node: *const JSXText<'a>,
        offset: u32,
    },
    Static {
        content: &'a str,
        range: ByteRange,
    },
}

/// Text node view.
#[derive(Clone, Copy)]
pub struct MarkupText<'a> {
    inner: MarkupTextInner<'a>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> MarkupText<'a> {
    const fn from_inner(inner: MarkupTextInner<'a>) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    pub(super) const fn from_relief(node: &'a TextNode) -> Self {
        Self::from_inner(MarkupTextInner::Relief(node))
    }

    pub(super) const fn from_jsx(node: *const JSXText<'a>, offset: u32) -> Self {
        Self::from_inner(MarkupTextInner::Jsx { node, offset })
    }

    pub(super) const fn from_static(content: &'a str, range: ByteRange) -> Self {
        Self::from_inner(MarkupTextInner::Static { content, range })
    }

    /// Raw text content.
    pub fn content(&self) -> &'a str {
        match self.inner {
            MarkupTextInner::Relief(node) => node.content,
            MarkupTextInner::Jsx { node, .. } => jsx_text_ref(node).value.as_str(),
            MarkupTextInner::Static { content, .. } => content,
        }
    }

    /// Byte range.
    pub fn range(&self) -> ByteRange {
        match self.inner {
            MarkupTextInner::Relief(node) => loc_to_range(&node.loc),
            MarkupTextInner::Jsx { node, offset } => span_to_range(jsx_text_ref(node).span, offset),
            MarkupTextInner::Static { range, .. } => range,
        }
    }

    /// Whether the text contains any non-whitespace content.
    pub fn is_significant(&self) -> bool {
        !self.content().trim().is_empty()
    }
}

/// Direct child node view.
#[derive(Clone, Copy)]
pub enum MarkupNode<'a> {
    /// Concrete element.
    Element(MarkupElement<'a>),
    /// Text node.
    Text(MarkupText<'a>),
    /// Comment node.
    Comment(ByteRange),
    /// Interpolation node.
    Interpolation(ByteRange),
    /// A JSX conditional expression (`cond && <x/>`, a ternary).
    If(ByteRange),
    /// A JSX list expression (`items.map(...)`).
    For(ByteRange),
    /// Any other node that is currently not projected.
    Other(ByteRange),
}

impl<'a> MarkupNode<'a> {
    /// One Relief child as authored: a raw template's `v-if` / `v-for`
    /// carrier is an element; a lowered JSX root's `IfNode` / `ForNode` is a
    /// scope node.
    pub(super) fn from_relief_child(child: &'a TemplateChildNode<'a>) -> Self {
        match child {
            TemplateChildNode::Element(element) => Self::Element(MarkupElement::new(element)),
            TemplateChildNode::Text(text) => Self::Text(MarkupText::from_relief(text)),
            TemplateChildNode::Comment(comment) => Self::Comment(loc_to_range(&comment.loc)),
            TemplateChildNode::Interpolation(interpolation) => {
                Self::Interpolation(loc_to_range(&interpolation.loc))
            }
            TemplateChildNode::If(if_node) => Self::If(loc_to_range(&if_node.loc)),
            TemplateChildNode::For(for_node) => Self::For(loc_to_range(&for_node.loc)),
            other => Self::Other(loc_to_range(other.loc())),
        }
    }
}
