//! Markup facade helpers shared by accessibility rules.

use crate::markup::{MarkupBindingKind, MarkupElement};

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

/// First exact attribute or literal-valued bind, preserving authored order.
/// As on the template path, a matching static attribute ends the search even
/// when valueless; a nonliteral bind does not hide a later literal binding.
pub fn get_static_or_bound_literal_markup_value<'a>(
    element: &MarkupElement<'a>,
    name: &str,
) -> Option<&'a str> {
    let mut seen = false;
    let mut value = None;
    element.walk_bindings(&mut |binding| {
        if seen || !binding.is_unqualified_arg_exact(name) {
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
