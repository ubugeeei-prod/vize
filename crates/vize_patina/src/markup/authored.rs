//! Source-shaped L1 projection retained by the L2 markup document.
//!
//! L2 may introduce table owners and the normal L1 tree may implicitly close
//! nested interactive tags. Content-model consumers instead read the authored
//! parent/child relation, through the same typed element facade and decoders.

use super::element::queries::is_lint_component;
use super::element::{MarkupElement, MarkupElementInner};
use super::l1::L1Markup;
use super::l2::binding::{is_consumed_spelling, is_v_pre};
use super::l2::surface::{SurfaceDirective, offset_in, token_range};
use super::{MarkupAttribute, MarkupElementKind, MarkupNode, MarkupText};
use crate::ir::ByteRange;
use vize_l1::markup::{Attribute, Element, SurfaceChild};
use vize_l1_to_l2::lower::{children_ns, enter_ns};
use vize_l2::op::Namespace;

fn wrap<'a>(
    element: &'a Element<'a>,
    doc: &'a L1Markup<'a>,
    inherited: bool,
    parent_ns: Namespace,
) -> (MarkupElement<'a>, bool, Namespace) {
    let opens_v_pre = !inherited && element.open.attrs.iter().any(is_v_pre);
    let frozen = inherited || opens_v_pre;
    let ns = enter_ns(parent_ns, element.tag());
    let node = MarkupElement::from_inner(MarkupElementInner::Authored {
        element,
        doc,
        frozen,
        opens_v_pre,
        ns,
    });
    (node, frozen, ns)
}

pub(super) fn walk_tree<'a>(
    children: &'a [SurfaceChild<'a>],
    doc: &'a L1Markup<'a>,
    frozen: bool,
    enter: &mut impl FnMut(MarkupElement<'a>),
    exit: &mut impl FnMut(MarkupElement<'a>),
) {
    walk_elements(children, doc, frozen, Namespace::Html, enter, exit);
}

fn walk_elements<'a>(
    children: &'a [SurfaceChild<'a>],
    doc: &'a L1Markup<'a>,
    frozen: bool,
    ns: Namespace,
    enter: &mut impl FnMut(MarkupElement<'a>),
    exit: &mut impl FnMut(MarkupElement<'a>),
) {
    for child in children {
        if let SurfaceChild::Element(element) = child {
            let (node, frozen, ns) = wrap(element, doc, frozen, ns);
            enter(node);
            vize_l0::ensure_sufficient_stack(|| {
                walk_elements(
                    &element.children,
                    doc,
                    frozen,
                    children_ns(ns, element.tag()),
                    enter,
                    exit,
                )
            });
            exit(node);
        }
    }
}

pub(super) fn kind(element: &Element<'_>, frozen: bool, opens_v_pre: bool) -> MarkupElementKind {
    let tag = element.tag();
    if tag == "slot" {
        MarkupElementKind::Slot
    } else if tag == "template"
        && (!frozen || opens_v_pre)
        && element.open.attrs.iter().any(|attr| {
            SurfaceDirective::parse(attr.name.text)
                .is_some_and(|directive| directive.is_structural() || directive.name == "slot")
        })
    {
        MarkupElementKind::Template
    } else if is_lint_component(tag) {
        MarkupElementKind::Component
    } else {
        MarkupElementKind::Element
    }
}

pub(super) fn range(element: &Element<'_>, doc: &L1Markup<'_>) -> ByteRange {
    let start = offset_in(doc.source, element.open.lt_name.text);
    let end = token_range(doc.source, &element.open.gt).end;
    ByteRange::new(start, end)
}

/// The spelling of a directive the element keeps (not consumed into a scope).
pub(super) fn directive<'a>(attr: &Attribute<'a>) -> Option<SurfaceDirective<'a>> {
    SurfaceDirective::parse(attr.name.text).filter(|_| !is_consumed_spelling(attr))
}

/// An attribute the element keeps as a binding, with its directive
/// spelling (none when frozen); `None` when the facade consumes it.
pub(super) fn kept<'a>(
    attr: &Attribute<'a>,
    frozen: bool,
    opens_v_pre: bool,
) -> Option<Option<SurfaceDirective<'a>>> {
    if frozen {
        // Frozen structural spellings are plain attrs and do not become scopes.
        (!(opens_v_pre && is_v_pre(attr))).then_some(None)
    } else {
        let directive = SurfaceDirective::parse(attr.name.text);
        let consumed = is_v_pre(attr) || directive.is_some_and(|spelling| spelling.is_structural());
        (!consumed).then_some(directive)
    }
}

pub(super) fn walk_attributes<'a>(
    element: &'a Element<'a>,
    doc: &'a L1Markup<'a>,
    frozen: bool,
    opens_v_pre: bool,
    visitor: &mut impl FnMut(MarkupAttribute<'a>),
) {
    for attr in &element.open.attrs {
        // A kept attribute is static when frozen or when it spells no directive.
        if kept(attr, frozen, opens_v_pre).is_some_and(|directive| frozen || directive.is_none()) {
            let name = frozen_name(attr, doc, opens_v_pre);
            visitor(MarkupAttribute::from_authored(attr, doc, name));
        }
    }
}

pub(super) fn walk_children<'a>(
    element: &'a Element<'a>,
    doc: &'a L1Markup<'a>,
    frozen: bool,
    ns: Namespace,
    visitor: &mut impl FnMut(MarkupNode<'a>),
) {
    let child_ns = children_ns(ns, element.tag());
    for child in &element.children {
        let node = match child {
            SurfaceChild::Element(element) => {
                MarkupNode::Element(wrap(element, doc, frozen, child_ns).0)
            }
            SurfaceChild::Text(token) => MarkupNode::Text(MarkupText::from_static(
                doc.decode_text(token.text),
                token_range(doc.source, token),
            )),
            SurfaceChild::Interpolation(node) => {
                let start = offset_in(doc.source, node.open.text);
                let end = token_range(doc.source, &node.close).end;
                let range = ByteRange::new(start, end);
                if frozen {
                    let text = doc.source.get(start as usize..end as usize).unwrap_or("");
                    MarkupNode::Text(MarkupText::from_static(text, range))
                } else {
                    MarkupNode::Interpolation(range)
                }
            }
            SurfaceChild::Comment(token) => MarkupNode::Comment(token_range(doc.source, token)),
            SurfaceChild::Cdata(token) if ns != Namespace::Html => {
                let text = vize_l1_to_l2::lower::cdata_text(token.text);
                let start = offset_in(doc.source, text);
                MarkupNode::Text(MarkupText::from_static(
                    text,
                    ByteRange::new(start, start + text.len() as u32),
                ))
            }
            // Stray tags and PIs do not render. CDATA in HTML is a parse
            // diagnostic handled by the retained template parse, not text.
            SurfaceChild::Unexpected(_)
            | SurfaceChild::Cdata(_)
            | SurfaceChild::ProcessingInstruction(_) => continue,
        };
        visitor(node);
    }
}

/// Only the opening element was tokenized before v-pre took effect.
/// Descendant spellings keep their authored names unchanged.
pub(super) fn frozen_name<'a>(
    attr: &'a Attribute<'a>,
    doc: &'a L1Markup<'a>,
    opens_v_pre: bool,
) -> &'a str {
    if opens_v_pre {
        doc.frozen_attribute_name(attr.name.text)
    } else {
        attr.name.text
    }
}
