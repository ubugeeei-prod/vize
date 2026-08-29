//! Shared helper functions for HTML conformance rules.

use vize_relief::{ElementNode, ElementType, TemplateChildNode};

/// Deprecated HTML elements per the Living Standard
pub const DEPRECATED_ELEMENTS: &[&str] = &[
    "acronym",
    "applet",
    "basefont",
    "bgsound",
    "big",
    "blink",
    "center",
    "dir",
    "font",
    "frame",
    "frameset",
    "isindex",
    "keygen",
    "listing",
    "marquee",
    "menuitem",
    "multicol",
    "nextid",
    "nobr",
    "noembed",
    "noframes",
    "plaintext",
    "rb",
    "rtc",
    "spacer",
    "strike",
    "tt",
    "xmp",
];

/// Returns a CSS replacement suggestion if the attribute is deprecated on the given element.
pub fn deprecated_attr_suggestion(element: &str, attr: &str) -> Option<&'static str> {
    deprecated_attr_suggestion_by_tag(attr, |tag| element == tag)
}

/// Returns a CSS replacement suggestion for a deprecated attribute.
///
/// The tag predicate lets zero-copy frontends preserve exact tag semantics
/// without first allocating a normalized tag string. This matters for JSX
/// namespaced tags: the legacy lowering sees `svg:table`, which is not exactly
/// `table`, so table-only exceptions such as `border` must not accidentally
/// apply to the local name.
pub fn deprecated_attr_suggestion_by_tag(
    attr: &str,
    mut is_tag: impl FnMut(&str) -> bool,
) -> Option<&'static str> {
    match attr {
        "align" => Some("CSS `text-align` or `margin: auto`"),
        "bgcolor" => Some("CSS `background-color`"),
        "border" if !is_tag("table") => Some("CSS `border`"),
        "background" if is_tag("body") => Some("CSS `background-image`"),
        "text" if is_tag("body") => Some("CSS `color`"),
        "link" | "vlink" | "alink" if is_tag("body") => Some("CSS `:link`, `:visited`"),
        "cellpadding" if is_tag("table") => Some("CSS `padding` on cells"),
        "cellspacing" if is_tag("table") => Some("CSS `border-spacing`"),
        "width" | "height" if is_tag("td") || is_tag("th") => Some("CSS `width`/`height`"),
        "valign" if is_tag("td") || is_tag("th") => Some("CSS `vertical-align`"),
        "nowrap" if is_tag("td") || is_tag("th") => Some("CSS `white-space: nowrap`"),
        "hspace" | "vspace" if is_tag("img") => Some("CSS `margin`"),
        "clear" if is_tag("br") => Some("CSS `clear`"),
        "noshade" | "size" | "width" | "color" if is_tag("hr") => Some("CSS styling"),
        "type" if is_tag("li") || is_tag("ul") => Some("CSS `list-style-type`"),
        "width" if is_tag("pre") => Some("CSS `width`"),
        _ => None,
    }
}

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
            TemplateChildNode::Element(el) if el.tag_type != ElementType::Template => return true,
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
