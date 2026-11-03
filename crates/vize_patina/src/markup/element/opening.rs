//! Opening-tag projections for each markup backend.

use super::{MarkupElement, MarkupElementInner};
use crate::markup::jsx_names::{jsx_attribute_directive_kind, jsx_element_ref};
use crate::markup::l2::binding::{L2Item, walk_items};
use crate::markup::l2::surface::SurfaceDirective;
use crate::markup::{MarkupAttribute, MarkupBinding, MarkupDirective, relief_scopes};
use oxc_ast::ast::JSXAttributeItem;
use vize_relief::PropNode;

impl<'a> MarkupElement<'a> {
    /// Visit static attributes on this element.
    pub fn walk_attributes(&self, visitor: &mut impl FnMut(MarkupAttribute<'a>)) {
        match self.inner {
            MarkupElementInner::Authored {
                element,
                doc,
                frozen,
                opens_v_pre,
                ..
            }
            | MarkupElementInner::Surface {
                element,
                doc,
                frozen,
                opens_v_pre,
                ..
            } => {
                super::super::authored::walk_attributes(element, doc, frozen, opens_v_pre, visitor);
            }
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
                        visitor(MarkupAttribute::from_jsx(&raw const **attr, offset));
                    }
                }
            }
            MarkupElementInner::JsxFragment { .. } => {}
            MarkupElementInner::L2 { .. } | MarkupElementInner::L2Carrier { .. } => self
                .walk_l2_items(&mut |item| match item {
                    L2Item::Attribute { attribute, doc } => {
                        visitor(MarkupAttribute::from_l2(attribute, doc));
                    }
                    L2Item::Surface { attr, doc } if !item.is_directive() => {
                        visitor(MarkupAttribute::from_surface(attr, doc));
                    }
                    L2Item::Binding(_) | L2Item::Surface { .. } => {}
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
    pub fn walk_directives(&self, visitor: &mut impl FnMut(MarkupDirective<'a>)) {
        match self.inner {
            MarkupElementInner::Authored {
                element,
                doc,
                frozen,
                ..
            }
            | MarkupElementInner::Surface {
                element,
                doc,
                frozen,
                ..
            } => {
                if !frozen {
                    for attr in &element.open.attrs {
                        if let Some(directive) = super::super::authored::directive(attr) {
                            visitor(MarkupDirective::from_surface(attr, doc, directive));
                        }
                    }
                }
            }
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
                        visitor(MarkupDirective::from_jsx(&raw const **attr, offset));
                    }
                }
            }
            MarkupElementInner::JsxFragment { .. } => {}
            MarkupElementInner::L2 { .. } | MarkupElementInner::L2Carrier { .. } => self
                .walk_l2_items(&mut |item| match item {
                    L2Item::Binding(binding) => visitor(MarkupDirective::from_l2(binding)),
                    L2Item::Surface { attr, doc } => {
                        if let Some(directive) = SurfaceDirective::parse(attr.name.text) {
                            visitor(MarkupDirective::from_surface(attr, doc, directive));
                        }
                    }
                    L2Item::Attribute { .. } => {}
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
    pub fn walk_bindings(&self, visitor: &mut impl FnMut(MarkupBinding<'a>)) {
        match self.inner {
            MarkupElementInner::Authored {
                element,
                doc,
                frozen,
                opens_v_pre,
                ..
            }
            | MarkupElementInner::Surface {
                element,
                doc,
                frozen,
                opens_v_pre,
                ..
            } => {
                for attr in &element.open.attrs {
                    if let Some(directive) = super::super::authored::kept(attr, frozen, opens_v_pre)
                    {
                        let name = frozen
                            .then(|| super::super::authored::frozen_name(attr, doc, opens_v_pre));
                        visitor(MarkupBinding::from_surface_spelling(
                            attr, doc, name, directive,
                        ));
                    }
                }
            }
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
                        visitor(MarkupBinding::from_jsx(&raw const **attr, offset));
                    }
                }
            }
            MarkupElementInner::JsxFragment { .. } => {}
            MarkupElementInner::L2 { .. } | MarkupElementInner::L2Carrier { .. } => {
                self.walk_l2_items(&mut |item| visitor(MarkupBinding::from_l2_item(item)));
            }
        }
    }

    pub(in crate::markup) fn walk_l2_items(&self, visitor: &mut impl FnMut(L2Item<'a>)) {
        match self.inner {
            MarkupElementInner::L2 { op, doc, surface } => {
                let surface = surface.zip(doc.l1);
                walk_items(doc, op.attributes(), op.bindings(), surface, visitor);
            }
            MarkupElementInner::L2Carrier { element, doc, .. } => {
                let surface = doc.l1.map(|l1| (element, l1));
                walk_items(doc, &[], &[], surface, visitor);
            }
            MarkupElementInner::Authored { .. }
            | MarkupElementInner::Surface { .. }
            | MarkupElementInner::Relief(_)
            | MarkupElementInner::JsxElement { .. }
            | MarkupElementInner::JsxFragment { .. } => {}
        }
    }
}
