/// Return the authored body when `raw` is exactly one static string literal.
///
/// The scanner is delimiter-aware so a second template literal or an
/// unescaped interpolation opener cannot be mistaken for part of the first.
pub(super) fn static_string_literal_inner(raw: &str) -> Option<&str> {
    let source = raw.trim();
    let bytes = source.as_bytes();
    let (&delimiter, &last) = (bytes.first()?, bytes.last()?);
    if bytes.len() < 2 || delimiter != last || !matches!(delimiter, b'\'' | b'"' | b'`') {
        return None;
    }
    let inner = source.get(1..source.len() - 1)?;
    literal_body_is_static(inner, delimiter).then_some(inner)
}

fn literal_body_is_static(inner: &str, delimiter: u8) -> bool {
    let bytes = inner.as_bytes();
    let mut index = 0;
    let mut escaped = false;
    while let Some(&byte) = bytes.get(index) {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == delimiter {
            return false;
        } else if delimiter == b'`' && byte == b'$' && bytes.get(index + 1) == Some(&b'{') {
            return false;
        }
        index += 1;
    }
    !escaped
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
}
