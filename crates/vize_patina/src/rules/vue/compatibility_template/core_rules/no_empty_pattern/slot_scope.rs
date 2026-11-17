//! Empty destructuring patterns in an authored `slot-scope="..."` attribute.

/// The `{}` and `[]` patterns in the attribute `name="params"`, as byte
/// ranges of `attribute`. A pattern starts the parameter list or follows `(`,
/// `,`, `:` or `[`; an empty literal after `=` is a default value instead.
pub(super) fn empty_patterns(attribute: &str) -> Vec<(usize, usize)> {
    let bytes = attribute.as_bytes();
    let Some(equals) = attribute.find('=') else {
        return Vec::new();
    };
    let mut open = equals + 1;
    while bytes.get(open).is_some_and(u8::is_ascii_whitespace) {
        open += 1;
    }
    let Some(&quote @ (b'"' | b'\'')) = bytes.get(open) else {
        return Vec::new();
    };
    let start = open + 1;
    let end = (start..bytes.len())
        .find(|&at| bytes.get(at) == Some(&quote))
        .unwrap_or(bytes.len());
    let mut found = Vec::new();
    let mut previous = None;
    let mut index = start;
    while index < end {
        let byte = bytes.get(index).copied().unwrap_or_default();
        if matches!(byte, b'{' | b'[') {
            let close = if byte == b'{' { b'}' } else { b']' };
            let mut inner = index + 1;
            while inner < end && bytes.get(inner).is_some_and(u8::is_ascii_whitespace) {
                inner += 1;
            }
            let pattern_position = matches!(previous, None | Some(b'(' | b',' | b':' | b'['));
            if bytes.get(inner) == Some(&close) && pattern_position {
                found.push((index, inner + 1));
                previous = Some(close);
                index = inner + 1;
                continue;
            }
        }
        if !byte.is_ascii_whitespace() {
            previous = Some(byte);
        }
        index += 1;
    }
    found
}

#[cfg(test)]
mod tests {
    use super::empty_patterns;

    fn spans(attribute: &str) -> Vec<&str> {
        empty_patterns(attribute)
            .into_iter()
            .filter_map(|(start, end)| attribute.get(start..end))
            .collect()
    }

    #[test]
    fn finds_empty_patterns_but_not_default_values() {
        assert_eq!(spans(r#"slot-scope="{}""#), ["{}"]);
        assert_eq!(
            spans(r#"slot-scope="{ row: {}, list: [ ] }""#),
            ["{}", "[ ]"]
        );
        assert_eq!(spans(r#"scope='[]'"#), ["[]"]);
        assert_eq!(spans(r#"slot-scope="{ row = {} }""#), Vec::<&str>::new());
        assert_eq!(spans(r#"slot-scope="{ row }""#), Vec::<&str>::new());
        assert_eq!(spans(r#"slot-scope="scope""#), Vec::<&str>::new());
    }
}
