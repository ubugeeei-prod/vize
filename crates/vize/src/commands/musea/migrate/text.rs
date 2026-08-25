//! Escaping helpers for emitting valid `.art.vue` output.

use vize_s0::String;

/// HTML-escape a value for placement inside a double-quoted attribute, so an
/// expression or string containing `"` (e.g. `onClick={() => alert("hi")}`)
/// cannot break out of the attribute. Vue decodes the entities before parsing
/// the binding expression, so the round-trip is lossless.
pub(super) fn escape_attr(value: &str) -> String {
    let mut out = String::default();
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            other => out.push(other),
        }
    }
    out
}

/// Quote a Vue directive expression without HTML-escaping the expression's own
/// string literals. Keep the existing double-quoted attribute form when
/// possible; use single quotes only when the expression itself contains `"`.
pub(super) fn quote_directive_expression(value: &str) -> Option<String> {
    if !value.contains('"') {
        let mut out = String::from("\"");
        out.push_str(value);
        out.push('"');
        return Some(out);
    }

    if !value.contains('\'') {
        let mut out = String::from("'");
        out.push_str(value);
        out.push('\'');
        return Some(out);
    }

    None
}

/// Escape a value for a double-quoted JavaScript/TypeScript string literal (used
/// in the generated `defineArt("...", { ... })` call).
pub(super) fn escape_js_string(value: &str) -> String {
    let mut out = String::default();
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{escape_attr, escape_js_string, quote_directive_expression};

    #[test]
    fn escape_attr_encodes_quotes_and_amp() {
        assert_eq!(
            escape_attr("() => alert(\"hi\" & bye)").as_str(),
            "() => alert(&quot;hi&quot; &amp; bye)"
        );
    }

    #[test]
    fn escape_attr_leaves_plain_text() {
        assert_eq!(escape_attr("1 + 2").as_str(), "1 + 2");
    }

    #[test]
    fn escape_js_string_encodes_backslash_quote_newline() {
        assert_eq!(escape_js_string("a\\b\"c\nd").as_str(), "a\\\\b\\\"c\\nd");
    }

    #[test]
    fn quote_directive_expression_preserves_double_quotes() {
        assert_eq!(
            quote_directive_expression(r#"{ name: "students" }"#)
                .unwrap()
                .as_str(),
            r#"'{ name: "students" }'"#
        );
    }

    #[test]
    fn quote_directive_expression_uses_double_quotes_for_single_quoted_text() {
        assert_eq!(
            quote_directive_expression("{ name: 'students' }")
                .unwrap()
                .as_str(),
            r#""{ name: 'students' }""#
        );
    }
}
