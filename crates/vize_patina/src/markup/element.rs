//! [`MarkupElement`]: one element / component / fragment / template / slot.

use super::attribute::MarkupAttribute;
use super::binding::MarkupBinding;
use super::directive::MarkupDirective;
use super::jsx_names::{jsx_attribute_directive_kind, jsx_element_ref, jsx_fragment_ref};
use super::node::MarkupNode;
use super::s2::binding::{S2Item, walk_items};
use super::s2::surface::element_at;
use super::s2::{S2ElementOp, S2Markup};
use super::{MarkupElementKind, relief_scopes};
use crate::ir::ByteRange;
use oxc_ast::ast::{JSXAttributeItem, JSXElement, JSXFragment};
use std::marker::PhantomData;
use vize_relief::{ElementNode, PropNode};

mod projected;
mod queries;

use projected::relief_kind;

#[derive(Clone, Copy)]
pub(super) enum MarkupElementInner<'a> {
    Relief(&'a ElementNode<'a>),
    JsxElement {
        node: *const JSXElement<'a>,
        offset: u32,
    },
    JsxFragment {
        node: *const JSXFragment<'a>,
        offset: u32,
    },
    S2 {
        op: S2ElementOp<'a>,
        doc: &'a S2Markup<'a>,
        surface: Option<&'a vize_s1::Element<'a>>,
    },
    /// An authored `<template>` carrier S2 unwrapped into a `ui.if` branch or
    /// `ui.for` region: its surface comes from S1, its children from the
    /// region it was unwrapped into.
    S2Carrier {
        element: &'a vize_s1::Element<'a>,
        doc: &'a S2Markup<'a>,
        region: &'a [vize_s2::op::Op<'a>],
        span: vize_s0::Span,
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

    /// Wrap an S2 element-shaped op, locating its authored S1 element when the
    /// artifact carries a surface tree.
    pub(super) fn from_s2(op: S2ElementOp<'a>, doc: &'a S2Markup<'a>) -> Self {
        let surface = doc
            .surface
            .filter(|_| !op.is_synthesized(doc))
            .and_then(|tree| element_at(tree, op.span().start));
        Self::from_inner(MarkupElementInner::S2 { op, doc, surface })
    }

    pub(super) const fn from_s2_carrier(
        element: &'a vize_s1::Element<'a>,
        doc: &'a S2Markup<'a>,
        region: &'a [vize_s2::op::Op<'a>],
        span: vize_s0::Span,
    ) -> Self {
        Self::from_inner(MarkupElementInner::S2Carrier {
            element,
            doc,
            region,
            span,
        })
    }

    /// Tag name.
    #[inline]
    pub fn tag(&self) -> &str {
        match self.inner {
            MarkupElementInner::Relief(node) => node.tag,
            _ => self.projected_tag(),
        }
    }

    /// Element classification.
    #[inline]
    pub fn kind(&self) -> MarkupElementKind {
        match self.inner {
            MarkupElementInner::Relief(node) => relief_kind(node.tag_type),
            _ => self.projected_kind(),
        }
    }

    /// Whether this node is a framework component.
    #[inline]
    pub fn is_component(&self) -> bool {
        matches!(self.kind(), MarkupElementKind::Component)
    }

    /// Byte range in the original source.
    #[inline]
    pub fn range(&self) -> ByteRange {
        match self.inner {
            MarkupElementInner::Relief(node) => relief_scopes::carrier_range(node),
            _ => self.projected_range(),
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
            MarkupElementInner::S2 { op, doc, .. } => {
                super::s2::children::walk_nodes(doc, op.children(), visitor);
            }
            MarkupElementInner::S2Carrier { doc, region, .. } => {
                super::s2::children::walk_nodes(doc, region, visitor);
            }
        }
    }

    /// Visit static attributes on this element.
    pub fn walk_attributes(&self, visitor: &mut impl FnMut(MarkupAttribute<'a>)) {
        match self.inner {
            MarkupElementInner::Relief(node) => {
                for prop in &node.props {
                    if let PropNode::Attribute(attr) = prop {
                        visitor(MarkupAttribute::from_relief(attr));
                    }
                }
            }
            MarkupElementInner::JsxElement { node, offset } => {
                for attribute in &jsx_element_ref(node).opening_element.attributes {
                    if let JSXAttributeItem::Attribute(attr) = attribute {
                        visitor(MarkupAttribute::from_jsx(&**attr as *const _, offset));
                    }
                }
            }
            MarkupElementInner::JsxFragment { .. } => {}
            MarkupElementInner::S2 { .. } | MarkupElementInner::S2Carrier { .. } => self
                .walk_s2_items(&mut |item| match item {
                    S2Item::Attribute { attribute, doc } => {
                        visitor(MarkupAttribute::from_s2(attribute, doc));
                    }
                    S2Item::Surface { attr, doc } if !item.is_directive() => {
                        visitor(MarkupAttribute::from_surface(attr, doc));
                    }
                    S2Item::Binding(_) | S2Item::Surface { .. } => {}
                }),
        }
    }

    /// Visit directives on this element.
    ///
    /// For Vue templates this yields the explicit `v-*` directives except the
    /// structural ones, which the facade consumes into scopes. For JSX,
    /// directive-like attributes — events (`onClick`) and dynamic bindings
    /// (`class={…}`) — are projected as directives too, so a rule that reasons
    /// over [`MarkupDirective`] behaves consistently across backends. Plain
    /// static JSX attributes (`id="x"`) are *not* directives; use
    /// [`Self::walk_attributes`] or [`Self::walk_bindings`] for those.
    #[inline]
    pub fn walk_directives(&self, visitor: &mut impl FnMut(MarkupDirective<'a>)) {
        match self.inner {
            MarkupElementInner::Relief(node) => {
                for prop in &node.props {
                    if let PropNode::Directive(dir) = prop
                        && !relief_scopes::is_structural_directive(dir)
                    {
                        visitor(MarkupDirective::from_relief(dir));
                    }
                }
            }
            MarkupElementInner::JsxElement { node, offset } => {
                for attribute in &jsx_element_ref(node).opening_element.attributes {
                    if let JSXAttributeItem::Attribute(attr) = attribute
                        && jsx_attribute_directive_kind(attr).is_some()
                    {
                        visitor(MarkupDirective::from_jsx(&**attr as *const _, offset));
                    }
                }
            }
            MarkupElementInner::JsxFragment { .. } => {}
            MarkupElementInner::S2 { .. } | MarkupElementInner::S2Carrier { .. } => self
                .walk_s2_items(&mut |item| match item {
                    S2Item::Binding(binding) => visitor(MarkupDirective::from_s2(binding)),
                    S2Item::Surface { attr, doc } if item.is_directive() => {
                        visitor(MarkupDirective::from_surface(attr, doc));
                    }
                    S2Item::Attribute { .. } | S2Item::Surface { .. } => {}
                }),
        }
    }

    /// Visit every *binding* on this element in source order.
    ///
    /// A [`MarkupBinding`] is the normalized, backend-neutral view of anything
    /// written on the opening tag: plain attributes, `v-bind` (including
    /// `:key` shorthand), events (`v-on` / `onClick`), `v-model`, and custom
    /// directives. This is the projection most rules should target, because the
    /// same closure then runs unchanged over Vue templates and JSX/TSX.
    #[inline]
    pub fn walk_bindings(&self, visitor: &mut impl FnMut(MarkupBinding<'a>)) {
        match self.inner {
            MarkupElementInner::Relief(node) => {
                for prop in &node.props {
                    match prop {
                        PropNode::Attribute(attr) => {
                            visitor(MarkupBinding::from_relief_attribute(attr));
                        }
                        PropNode::Directive(dir)
                            if !relief_scopes::is_structural_directive(dir) =>
                        {
                            visitor(MarkupBinding::from_relief_directive(dir));
                        }
                        PropNode::Directive(_) => {}
                    }
                }
            }
            MarkupElementInner::JsxElement { node, offset } => {
                for attribute in &jsx_element_ref(node).opening_element.attributes {
                    if let JSXAttributeItem::Attribute(attr) = attribute {
                        visitor(MarkupBinding::from_jsx(&**attr as *const _, offset));
                    }
                }
            }
            MarkupElementInner::JsxFragment { .. } => {}
            MarkupElementInner::S2 { .. } | MarkupElementInner::S2Carrier { .. } => {
                self.walk_s2_items(&mut |item| visitor(MarkupBinding::from_s2_item(item)));
            }
        }
    }

    pub(super) fn walk_s2_items(&self, visitor: &mut impl FnMut(S2Item<'a>)) {
        match self.inner {
            MarkupElementInner::S2 { op, doc, surface } => {
                walk_items(doc, op.attributes(), op.bindings(), surface, visitor);
            }
            MarkupElementInner::S2Carrier { element, doc, .. } => {
                walk_items(doc, &[], &[], Some(element), visitor);
            }
            MarkupElementInner::Relief(_)
            | MarkupElementInner::JsxElement { .. }
            | MarkupElementInner::JsxFragment { .. } => {}
        }
    }
}
