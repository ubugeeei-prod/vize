//! Leaf model types of the Rendu render-IR: spans, source views, expression
//! refs, and element classification. Kept separate from the operation vocabulary
//! in `semantic.rs` so each file stays focused and under the source-length guard.

use crate::{ElementType, ExpressionNode, Position, SimpleExpressionNode, SourceLocation};

/// Source span carried by Rendu operations.
///
/// Carries full source [`Position`]s (offset/line/column) so a backend can emit
/// source-mapped output straight from Rendu without reaching back into the
/// Relief node.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct RenduSpan {
    pub start: Position,
    pub end: Position,
}

impl RenduSpan {
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub const fn from_location(loc: &SourceLocation) -> Self {
        Self::new(loc.start, loc.end)
    }

    /// Byte offset of the span start.
    pub const fn start_offset(self) -> u32 {
        self.start.offset
    }

    /// Byte offset of the span end.
    pub const fn end_offset(self) -> u32 {
        self.end.offset
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
///
/// `Node` carries the real Relief [`ExpressionNode`] (for identifier prefixing,
/// `is_static`, ...); equality is by source text across all variants.
#[derive(Debug, Clone, Copy)]
pub enum RenduExprRef<'a> {
    Relief(&'a str),
    Oxc(&'a str),
    Croquis(&'a str),
    Node(&'a ExpressionNode<'a>),
}

impl<'a> RenduExprRef<'a> {
    pub const fn from_expression(expression: &'a ExpressionNode<'a>) -> Self {
        Self::Node(expression)
    }

    /// The borrowed source text of this expression, regardless of origin.
    pub fn text(self) -> &'a str {
        match self {
            Self::Relief(text) | Self::Oxc(text) | Self::Croquis(text) => text,
            Self::Node(ExpressionNode::Simple(simple)) => simple.content.as_str(),
            Self::Node(ExpressionNode::Compound(compound)) => compound.loc.source.as_str(),
        }
    }

    /// The underlying Relief expression node, when this ref carries one.
    pub const fn node(self) -> Option<&'a ExpressionNode<'a>> {
        match self {
            Self::Node(expression) => Some(expression),
            _ => None,
        }
    }

    /// Whether this ref carries a simple Relief expression with the given text.
    pub fn is_simple(self, text: &str) -> bool {
        matches!(self.node(), Some(ExpressionNode::Simple(simple)) if simple.content == text)
    }
}

impl PartialEq for RenduExprRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.text() == other.text()
    }
}

impl Eq for RenduExprRef<'_> {}

/// Element-like render operation kind.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum RenduElementKind {
    Element,
    Component,
    SlotOutlet,
    Template,
}

impl RenduElementKind {
    /// Whether this is a user component rather than a plain host element.
    pub const fn is_component(self) -> bool {
        matches!(self, Self::Component)
    }

    /// Whether this is a `<slot>` outlet.
    pub const fn is_slot_outlet(self) -> bool {
        matches!(self, Self::SlotOutlet)
    }

    /// Whether this is a `<template>` fragment wrapper.
    pub const fn is_template(self) -> bool {
        matches!(self, Self::Template)
    }

    /// Whether this is a plain host element (not a component, slot, or
    /// template).
    pub const fn is_plain_element(self) -> bool {
        matches!(self, Self::Element)
    }
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

/// Borrowed directive modifiers (`.stop`, `.prevent`, `.camel`, ...).
///
/// Carries a directive's modifier expressions without owning them. Equality is
/// by modifier *name* (the expression `content`), matching [`RenduExprRef`]'s
/// by-text equality, so an op holding `RenduModifiers` can still be `Copy + Eq`.
#[derive(Debug, Clone, Copy)]
pub struct RenduModifiers<'a> {
    modifiers: &'a [SimpleExpressionNode<'a>],
}

impl<'a> RenduModifiers<'a> {
    pub const fn new(modifiers: &'a [SimpleExpressionNode<'a>]) -> Self {
        Self { modifiers }
    }

    /// An empty modifier set, for ops with no directive modifiers.
    pub const fn empty() -> Self {
        Self { modifiers: &[] }
    }

    pub const fn is_empty(self) -> bool {
        self.modifiers.is_empty()
    }

    pub const fn len(self) -> usize {
        self.modifiers.len()
    }

    /// The modifier names in source order.
    pub fn names(self) -> impl Iterator<Item = &'a str> {
        self.modifiers
            .iter()
            .map(|modifier| modifier.content.as_str())
    }

    /// Whether a modifier with the given name is present.
    pub fn contains(self, name: &str) -> bool {
        self.names().any(|present| present == name)
    }
}

impl PartialEq for RenduModifiers<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.names().zip(other.names()).all(|(a, b)| a == b)
    }
}

impl Eq for RenduModifiers<'_> {}
