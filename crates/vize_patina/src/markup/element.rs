//! [`MarkupElement`]: one element / component / fragment / template / slot.

use super::jsx_names::{jsx_element_kind, jsx_element_name, jsx_element_ref, jsx_fragment_ref};
use super::l2::surface::element_at;
use super::l2::{L2ElementOp, L2Markup};
use super::node::MarkupNode;
use super::{MarkupElementKind, relief_scopes, span_to_range};
use crate::ir::ByteRange;
use oxc_ast::ast::{JSXElement, JSXFragment};
use std::marker::PhantomData;
use vize_relief::{ElementNode, ElementType};

mod opening;
pub(super) mod queries;

use queries::{is_lint_component, l2_template_is_special};

#[derive(Clone, Copy)]
pub(super) enum MarkupElementInner<'a> {
    Relief(&'a ElementNode<'a>),
    /// Source-shaped L1 view used by authored-tree consumers. This is not an
    /// unwrapped L2 template carrier: its kind follows the actual tag/attrs.
    Authored {
        element: &'a vize_l1::Element<'a>,
        doc: &'a L2Markup<'a>,
        frozen: bool,
        opens_v_pre: bool,
        ns: vize_relief::Namespace,
    },
    JsxElement {
        node: *const JSXElement<'a>,
        offset: u32,
    },
    JsxFragment {
        node: *const JSXFragment<'a>,
        offset: u32,
    },
    L2 {
        op: L2ElementOp<'a>,
        doc: &'a L2Markup<'a>,
        surface: Option<&'a vize_l1::Element<'a>>,
    },
    /// An authored `<template>` carrier L2 unwrapped into a `ui.if` branch or
    /// `ui.for` region: its surface comes from L1, its children from the
    /// region it was unwrapped into.
    L2Carrier {
        element: &'a vize_l1::Element<'a>,
        doc: &'a L2Markup<'a>,
        region: &'a [vize_l2::op::Op<'a>],
        span: vize_l0::Span,
    },
}

/// Wrapper around a concrete element node.
#[derive(Clone, Copy)]
pub struct MarkupElement<'a> {
    pub(super) inner: MarkupElementInner<'a>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> MarkupElement<'a> {
    /// Create a markup element wrapper from a Vue template node.
    pub const fn new(node: &'a ElementNode<'a>) -> Self {
        Self::from_inner(MarkupElementInner::Relief(node))
    }

    pub(super) const fn from_inner(inner: MarkupElementInner<'a>) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    pub(super) const fn from_jsx_element(node: *const JSXElement<'a>, offset: u32) -> Self {
        Self::from_inner(MarkupElementInner::JsxElement { node, offset })
    }

    pub(super) const fn from_jsx_fragment(node: *const JSXFragment<'a>, offset: u32) -> Self {
        Self::from_inner(MarkupElementInner::JsxFragment { node, offset })
    }

    /// Wrap an L2 element-shaped op, locating its authored L1 element when the
    /// artifact carries a surface tree.
    pub(super) fn from_l2(op: L2ElementOp<'a>, doc: &'a L2Markup<'a>) -> Self {
        let surface = doc
            .surface
            .filter(|_| !op.is_synthesized(doc))
            .and_then(|tree| element_at(tree, op.span().start));
        Self::from_inner(MarkupElementInner::L2 { op, doc, surface })
    }

    pub(super) const fn from_l2_carrier(
        element: &'a vize_l1::Element<'a>,
        doc: &'a L2Markup<'a>,
        region: &'a [vize_l2::op::Op<'a>],
        span: vize_l0::Span,
    ) -> Self {
        Self::from_inner(MarkupElementInner::L2Carrier {
            element,
            doc,
            region,
            span,
        })
    }

    /// Tag name.
    pub fn tag(&self) -> &str {
        match self.inner {
            MarkupElementInner::Relief(node) => node.tag,
            MarkupElementInner::Authored { element, .. } => element.tag(),
            MarkupElementInner::JsxElement { node, .. } => {
                jsx_element_name(&jsx_element_ref(node).opening_element.name)
            }
            MarkupElementInner::JsxFragment { .. } => "",
            MarkupElementInner::L2 { op, .. } => op.tag(),
            MarkupElementInner::L2Carrier { element, .. } => element.tag(),
        }
    }

    /// Element classification.
    pub fn kind(&self) -> MarkupElementKind {
        match self.inner {
            MarkupElementInner::Authored {
                element, frozen, ..
            } => super::authored::kind(element, frozen),
            MarkupElementInner::Relief(node) => match node.tag_type {
                ElementType::Element => MarkupElementKind::Element,
                ElementType::Component => MarkupElementKind::Component,
                ElementType::Slot => MarkupElementKind::Slot,
                ElementType::Template => MarkupElementKind::Template,
            },
            MarkupElementInner::JsxElement { node, .. } => {
                jsx_element_kind(&jsx_element_ref(node).opening_element.name)
            }
            MarkupElementInner::JsxFragment { .. } => MarkupElementKind::Template,
            MarkupElementInner::L2 { op, surface, .. } => match op {
                L2ElementOp::Slot(_) => MarkupElementKind::Slot,
                _ if op.tag() == "template" && l2_template_is_special(op, surface) => {
                    MarkupElementKind::Template
                }
                // A template classifies the way the lint-mode parse does: a
                // component is a core built-in or a capitalized tag. L2's
                // element/component split is DOM resolution (`is_native_tag`),
                // which the lint lane's rules and snapshots are not written
                // against.
                _ if surface.is_some() => {
                    if is_lint_component(op.tag()) {
                        MarkupElementKind::Component
                    } else {
                        MarkupElementKind::Element
                    }
                }
                L2ElementOp::Component(_) => MarkupElementKind::Component,
                L2ElementOp::Element(_) => MarkupElementKind::Element,
            },
            MarkupElementInner::L2Carrier { .. } => MarkupElementKind::Template,
        }
    }

    /// Whether this node is a framework component.
    pub fn is_component(&self) -> bool {
        matches!(self.kind(), MarkupElementKind::Component)
    }

    /// Byte range in the original source.
    pub fn range(&self) -> ByteRange {
        match self.inner {
            MarkupElementInner::Relief(node) => relief_scopes::carrier_range(node),
            MarkupElementInner::Authored { element, doc, .. } => {
                super::authored::range(element, doc)
            }
            MarkupElementInner::JsxElement { node, offset } => {
                span_to_range(jsx_element_ref(node).span, offset)
            }
            MarkupElementInner::JsxFragment { node, offset } => {
                span_to_range(jsx_fragment_ref(node).span, offset)
            }
            // An implicit table owner (`tbody` / `tr`) is zero-width at the
            // tag name of the row or cell that opened it, as the parser's
            // tree construction records it.
            MarkupElementInner::L2 { op, doc, .. } if op.is_synthesized(doc) => {
                let at = op.span().start + 1;
                ByteRange::new(at, at)
            }
            MarkupElementInner::L2 { op, doc, .. } => doc.open_tag_range(op.span()),
            MarkupElementInner::L2Carrier { span, doc, .. } => doc.open_tag_range(span),
        }
    }

    /// Visit direct child nodes, as authored.
    ///
    /// In a template an element that carries `v-if` / `v-for` is itself the
    /// child — the scope structure is what the visitor's scope hooks carry.
    /// A JSX conditional or list is an expression, so it is one
    /// [`MarkupNode::If`] / [`MarkupNode::For`] (as is an `IfNode` /
    /// `ForNode` in a lowered JSX root).
    pub fn walk_children(&self, visitor: &mut impl FnMut(MarkupNode<'a>)) {
        match self.inner {
            MarkupElementInner::Authored {
                element,
                doc,
                frozen,
                ns,
                ..
            } => super::authored::walk_children(element, doc, frozen, ns, visitor),
            MarkupElementInner::Relief(node) => {
                for child in &node.children {
                    visitor(MarkupNode::from_relief_child(child));
                }
            }
            MarkupElementInner::JsxElement { node, offset } => {
                for child in &jsx_element_ref(node).children {
                    visitor(MarkupNode::from_jsx_child(child, offset));
                }
            }
            MarkupElementInner::JsxFragment { node, offset } => {
                for child in &jsx_fragment_ref(node).children {
                    visitor(MarkupNode::from_jsx_child(child, offset));
                }
            }
            MarkupElementInner::L2 { op, doc, .. } => {
                super::l2::children::walk_nodes(doc, op.children(), visitor);
            }
            MarkupElementInner::L2Carrier { doc, region, .. } => {
                super::l2::children::walk_nodes(doc, region, visitor);
            }
        }
    }
}
