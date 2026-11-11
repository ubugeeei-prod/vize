//! Markup facade helpers shared by accessibility rules.

use crate::markup::{MarkupBindingKind, MarkupElement, MarkupElementKind};

/// Static Boolean presence, excluding bindings that can override the value.
fn has_static_boolean(element: &MarkupElement<'_>, name: &str) -> bool {
    let mut present = false;
    element.walk_bindings(&mut |binding| {
        if binding.kind() == MarkupBindingKind::Attribute && binding.is_unqualified_arg_exact(name)
        {
            present = true;
        }
    });
    present && !has_unknown_binding(element, name)
}

fn has_unknown_binding(element: &MarkupElement<'_>, name: &str) -> bool {
    let mut unknown = false;
    element.walk_bindings(&mut |binding| {
        if binding.kind() == MarkupBindingKind::Bind
            && (binding
                .arg_name()
                .is_none_or(|arg| !binding.is_static_unqualified_arg_exact(arg))
                || binding.is_unqualified_arg_exact(name))
        {
            unknown = true;
        }
    });
    unknown
}

/// Only a physical element's static Boolean attribute establishes inertness.
pub(crate) fn has_static_inert(element: &MarkupElement<'_>) -> bool {
    element.kind() == MarkupElementKind::Element && has_static_boolean(element, "inert")
}

/// The HTML `hidden` attribute in its Hidden state (`hidden`, `hidden=""` or
/// `hidden="hidden"`) on a physical element: the element is not rendered and is
/// not in the accessibility tree. `hidden="until-found"` can be revealed, and a
/// binding (`:hidden` or `v-bind="attrs"`) can clear it, so neither counts.
pub(crate) fn has_static_hidden(element: &MarkupElement<'_>) -> bool {
    if element.kind() != MarkupElementKind::Element || has_unknown_binding(element, "hidden") {
        return false;
    }
    let mut hidden = false;
    element.walk_bindings(&mut |binding| {
        if binding.kind() == MarkupBindingKind::Attribute
            && binding.is_unqualified_arg_exact("hidden")
            && binding
                .static_value()
                .is_none_or(|value| value.is_empty() || value.eq_ignore_ascii_case("hidden"))
        {
            hidden = true;
        }
    });
    hidden
}

/// Component placement and modal-dialog state are unknown at lint time.
pub(crate) fn blocks_inert_inheritance(element: &MarkupElement<'_>) -> bool {
    matches!(
        element.kind(),
        MarkupElementKind::Component | MarkupElementKind::Slot
    ) || element.is_unqualified_tag_exact("dialog")
}

/// Narrow aria-hidden exclusions; other focus rules retain their old predicate.
pub(crate) fn is_statically_non_focusable(element: &MarkupElement<'_>) -> bool {
    if (element.is_unqualified_tag_exact("button")
        || element.is_unqualified_tag_exact("input")
        || element.is_unqualified_tag_exact("select")
        || element.is_unqualified_tag_exact("textarea"))
        && has_static_boolean(element, "disabled")
    {
        return true;
    }
    element.is_unqualified_tag_exact("input")
        && !has_unknown_binding(element, "type")
        && get_static_markup_attribute_value(element, "type")
            .is_some_and(|value| value.eq_ignore_ascii_case("hidden"))
}

/// Check if a markup facade element is natively interactive.
pub fn is_interactive_markup_element(element: &MarkupElement<'_>) -> bool {
    element.is_unqualified_tag_exact("a")
        || element.is_unqualified_tag_exact("button")
        || element.is_unqualified_tag_exact("input")
        || element.is_unqualified_tag_exact("select")
        || element.is_unqualified_tag_exact("textarea")
        || element.is_unqualified_tag_exact("details")
        || element.is_unqualified_tag_exact("summary")
        || element.is_unqualified_tag_exact("video")
        || element.is_unqualified_tag_exact("audio")
}

/// Check if a markup facade element is focusable (natively or via tabindex).
pub fn is_focusable_markup_element(element: &MarkupElement<'_>) -> bool {
    if let Some(tabindex) = get_static_markup_attribute_value(element, "tabindex") {
        if let Ok(val) = tabindex.parse::<i32>() {
            // `tabindex="-1"` is programmatically focusable. Lower values are not.
            return val >= -1;
        }
        return true;
    }

    if (element.is_unqualified_tag_exact("a") || element.is_unqualified_tag_exact("area"))
        && has_named_markup_prop(element, "href")
    {
        return true;
    }

    if element.is_unqualified_tag_exact("button")
        || element.is_unqualified_tag_exact("input")
        || element.is_unqualified_tag_exact("select")
        || element.is_unqualified_tag_exact("textarea")
        || element.is_unqualified_tag_exact("summary")
    {
        return true;
    }

    if let Some(val) = get_static_markup_attribute_value(element, "contenteditable")
        && val != "false"
    {
        return true;
    }

    false
}

fn has_named_markup_prop(element: &MarkupElement<'_>, name: &str) -> bool {
    let mut found = false;
    element.walk_bindings(&mut |binding| {
        if matches!(
            binding.kind(),
            MarkupBindingKind::Attribute | MarkupBindingKind::Bind
        ) && binding.is_static_unqualified_arg_exact(name)
        {
            found = true;
        }
    });
    found
}

/// Get the first static markup attribute value for an exact unqualified name.
///
/// Like `get_static_attribute_value`, a valueless first match returns `None`
/// without considering later duplicate attributes.
pub fn get_static_markup_attribute_value<'a>(
    element: &MarkupElement<'a>,
    name: &str,
) -> Option<&'a str> {
    let mut seen = false;
    let mut value = None;
    element.walk_bindings(&mut |binding| {
        if !seen
            && binding.kind() == MarkupBindingKind::Attribute
            && binding.is_unqualified_arg_exact(name)
        {
            seen = true;
            value = binding.static_value();
        }
    });
    value
}

/// HTML attribute names are ASCII case-insensitive. `ARIA-CHECKED` is
/// `aria-checked`, and `ROLE` is `role`.
pub fn get_static_or_bound_literal_markup_value_ignore_ascii_case<'a>(
    element: &MarkupElement<'a>,
    name: &str,
) -> Option<&'a str> {
    literal_markup_value(element, name, true)
}

/// `<div is="vue:name">` renders that component, not the native tag.
pub(crate) fn is_vue_is_component(element: &MarkupElement<'_>) -> bool {
    get_static_or_bound_literal_markup_value_ignore_ascii_case(element, "is")
        .is_some_and(|value| value.starts_with("vue:"))
}

/// First exact attribute or literal-valued bind, preserving authored order.
/// As on the template path, a matching static attribute ends the search even
/// when valueless; a nonliteral bind does not hide a later literal binding.
pub fn get_static_or_bound_literal_markup_value<'a>(
    element: &MarkupElement<'a>,
    name: &str,
) -> Option<&'a str> {
    literal_markup_value(element, name, false)
}

fn literal_markup_value<'a>(
    element: &MarkupElement<'a>,
    name: &str,
    ignore_ascii_case: bool,
) -> Option<&'a str> {
    let mut seen = false;
    let mut value = None;
    element.walk_bindings(&mut |binding| {
        let matches_name = if ignore_ascii_case {
            binding.is_unqualified_arg_eq_ignore_ascii_case(name)
        } else {
            binding.is_unqualified_arg_exact(name)
        };
        if seen || !matches_name {
            return;
        }
        match binding.kind() {
            MarkupBindingKind::Attribute => {
                seen = true;
                value = binding.static_value();
            }
            MarkupBindingKind::Bind => {
                if let Some(literal) = binding
                    .expression()
                    .and_then(super::helpers::string_literal_value)
                {
                    seen = true;
                    value = Some(literal);
                }
            }
            _ => {}
        }
    });
    value
}

/// Whether a static attribute or statically named bind is present.
pub fn has_named_markup_attribute_or_bind(element: &MarkupElement<'_>, name: &str) -> bool {
    has_named_markup_prop(element, name)
}

/// Components, slots, templates and non-native tags render unknown DOM.
pub(crate) fn is_component_like_markup_element(element: &MarkupElement<'_>) -> bool {
    matches!(
        element.kind(),
        MarkupElementKind::Component | MarkupElementKind::Slot | MarkupElementKind::Template
    ) || !vize_l0::is_native_tag(element.tag())
}

/// `role="presentation"` / `role="none"` opts the element out.
pub(crate) fn is_presentation_markup_role(element: &MarkupElement<'_>) -> bool {
    matches!(
        get_static_markup_attribute_value(element, "role"),
        Some("presentation" | "none")
    )
}

/// A static or literal `aria-hidden="true"`, or a bind whose expression is `true`.
pub(crate) fn is_aria_hidden_true_markup(element: &MarkupElement<'_>) -> bool {
    if get_static_or_bound_literal_markup_value(element, "aria-hidden") == Some("true") {
        return true;
    }
    let mut hidden = false;
    element.walk_bindings(&mut |binding| {
        hidden |= binding.kind() == MarkupBindingKind::Bind
            && binding.is_static_unqualified_arg_exact("aria-hidden")
            && binding.expression() == Some("true");
    });
    hidden
}

/// Whether a static-argument `v-on` handler for one of `events` is present.
pub(crate) fn has_static_markup_event(element: &MarkupElement<'_>, events: &[&str]) -> bool {
    let mut found = false;
    element.walk_bindings(&mut |binding| {
        found |= binding.kind() == MarkupBindingKind::On
            && events
                .iter()
                .any(|event| binding.is_static_unqualified_arg_exact(event))
            && (binding.expression().is_some() || binding.retained_expression().is_some());
    });
    found
}

/// Whether a real handler is present across direct JSX and lowered projections.
pub(crate) fn has_static_markup_handler(element: &MarkupElement<'_>, events: &[&str]) -> bool {
    let mut found = false;
    element.walk_bindings(&mut |binding| {
        found |= events
            .iter()
            .any(|event| binding.is_static_event_handler_for(event))
            && (binding.expression().is_some() || binding.retained_expression().is_some());
    });
    found
}
