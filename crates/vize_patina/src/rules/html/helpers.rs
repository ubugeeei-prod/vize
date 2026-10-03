//! Shared helper functions for HTML conformance rules.

use vize_relief::{ElementNode, ElementType, TemplateChildNode};

pub use crate::tag_policy::DEPRECATED_ELEMENTS;

pub use crate::attribute_policy::{deprecated_attr_suggestion, deprecated_attr_suggestion_by_tag};

/// Boolean HTML attributes that should not have explicit values
pub const BOOLEAN_ATTRIBUTES: &[&str] = &[
    "allowfullscreen",
    "async",
    "autofocus",
    "autoplay",
    "checked",
    "controls",
    "default",
    "defer",
    "disabled",
    "formnovalidate",
    "hidden",
    "inert",
    "ismap",
    "itemscope",
    "loop",
    "multiple",
    "muted",
    "nomodule",
    "novalidate",
    "open",
    "playsinline",
    "readonly",
    "required",
    "reversed",
    "selected",
];

/// Elements that expect palpable (visible) content per HTML spec
pub const PALPABLE_CONTENT_ELEMENTS: &[&str] = &[
    "p",
    "li",
    "dt",
    "dd",
    "th",
    "td",
    "figcaption",
    "summary",
    "legend",
    "caption",
    "label",
    "option",
];

/// Check if element has any visible (palpable) content
pub fn has_palpable_content(element: &ElementNode) -> bool {
    for child in &element.children {
        match child {
            TemplateChildNode::Text(text) if !text.content.trim().is_empty() => return true,
            TemplateChildNode::Interpolation(_) => return true,
            TemplateChildNode::Element(el) if el.tag_type == ElementType::Template => {
                if has_palpable_content(el) {
                    return true;
                }
            }
            TemplateChildNode::Element(_) => return true,
            _ => {}
        }
    }
    false
}

/// Simple datetime format validation for `<time>` element.
/// Accepts common ISO 8601 subsets: dates, times, datetimes, durations.
pub fn is_valid_datetime(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }

    // Duration: P1D, PT1H30M, P1Y2M3DT4H5M6S
    if let Some(rest) = s.strip_prefix('P') {
        return rest
            .chars()
            .all(|c| c.is_ascii_digit() || "YMWDTHS.".contains(c));
    }

    // Must contain at least one digit
    if !s.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }

    // Valid chars for datetime strings: digits, -, :, T, Z, +, ., W, space
    s.chars()
        .all(|c| c.is_ascii_digit() || "-:TtZz+. W".contains(c))
}
