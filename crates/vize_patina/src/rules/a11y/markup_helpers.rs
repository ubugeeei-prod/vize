//! Markup facade helpers shared by accessibility rules.

use crate::markup::{MarkupBindingKind, MarkupElement};

/// Check if a markup facade element is focusable (natively or via tabindex).
pub fn is_focusable_markup_element(element: &MarkupElement<'_>) -> bool {
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

    if let Some(tabindex) = get_static_markup_attribute_value(element, "tabindex") {
        if let Ok(val) = tabindex.parse::<i32>() {
            return val >= 0;
        }
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
