use oxc_syntax::identifier::is_identifier_part;

pub(super) fn helper_call_position(text: &str, alias: &str) -> Option<usize> {
    debug_assert!(
        alias.starts_with('_'),
        "helper aliases start with underscore"
    );
    let bytes = text.as_bytes();
    let alias = alias.as_bytes();
    let mut position = 0;
    while let Some(&byte) = bytes.get(position) {
        let rest = bytes.get(position + 1..).unwrap_or_default();
        match byte {
            b'\'' | b'"' | b'`' => position = quoted_end(bytes, position),
            b'/' if rest.first() == Some(&b'/') => {
                position = rest
                    .iter()
                    .skip(1)
                    .position(|byte| *byte == b'\n')
                    .map_or(bytes.len(), |end| position + 2 + end);
            }
            b'/' if rest.first() == Some(&b'*') => {
                position = rest
                    .get(1..)
                    .unwrap_or_default()
                    .windows(2)
                    .position(|pair| pair == b"*/")
                    .map_or(bytes.len(), |end| position + 4 + end);
            }
            // `alias` is UTF-8, so a match starts on a char boundary and
            // `text.get(..position)` is present.
            b'_' if bytes
                .get(position..)
                .is_some_and(|tail| tail.starts_with(alias))
                && can_start_alias(text, position) =>
            {
                let after = position + alias.len();
                if bytes
                    .get(after..)
                    .unwrap_or_default()
                    .iter()
                    .find(|byte| !byte.is_ascii_whitespace())
                    == Some(&b'(')
                {
                    return Some(position);
                }
                position = after;
            }
            _ => position += 1,
        }
    }
    None
}

/// Visit every alias-shaped call in lexical order under the same rules as
/// [`helper_call_position`]. The visitor returns `true` once it has found all
/// calls it needs.
pub(super) fn underscore_call_sites(text: &str, mut visit: impl FnMut(usize, &str) -> bool) {
    let bytes = text.as_bytes();
    let mut position = 0;
    while let Some(&byte) = bytes.get(position) {
        let rest = bytes.get(position + 1..).unwrap_or_default();
        match byte {
            b'\'' | b'"' | b'`' => position = quoted_end(bytes, position),
            b'/' if rest.first() == Some(&b'/') => {
                position = rest
                    .iter()
                    .skip(1)
                    .position(|byte| *byte == b'\n')
                    .map_or(bytes.len(), |end| position + 2 + end);
            }
            b'/' if rest.first() == Some(&b'*') => {
                position = rest
                    .get(1..)
                    .unwrap_or_default()
                    .windows(2)
                    .position(|pair| pair == b"*/")
                    .map_or(bytes.len(), |end| position + 4 + end);
            }
            b'_' if can_start_alias(text, position) => {
                let len = rest
                    .iter()
                    .position(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')))
                    .unwrap_or(rest.len());
                let after = position + 1 + len;
                let called = bytes
                    .get(after..)
                    .unwrap_or_default()
                    .iter()
                    .find(|byte| !byte.is_ascii_whitespace())
                    == Some(&b'(');
                if called
                    && let Some(name) = text.get(position..after)
                    && visit(position, name)
                {
                    return;
                }
                position = after;
            }
            _ => position += 1,
        }
    }
}

fn can_start_alias(text: &str, position: usize) -> bool {
    if position == 0 {
        return true;
    }
    match text.as_bytes().get(position - 1).copied() {
        Some(byte) if byte.is_ascii() => {
            !(byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'.'))
        }
        Some(_) => text.get(..position).is_some_and(|before| {
            before
                .chars()
                .next_back()
                .is_none_or(|ch| !is_identifier_part(ch) && ch != '.')
        }),
        None => false,
    }
}

fn quoted_end(bytes: &[u8], start: usize) -> usize {
    let Some(&quote) = bytes.get(start) else {
        return bytes.len();
    };
    let mut position = start + 1;
    while let Some(&byte) = bytes.get(position) {
        match byte {
            b'\\' => position += 2,
            byte if byte == quote => return position + 1,
            _ => position += 1,
        }
    }
    bytes.len()
}

#[cfg(test)]
mod tests {
    use super::{can_start_alias, helper_call_position};

    #[test]
    fn scans_helper_calls_without_picking_up_strings_comments_or_members() {
        for (code, expected) in [
            ("abc _createVNode()", Some(4)),
            ("a._createVNode(); _createVNode (x)", Some(18)),
            (
                "'_createVNode()' /* _createVNode() */ _createVNode()",
                Some(38),
            ),
            ("a_createVNode(); _createVNode\n(x)", Some(17)),
            ("_createVNodeExtra(); // _createVNode()", None),
        ] {
            assert_eq!(
                helper_call_position(code, "_createVNode"),
                expected,
                "{code}"
            );
        }
    }

    #[test]
    fn ascii_boundaries_and_unicode_fallback_match_the_identifier_oracle() {
        for ch in (0u8..=127)
            .map(char::from)
            .chain(['é', '中', '\u{200c}', '😀', '\u{a0}'])
        {
            let mut text = String::new();
            text.push(ch);
            let position = text.len();
            text.push_str("_createVNode()");
            assert_eq!(
                can_start_alias(&text, position),
                !oxc_syntax::identifier::is_identifier_part(ch) && ch != '.',
                "{ch:?}"
            );
        }
        assert!(can_start_alias("_createVNode()", 0));
    }
}
