//! Source-shaped L1 projection retained by the L2 markup document.
//!
//! L2 may introduce table owners and the normal L1 tree may implicitly close
//! nested interactive tags. Content-model consumers instead read the authored
//! parent/child relation, through the same typed element facade and decoders.

use super::element::queries::is_lint_component;
use super::element::{MarkupElement, MarkupElementInner};
use super::l2::L2Markup;
use super::l2::binding::is_consumed_spelling;
use super::l2::surface::{SurfaceDirective, offset_in, token_range};
use super::{MarkupAttribute, MarkupElementKind, MarkupNode, MarkupText};
use crate::ir::ByteRange;
use vize_l1::{Attribute, Element, SurfaceChild};
use vize_relief::Namespace;

fn wrap<'a>(
    element: &'a Element<'a>,
    doc: &'a L2Markup<'a>,
    inherited: bool,
    parent_ns: Namespace,
) -> (MarkupElement<'a>, bool, Namespace) {
    let opens_v_pre = !inherited
        && element
            .open
            .attrs
            .iter()
            .any(|attr| attr.name.text == "v-pre");
    let frozen = inherited || opens_v_pre;
    let ns = if vize_l0::is_svg_tag(element.tag()) {
        Namespace::Svg
    } else if vize_l0::is_math_ml_tag(element.tag()) {
        Namespace::MathMl
    } else {
        parent_ns
    };
    let node = MarkupElement::from_inner(MarkupElementInner::Authored {
        element,
        doc,
        frozen,
        opens_v_pre,
        ns,
    });
    (node, frozen, ns)
}

fn children_ns(ns: Namespace, tag: &str) -> Namespace {
    match ns {
        Namespace::Svg if matches!(tag, "foreignObject" | "desc" | "title") => Namespace::Html,
        Namespace::MathMl
            if matches!(tag, "annotation-xml" | "mi" | "mo" | "mn" | "ms" | "mtext") =>
        {
            Namespace::Html
        }
        _ => ns,
    }
}

pub(super) fn walk_tree<'a>(
    children: &'a [SurfaceChild<'a>],
    doc: &'a L2Markup<'a>,
    frozen: bool,
    enter: &mut impl FnMut(MarkupElement<'a>),
    exit: &mut impl FnMut(MarkupElement<'a>),
) {
    walk_elements(children, doc, frozen, Namespace::Html, enter, exit);
}

fn walk_elements<'a>(
    children: &'a [SurfaceChild<'a>],
    doc: &'a L2Markup<'a>,
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

pub(super) fn kind(element: &Element<'_>, frozen: bool) -> MarkupElementKind {
    let tag = element.tag();
    if tag == "slot" {
        MarkupElementKind::Slot
    } else if tag == "template"
        && !frozen
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

pub(super) fn range(element: &Element<'_>, doc: &L2Markup<'_>) -> ByteRange {
    let start = offset_in(doc.source, element.open.lt_name.text);
    let end = token_range(doc.source, &element.open.gt).end;
    ByteRange::new(start, end)
}

pub(super) fn is_directive(attr: &Attribute<'_>) -> bool {
    SurfaceDirective::parse(attr.name.text).is_some() && !is_consumed_spelling(attr)
}

pub(super) fn consumed(attr: &Attribute<'_>, frozen: bool, opens_v_pre: bool) -> bool {
    // Frozen structural spellings are plain attrs and do not become scopes.
    if frozen {
        opens_v_pre && attr.name.text == "v-pre"
    } else {
        is_consumed_spelling(attr)
    }
}

pub(super) fn walk_attributes<'a>(
    element: &'a Element<'a>,
    doc: &'a L2Markup<'a>,
    frozen: bool,
    opens_v_pre: bool,
    visitor: &mut impl FnMut(MarkupAttribute<'a>),
) {
    for attr in &element.open.attrs {
        if !consumed(attr, frozen, opens_v_pre)
            && (frozen || SurfaceDirective::parse(attr.name.text).is_none())
        {
            let name = frozen_name(attr, doc, opens_v_pre);
            visitor(MarkupAttribute::from_authored(attr, doc, name));
        }
    }
}

pub(super) fn walk_children<'a>(
    element: &'a Element<'a>,
    doc: &'a L2Markup<'a>,
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
                doc.static_part_text(token.text),
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
                let text = token.text.strip_prefix("<![CDATA[").unwrap_or(token.text);
                let text = text.strip_suffix("]]>").unwrap_or(text);
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
    doc: &'a L2Markup<'a>,
    opens_v_pre: bool,
) -> &'a str {
    if opens_v_pre {
        doc.frozen_attribute_name(attr.name.text)
    } else {
        attr.name.text
    }
}
