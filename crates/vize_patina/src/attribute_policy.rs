//! Shared exact authored attribute policy for HTML conformance rules.

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
