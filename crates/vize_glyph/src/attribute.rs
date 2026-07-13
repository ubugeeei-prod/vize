//! Shared attribute serialization helpers.

/// Write an attribute value with a safe quote style and escaped delimiters.
///
/// The callback keeps the common path allocation-free for callers that already
/// own an output buffer. Double quotes are preferred unless the value contains
/// double quotes but no single quotes, in which case single quotes avoid
/// unnecessary entities.
#[inline]
pub(crate) fn write_attr_value(value: &str, mut write: impl FnMut(&str)) {
    let contains_double_quote = value.contains('"');
    if contains_double_quote && !value.contains('\'') {
        write("'");
        write(value);
        write("'");
        return;
    }

    write("\"");
    if contains_double_quote {
        let mut segments = value.split('"');
        if let Some(first) = segments.next() {
            write(first);
        }
        for segment in segments {
            write("&quot;");
            write(segment);
        }
    } else {
        write(value);
    }
    write("\"");
}

#[cfg(test)]
mod tests {
    use super::write_attr_value;

    #[test]
    fn selects_safe_quotes_and_escapes_delimiters() {
        let cases = [
            ("a'b", r#""a'b""#),
            (r#"a"b"#, r#"'a"b'"#),
            (r#"a"b'c"#, r#""a&quot;b'c""#),
        ];

        for (value, expected) in cases {
            let mut output = String::new();
            write_attr_value(value, |segment| output.push_str(segment));
            assert_eq!(output, expected, "unexpected output for {value}");
        }
    }
}
