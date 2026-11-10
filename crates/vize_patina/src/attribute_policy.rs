//! Shared exact authored attribute policy for HTML conformance rules.

mod img_alt;
pub(crate) use img_alt::{ImageAltMessages, ImageAltPolicy};

/// Match the JavaScript numeric coercion used by eslint-plugin-vuejs-accessibility.
pub fn positive_tabindex(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return false;
    }
    for (prefix, radix) in [
        ("0x", 16),
        ("0X", 16),
        ("0o", 8),
        ("0O", 8),
        ("0b", 2),
        ("0B", 2),
    ] {
        if let Some(digits) = value.strip_prefix(prefix) {
            return !digits.is_empty()
                && digits.chars().all(|digit| digit.is_digit(radix))
                && digits.chars().any(|digit| digit != '0');
        }
    }
    if matches!(value, "Infinity" | "+Infinity") {
        return true;
    }
    if value
        .bytes()
        .any(|byte| byte.is_ascii_alphabetic() && !matches!(byte, b'e' | b'E'))
    {
        return false;
    }
    value.parse::<f64>().is_ok_and(|number| number > 0.0)
}

/// Evaluate only a JavaScript numeric or string literal used by `v-bind:tabindex`.
pub(crate) fn positive_bound_tabindex_literal(expression: &str) -> bool {
    let expression = expression.trim();
    if expression.len() >= 2
        && let Some(quote) = expression.as_bytes().first().copied()
        && matches!(quote, b'\'' | b'"' | b'`')
        && expression.as_bytes().last() == Some(&quote)
    {
        return expression
            .get(1..expression.len() - 1)
            .is_some_and(positive_tabindex);
    }

    let mut characters = expression.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    let numeric_start = first.is_ascii_digit()
        || first == '.'
        || matches!(first, '+' | '-')
            && characters
                .next()
                .is_some_and(|next| next.is_ascii_digit() || next == '.');
    numeric_start && positive_tabindex(expression)
}

#[cfg(test)]
mod tests {
    use super::{positive_bound_tabindex_literal, positive_tabindex};

    #[test]
    fn tabindex_values_follow_javascript_numeric_coercion() {
        for value in [
            "1",
            " 1 ",
            "1.5",
            ".5",
            "1e1",
            "0x1",
            "0o10",
            "0b10",
            "Infinity",
            "+Infinity",
        ] {
            assert!(positive_tabindex(value), "{value}");
        }
        for value in [
            "", "0", "-1", "inf", "+inf", "infinity", "NaN", "+0x1", "1e", "１",
        ] {
            assert!(!positive_tabindex(value), "{value}");
        }
    }

    #[test]
    fn bound_tabindex_only_evaluates_literals() {
        for value in ["2", "2.5", "'2'", "\" 2 \"", "`0x2`"] {
            assert!(positive_bound_tabindex_literal(value), "{value}");
        }
        for value in ["value", "Infinity", "`${value}`", "'0'", "-1"] {
            assert!(!positive_bound_tabindex_literal(value), "{value}");
        }
    }
}

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
    // Vue's built-in `<component>` is never an HTML element.
    if is_tag("component") {
        return None;
    }
    match attr {
        "align" => Some("CSS `text-align` or `margin: auto`"),
        "bgcolor" => Some("CSS `background-color`"),
        // HTML only ever defined `border` on these; elsewhere it is an
        // unknown attribute (UnoCSS attributify's `border="b base"`).
        "border" if is_tag("img") || is_tag("object") => Some("CSS `border`"),
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
