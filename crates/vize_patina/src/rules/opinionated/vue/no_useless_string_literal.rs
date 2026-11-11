/// Return the authored body when `raw` is exactly one static string literal.
///
/// Like the parsed expression of the reference rule, comments and wrapping
/// parentheses around the literal are not part of it. The scanner is
/// delimiter-aware so a second template literal or an unescaped
/// interpolation opener cannot be mistaken for part of the first.
pub(super) fn static_string_literal_inner(raw: &str) -> Option<&str> {
    let mut open = 0;
    let mut rest = skip_trivia(raw)?;
    while let Some(inner) = rest.strip_prefix('(') {
        open += 1;
        rest = skip_trivia(inner)?;
    }
    let delimiter = *rest.as_bytes().first()?;
    if !matches!(delimiter, b'\'' | b'"' | b'`') {
        return None;
    }
    let body = rest.get(1..)?;
    let end = literal_end(body, delimiter)?;
    let mut tail = skip_trivia(body.get(end + 1..)?)?;
    while let Some(after) = tail.strip_prefix(')') {
        open -= 1;
        tail = skip_trivia(after)?;
    }
    (tail.is_empty() && open == 0)
        .then(|| body.get(..end))
        .flatten()
}

/// Skip leading whitespace and block comments. `None` for an unterminated
/// comment or a line comment, which leaves the attribute value unparsable.
fn skip_trivia(source: &str) -> Option<&str> {
    let mut rest = source.trim_start();
    loop {
        if let Some(comment) = rest.strip_prefix("/*") {
            rest = comment.split_once("*/")?.1.trim_start();
        } else if rest.starts_with("//") {
            return None;
        } else {
            return Some(rest);
        }
    }
}

/// The index of the closing delimiter, or `None` when the literal is
/// unterminated or interpolates.
fn literal_end(body: &str, delimiter: u8) -> Option<usize> {
    let bytes = body.as_bytes();
    let mut index = 0;
    let mut escaped = false;
    while let Some(&byte) = bytes.get(index) {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == delimiter {
            return Some(index);
        } else if delimiter == b'`' && byte == b'$' && bytes.get(index + 1) == Some(&b'{') {
            return None;
        }
        index += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::static_string_literal_inner;

    #[test]
    fn scans_the_complete_authored_literal() {
        assert_eq!(static_string_literal_inner("`plain`"), Some("plain"));
        assert_eq!(
            static_string_literal_inner(r"`escaped \${text}`"),
            Some(r"escaped \${text}")
        );
        assert_eq!(static_string_literal_inner("`a` + `b`"), None);
        assert_eq!(static_string_literal_inner(r"`dynamic ${text}`"), None);
        assert_eq!(static_string_literal_inner(r"`unterminated\`"), None);
    }

    #[test]
    fn ignores_comments_and_parentheses_around_the_literal() {
        for raw in [
            "'a' /* note */",
            "/* note */ 'a'",
            "('a')",
            "( /* x */ ('a') )",
        ] {
            assert_eq!(static_string_literal_inner(raw), Some("a"), "{raw}");
        }
        for raw in [
            "('a'",
            "'a')",
            "('a') + ('b')",
            "'a' as string",
            "'a' /* open",
            "'a' // line",
        ] {
            assert_eq!(static_string_literal_inner(raw), None, "{raw}");
        }
    }
}
