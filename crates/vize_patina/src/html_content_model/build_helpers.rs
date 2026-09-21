//! Element-level projections the skeleton builder reads from the markup
//! facade: component classification, slot names, attribute facts and the
//! namespace Vue's compiler assigns.

use vize_s0::{CompactString, Span};

use super::facts::{Attr, Ns};
use super::skeleton::{BoundaryKind, Element, NodeKind};
use crate::markup::{MarkupBindingKind, MarkupElement, MarkupElementKind};

/// Render-in-place built-ins: their children render where they stand.
pub(super) const TRANSPARENT_BUILTINS: &[&str] = &[
    "Transition",
    "transition",
    "BaseTransition",
    "base-transition",
    "KeepAlive",
    "keep-alive",
    "Suspense",
    "suspense",
];

/// The namespace Vue's compiler gives an element (`getNamespace`), from its
/// parent's namespace, tag and HTML-encoding flag; a template root takes the
/// namespace its tag names.
pub(super) fn compiler_ns(tag: &str, parent: Option<(Ns, &str, bool)>) -> Ns {
    let Some((parent_ns, parent_tag, encoding_html)) = parent else {
        return if vize_s0::is_svg_tag(tag) {
            Ns::Svg
        } else if vize_s0::is_math_ml_tag(tag) {
            Ns::MathMl
        } else {
            Ns::Html
        };
    };
    let mut ns = parent_ns;
    match parent_ns {
        Ns::MathMl if parent_tag == "annotation-xml" => {
            if tag == "svg" {
                return Ns::Svg;
            }
            if encoding_html {
                ns = Ns::Html;
            }
        }
        Ns::MathMl
            if matches!(parent_tag, "mi" | "mo" | "mn" | "ms" | "mtext")
                && !matches!(tag, "mglyph" | "malignmark") =>
        {
            ns = Ns::Html;
        }
        Ns::Svg if matches!(parent_tag, "foreignObject" | "desc" | "title") => ns = Ns::Html,
        _ => {}
    }
    match (ns, tag) {
        (Ns::Html, "svg") => Ns::Svg,
        (Ns::Html, "math") => Ns::MathMl,
        _ => ns,
    }
}

/// Component namespaces whose `namespace.tag` members render the intrinsic
/// HTML element `tag` (motion-v's `<motion.div>`).
const INTRINSIC_MEMBER_COMPONENT_NAMESPACES: &[&str] = &["motion"];

pub(super) fn intrinsic_member_tag(tag: &str) -> Option<&str> {
    let (namespace, member) = tag.split_once('.')?;
    (INTRINSIC_MEMBER_COMPONENT_NAMESPACES.contains(&namespace) && vize_s0::is_html_tag(member))
        .then_some(member)
}

pub(super) fn name_span(start: u32, tag: &str) -> Span {
    Span::new(start + 1, start + 1 + tag.len() as u32)
}

/// The node a component tag opens; `None` for render-in-place built-ins
/// (and `TransitionGroup`, handled by the caller).
pub(super) fn component_kind(element: &MarkupElement<'_>) -> Option<NodeKind> {
    let tag = element.tag();
    if TRANSPARENT_BUILTINS.contains(&tag) || matches!(tag, "TransitionGroup" | "transition-group")
    {
        return None;
    }
    if matches!(tag, "Teleport" | "teleport") {
        let disabled = element
            .static_attribute("disabled")
            .is_some_and(|attr| attr.value().is_none_or(|value| value != "false"));
        return if disabled {
            None
        } else {
            Some(NodeKind::Boundary(BoundaryKind::Teleport))
        };
    }
    if matches!(tag, "component" | "Component") {
        return Some(NodeKind::Boundary(BoundaryKind::DynamicComponent));
    }
    Some(NodeKind::Component {
        name: CompactString::new(tag),
    })
}

/// `<x is="vue:…">` or a bound `is`: Vue resolves a component at runtime.
pub(super) fn is_dynamic_is(element: &MarkupElement<'_>) -> bool {
    element.has_bound_attribute("is")
        || element
            .static_attribute("is")
            .and_then(|attr| attr.value())
            .is_some_and(|value| value.starts_with("vue:"))
}

/// A static attribute's value (`Some(Some(v))`), a bound one (`Some(None)`),
/// or absent (`None`).
pub(super) fn static_name(
    element: &MarkupElement<'_>,
    attr: &str,
) -> Option<Option<CompactString>> {
    if let Some(found) = element.static_attribute(attr) {
        return Some(found.value().map(CompactString::new));
    }
    element.has_bound_attribute(attr).then_some(None)
}

/// `<template #name>` / `<template v-slot:name>`: `Some(Some(name))`,
/// `Some(None)` for a dynamic slot name, `None` when not a slot template.
pub(super) fn slot_template_name(element: &MarkupElement<'_>) -> Option<Option<CompactString>> {
    if element.kind() != MarkupElementKind::Template {
        return None;
    }
    let mut found = None;
    element.walk_directives(&mut |directive| {
        if directive.name() == "slot" {
            found = Some(match directive.arg_name() {
                None => Some(CompactString::new("default")),
                Some(arg) if arg.starts_with('[') => None,
                Some(arg) => Some(CompactString::new(arg)),
            });
        }
    });
    found
}

pub(super) fn element_node(element: &MarkupElement<'_>, tag: &str, start: u32) -> Element {
    let mut node = Element::new(tag, name_span(start, tag));
    element.walk_bindings(&mut |binding| {
        let kind = binding.kind();
        let arg = binding.arg_name();
        match (kind, arg) {
            (MarkupBindingKind::Attribute, Some(name)) => {
                if let Some(attr) = attr_of(name, binding.static_value()) {
                    node.attrs.set_yes(attr);
                }
            }
            (MarkupBindingKind::Bind, Some(name)) => {
                if matches!(
                    name,
                    "innerHTML" | "inner-html" | "textContent" | "text-content"
                ) {
                    node.dynamic_content = true;
                } else if let Some(attr) = bound_attr(name) {
                    node.attrs.set_maybe(attr);
                }
            }
            (MarkupBindingKind::Bind, None) => node.attrs.set_all_maybe(),
            (MarkupBindingKind::Custom, Some("html" | "text")) => node.dynamic_content = true,
            _ => {}
        }
    });
    node
}

/// The attribute fact a static attribute establishes.
fn attr_of(name: &str, value: Option<&str>) -> Option<Attr> {
    match name {
        "href" => Some(Attr::Href),
        "controls" => Some(Attr::Controls),
        "usemap" => Some(Attr::Usemap),
        "itemprop" => Some(Attr::Itemprop),
        "tabindex" => Some(Attr::Tabindex),
        "type" => value
            .is_some_and(|value| value.eq_ignore_ascii_case("hidden"))
            .then_some(Attr::TypeHidden),
        "color" | "face" | "size" => Some(Attr::FontPresentational),
        "encoding" => value
            .is_some_and(|value| {
                value.eq_ignore_ascii_case("text/html")
                    || value.eq_ignore_ascii_case("application/xhtml+xml")
            })
            .then_some(Attr::EncodingHtml),
        _ => None,
    }
}

/// The attribute fact a bound attribute makes unknown.
fn bound_attr(name: &str) -> Option<Attr> {
    match name {
        "type" => Some(Attr::TypeHidden),
        "encoding" => Some(Attr::EncodingHtml),
        other => attr_of(other, Some("hidden")),
    }
}
