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
                && text.get(..position).is_some_and(|before| {
                    before
                        .chars()
                        .next_back()
                        .is_none_or(|ch| !is_identifier_part(ch) && ch != '.')
                }) =>
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

/// Visit alias-shaped calls in lexical order under the same rules as
/// [`helper_call_position`], starting at `from`. The visitor returns `true`
/// once it has what it needs; the scan then stops and returns where a later
/// call can resume. A finished scan returns `text.len()`.
///
/// `from` is `0` or a position this function returned for the same text, so
/// it never falls inside a string or a comment.
pub(super) fn underscore_call_sites(
    text: &str,
    from: usize,
    mut visit: impl FnMut(usize, &str) -> bool,
) -> usize {
    let bytes = text.as_bytes();
    let mut position = from;
    // Only a quote, a slash or an underscore starts anything here, so the
    // scan jumps from one to the next instead of dispatching on every byte.
    while let Some(skipped) = bytes.get(position..).and_then(|tail| {
        tail.iter()
            .position(|byte| matches!(byte, b'\'' | b'"' | b'`' | b'/' | b'_'))
    }) {
        position += skipped;
        let Some(&byte) = bytes.get(position) else {
            break;
        };
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
            b'_' if starts_identifier(text, position) => {
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
                    return after;
                }
                position = after;
            }
            _ => position += 1,
        }
    }
    bytes.len()
}

/// Whether an identifier can start at `position`: nothing stands before it,
/// or a character that is neither part of an identifier nor a member dot.
fn starts_identifier(text: &str, position: usize) -> bool {
    let Some(before) = text.get(..position) else {
        return false;
    };
    match before.as_bytes().last() {
        None => true,
        // An ASCII byte decides without decoding a character: there the
        // identifier-part set is `[A-Za-z0-9_$]`.
        Some(byte) if byte.is_ascii() => {
            !(byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'.'))
        }
        Some(_) => before
            .chars()
            .next_back()
            .is_none_or(|ch| !is_identifier_part(ch) && ch != '.'),
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
    use super::{helper_call_position, underscore_call_sites};
    use vize_l0::String;

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

    /// `starts_identifier` decides an ASCII prefix without decoding it. That
    /// is sound only while the identifier-part set is `[A-Za-z0-9_$]` on
    /// ASCII, which this pins against the parser's own predicate.
    #[test]
    fn ascii_identifier_parts_are_alphanumerics_underscore_and_dollar() {
        for byte in 0_u8..=127 {
            assert_eq!(
                oxc_syntax::identifier::is_identifier_part(char::from(byte)),
                byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'),
                "{byte:#x}"
            );
        }
    }

    /// A scan stopped at a call resumes after it and sees exactly the calls
    /// an uninterrupted scan sees.
    #[test]
    fn a_resumed_scan_visits_the_calls_an_uninterrupted_scan_visits() {
        let text = "_a(1) '_b(' _c (2) /* _d( */ x._e(3) _f\n(4) `_g(` _h(5) // _i(\n_j(6)";
        let mut whole = std::vec::Vec::new();
        assert_eq!(
            underscore_call_sites(text, 0, |position, alias| {
                whole.push((position, String::from(alias)));
                false
            }),
            text.len()
        );
        assert_eq!(
            whole
                .iter()
                .map(|(_, alias)| alias.as_str())
                .collect::<std::vec::Vec<_>>(),
            ["_a", "_c", "_f", "_h", "_j"]
        );
        let mut stepped = std::vec::Vec::new();
        let mut from = 0;
        while from < text.len() {
            from = underscore_call_sites(text, from, |position, alias| {
                stepped.push((position, String::from(alias)));
                true
            });
        }
        assert_eq!(stepped, whole);
    }

    #[test]
    fn batched_and_scalar_scans_agree_across_ascii_and_unicode_prefixes() {
        for ch in (0u8..=127)
            .map(char::from)
            .chain(['é', '中', '\u{200c}', '😀', '\u{a0}'])
        {
            let mut text = String::default();
            text.push(ch);
            text.push_str("_createVNode()");
            let mut selected = None;
            underscore_call_sites(&text, 0, |position, alias| {
                if alias == "_createVNode" {
                    selected = Some(position);
                }
                false
            });
            assert_eq!(
                selected,
                helper_call_position(&text, "_createVNode"),
                "{ch:?}"
            );
        }
    }
}
