//! [`MarkupElement`]'s hot queries for every backend, kept out of line: the
//! public query inlines only its Relief arm (today's template lint lane) into
//! each rule and calls here for the rest.

use super::queries::{is_lint_component, s2_template_is_special};
use super::{MarkupElement, MarkupElementInner};
use crate::ir::ByteRange;
use crate::markup::jsx_names::{
    jsx_element_kind, jsx_element_name, jsx_element_ref, jsx_fragment_ref,
};
use crate::markup::s2::S2ElementOp;
use crate::markup::{MarkupElementKind, relief_scopes, span_to_range};
use vize_relief::ElementType;

/// A Relief element's classification.
#[inline]
pub(super) fn relief_kind(tag_type: ElementType) -> MarkupElementKind {
    match tag_type {
        ElementType::Element => MarkupElementKind::Element,
        ElementType::Component => MarkupElementKind::Component,
        ElementType::Slot => MarkupElementKind::Slot,
        ElementType::Template => MarkupElementKind::Template,
    }
}

impl<'a> MarkupElement<'a> {
    #[inline(never)]
    pub(super) fn projected_tag(&self) -> &str {
        match self.inner {
            MarkupElementInner::Relief(node) => node.tag,
            MarkupElementInner::JsxElement { node, .. } => {
                jsx_element_name(&jsx_element_ref(node).opening_element.name)
            }
            MarkupElementInner::JsxFragment { .. } => "",
            MarkupElementInner::S2 { op, .. } => op.tag(),
            MarkupElementInner::S2Carrier { element, .. } => element.tag(),
        }
    }

    #[inline(never)]
    pub(super) fn projected_kind(&self) -> MarkupElementKind {
        match self.inner {
            MarkupElementInner::Relief(node) => relief_kind(node.tag_type),
            MarkupElementInner::JsxElement { node, .. } => {
                jsx_element_kind(&jsx_element_ref(node).opening_element.name)
            }
            MarkupElementInner::JsxFragment { .. } => MarkupElementKind::Template,
            MarkupElementInner::S2 { op, surface, .. } => match op {
                S2ElementOp::Slot(_) => MarkupElementKind::Slot,
                _ if op.tag() == "template" && s2_template_is_special(op, surface) => {
                    MarkupElementKind::Template
                }
                // A template classifies the way the lint-mode parse does: a
                // component is a core built-in or a capitalized tag. S2's
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
                S2ElementOp::Component(_) => MarkupElementKind::Component,
                S2ElementOp::Element(_) => MarkupElementKind::Element,
            },
            MarkupElementInner::S2Carrier { .. } => MarkupElementKind::Template,
        }
    }

    #[inline(never)]
    pub(super) fn projected_range(&self) -> ByteRange {
        match self.inner {
            MarkupElementInner::Relief(node) => relief_scopes::carrier_range(node),
            MarkupElementInner::JsxElement { node, offset } => {
                span_to_range(jsx_element_ref(node).span, offset)
            }
            MarkupElementInner::JsxFragment { node, offset } => {
                span_to_range(jsx_fragment_ref(node).span, offset)
            }
            // An implicit table owner (`tbody` / `tr`) is zero-width at the
            // tag name of the row or cell that opened it, as the parser's
            // tree construction records it.
            MarkupElementInner::S2 { op, doc, .. } if op.is_synthesized(doc) => {
                let at = op.span().start + 1;
                ByteRange::new(at, at)
            }
            MarkupElementInner::S2 { op, doc, .. } => doc.open_tag_range(op.span()),
            MarkupElementInner::S2Carrier { span, doc, .. } => doc.open_tag_range(span),
        }
    }
}
