//! Scope views: [`MarkupConditional`] and [`MarkupList`].
//!
//! Every backend answers the same scope questions: S2 `ui.if` / `ui.for`
//! regions, the `IfNode` / `ForNode` a lowered JSX Relief root carries, and
//! the chains [`super::relief_scopes`] synthesizes over a raw template parse.

use super::element::MarkupElement;
use super::loc_to_range;
use super::relief_scopes::ReliefChain;
use super::s2::children::if_range;
use super::s2::walk::{S2Step, scope_region};
use super::s2::{S2ElementOp, S2Markup};
use crate::ir::ByteRange;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, ForNode, IfNode, TemplateChildNode};
use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op::{ForOp, IfOp};

#[derive(Clone, Copy)]
enum MarkupConditionalInner<'a> {
    Relief(&'a IfNode<'a>),
    ReliefChain(ReliefChain<'a>),
    S2 {
        op: &'a IfOp<'a>,
        doc: &'a S2Markup<'a>,
    },
}

/// A conditional scope: a Vue `v-if` / `v-else-if` / `v-else` chain, a
/// lowered JSX `cond && <x/>` / ternary, or an S2 `ui.if`.
#[derive(Clone, Copy)]
pub struct MarkupConditional<'a> {
    inner: MarkupConditionalInner<'a>,
}

impl<'a> MarkupConditional<'a> {
    pub(super) const fn from_relief(node: &'a IfNode<'a>) -> Self {
        Self {
            inner: MarkupConditionalInner::Relief(node),
        }
    }

    pub(super) const fn from_chain(chain: ReliefChain<'a>) -> Self {
        Self {
            inner: MarkupConditionalInner::ReliefChain(chain),
        }
    }

    pub(super) const fn from_s2(op: &'a IfOp<'a>, doc: &'a S2Markup<'a>) -> Self {
        Self {
            inner: MarkupConditionalInner::S2 { op, doc },
        }
    }

    /// Number of branches in the chain (`v-if` + each `v-else-if` + optional
    /// `v-else`).
    pub fn branch_count(&self) -> usize {
        match self.inner {
            MarkupConditionalInner::Relief(node) => node.branches.len(),
            MarkupConditionalInner::ReliefChain(chain) => chain.branch_count(),
            MarkupConditionalInner::S2 { op, .. } => op.branches.len(),
        }
    }

    /// Whether the chain has a terminal unconditional branch.
    pub fn has_else(&self) -> bool {
        match self.inner {
            MarkupConditionalInner::Relief(node) => node
                .branches
                .iter()
                .any(|branch| branch.condition.is_none()),
            MarkupConditionalInner::ReliefChain(chain) => chain.has_else(),
            MarkupConditionalInner::S2 { op, .. } => {
                op.branches.iter().any(|branch| branch.condition.is_none())
            }
        }
    }

    /// The byte range of the whole conditional in the original source.
    pub fn range(&self) -> ByteRange {
        match self.inner {
            MarkupConditionalInner::Relief(node) => loc_to_range(&node.loc),
            MarkupConditionalInner::ReliefChain(chain) => chain.range(),
            MarkupConditionalInner::S2 { op, doc } => if_range(doc, op),
        }
    }
}

#[derive(Clone, Copy)]
enum MarkupListInner<'a> {
    Relief(&'a ForNode<'a>),
    ReliefDirective {
        element: &'a ElementNode<'a>,
        directive: &'a DirectiveNode<'a>,
    },
    S2 {
        op: &'a ForOp<'a>,
        doc: &'a S2Markup<'a>,
    },
}

/// A list scope: a Vue `v-for`, a lowered JSX `items.map(...)`, or an S2
/// `ui.for`.
///
/// The repeated element is still visited as a normal [`MarkupElement`]; the
/// element that carried the `v-for` (a `<template>` included) is the
/// repeated element [`Self::walk_elements`] yields.
#[derive(Clone, Copy)]
pub struct MarkupList<'a> {
    inner: MarkupListInner<'a>,
}

impl<'a> MarkupList<'a> {
    pub(super) const fn from_relief(node: &'a ForNode<'a>) -> Self {
        Self {
            inner: MarkupListInner::Relief(node),
        }
    }

    pub(super) const fn from_relief_directive(
        element: &'a ElementNode<'a>,
        directive: &'a DirectiveNode<'a>,
    ) -> Self {
        Self {
            inner: MarkupListInner::ReliefDirective { element, directive },
        }
    }

    pub(super) const fn from_s2(op: &'a ForOp<'a>, doc: &'a S2Markup<'a>) -> Self {
        Self {
            inner: MarkupListInner::S2 { op, doc },
        }
    }

    /// The source iterable expression text (`items` in `item in items`).
    pub fn source_expression(&self) -> Option<&'a str> {
        match self.inner {
            MarkupListInner::Relief(node) => simple_text(Some(&node.source)),
            MarkupListInner::ReliefDirective { directive, .. } => {
                split_directive(directive).map(|(_, source)| source)
            }
            MarkupListInner::S2 { op, .. } => admitted_text(&op.binding.source),
        }
    }

    /// The value-alias expression text (`item` in `item in items`).
    pub fn value_alias(&self) -> Option<&'a str> {
        match self.inner {
            MarkupListInner::Relief(node) => simple_text(node.value_alias.as_ref()),
            MarkupListInner::ReliefDirective { directive, .. } => {
                split_directive(directive).and_then(|(value, _)| value)
            }
            MarkupListInner::S2 { op, .. } => admitted_text(&op.binding.value),
        }
    }

    /// Visit the elements this list repeats — the elements a `:key`
    /// requirement applies to.
    pub fn walk_elements(&self, visitor: &mut impl FnMut(MarkupElement<'a>)) {
        match self.inner {
            MarkupListInner::Relief(node) => {
                for child in &node.children {
                    if let TemplateChildNode::Element(element) = child {
                        visitor(MarkupElement::new(element));
                    }
                }
            }
            MarkupListInner::ReliefDirective { element, .. } => {
                visitor(MarkupElement::new(element));
            }
            MarkupListInner::S2 { op, doc } => match scope_region(doc, op.span, &op.region.ops) {
                S2Step::Element(element, _) => visitor(element),
                S2Step::Region(region) => {
                    for op in region {
                        if let Some(element) = S2ElementOp::from_op(op) {
                            visitor(MarkupElement::from_s2(element, doc));
                        }
                    }
                }
            },
        }
    }

    /// The byte range of the whole list scope in the original source.
    pub fn range(&self) -> ByteRange {
        match self.inner {
            MarkupListInner::Relief(node) => loc_to_range(&node.loc),
            MarkupListInner::ReliefDirective { element, .. } => loc_to_range(&element.loc),
            MarkupListInner::S2 { op, doc } => doc.open_tag_range(op.span),
        }
    }
}

/// A raw `v-for` directive's value alias and source, split with the
/// lowering's own grammar (one splitter, never a second reading).
fn split_directive<'a>(directive: &'a DirectiveNode<'a>) -> Option<(Option<&'a str>, &'a str)> {
    let Some(ExpressionNode::Simple(value)) = directive.exp.as_ref() else {
        return None;
    };
    let (alias, source) = vize_s1_to_s2::lower::split_v_for_value(value.content)?;
    let alias = alias.map(str::trim).filter(|text| !text.is_empty());
    Some(source.trim())
        .filter(|text| !text.is_empty())
        .map(|source| (alias, source))
}

fn simple_text<'a>(expression: Option<&'a ExpressionNode<'a>>) -> Option<&'a str> {
    match expression {
        Some(ExpressionNode::Simple(simple)) => {
            Some(simple.content.trim()).filter(|text| !text.is_empty())
        }
        _ => None,
    }
}

/// An S2 `ui.for` position's text, unless it is the undecomposable escape
/// (`OpaqueReason::ForValue`) or an unauthored hole.
fn admitted_text<'a>(expression: &ExprRef<'a>) -> Option<&'a str> {
    if let ExprRef::Opaque(opaque) = expression
        && opaque.reason == OpaqueReason::ForValue
    {
        return None;
    }
    Some(expression.source().trim()).filter(|text| !text.is_empty())
}
