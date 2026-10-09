//! Borrowed element identity and markup namespace selection.
use super::{Analyzed, AttrForm, Cx, Head, attr_slice, attr_span, classify};
use vize_l0::{String, is_math_ml_tag, is_svg_tag};
use vize_l1::{Attribute, Element, embed::prepare_attribute_value};
use vize_l2::op::Namespace;

pub(super) fn vue_is<'a>(
    cx: &Cx<'a>,
    element: &Element<'a>,
    analyzed: &Analyzed<'a>,
) -> Option<(usize, &'a str)> {
    let ordinal = analyzed.static_is?;
    if cx.v_pre_suppressed()
        || analyzed.opens_v_pre
        || cx.is_custom_element(element.tag())
        || matches!(element.tag(), "component" | "Component" | "slot")
        || element.tag() == "template"
            && (analyzed.branch.is_some()
                || analyzed.vfor.is_some()
                || analyzed.has_slot_spelling())
    {
        return None;
    }
    let attribute = element.open.attrs.get(ordinal)?;
    let value = attribute.value.as_ref()?;
    let decoded =
        prepare_attribute_value(cx.allocator, cx.source, cx.token_span(&value.content)).ok()?;
    Some((ordinal, decoded.text().strip_prefix("vue:")?))
}

/// Table owner admission must see the cast before minting synthetic wrappers.
pub(in crate::lower) fn casts_component(cx: &Cx<'_>, element: &Element<'_>) -> bool {
    if cx.v_pre_suppressed() || element.open.is_verbatim() || cx.is_custom_element(element.tag()) {
        return false;
    }
    let mut selector = None;
    for attribute in &element.open.attrs {
        if attribute.name.text.starts_with("v-pre")
            && matches!(classify(attribute.name.text), AttrForm::Directive(directive) if directive.head == Head::Pre)
        {
            return false;
        }
        if selector.is_none() && attribute.name.text == "is" {
            selector = Some(attribute);
        }
    }
    let Some(attribute) = selector else {
        return false;
    };
    let Some(value) = &attribute.value else {
        return false;
    };
    prepare_attribute_value(cx.allocator, cx.source, cx.token_span(&value.content))
        .is_ok_and(|decoded| decoded.text().starts_with("vue:"))
}

pub(super) fn consume(cx: &mut Cx<'_>, attribute: &Attribute<'_>) {
    cx.record(
        "drop.vue-is",
        None,
        attr_slice(cx, attribute),
        String::default(),
        attr_span(cx, attribute),
    );
}

/// An element's own namespace, entered by tag.
pub(super) fn enter_ns(parent: Namespace, tag: &str) -> Namespace {
    if is_svg_tag(tag) {
        Namespace::Svg
    } else if is_math_ml_tag(tag) {
        Namespace::MathMl
    } else {
        parent
    }
}

/// The namespace an element's children live in (the integration points
/// return to HTML).
pub(super) fn children_ns(own: Namespace, tag: &str) -> Namespace {
    match own {
        Namespace::Svg if matches!(tag, "foreignObject" | "desc" | "title") => Namespace::Html,
        Namespace::MathMl
            if matches!(tag, "annotation-xml" | "mi" | "mo" | "mn" | "ms" | "mtext") =>
        {
            Namespace::Html
        }
        other => other,
    }
}
