use oxc_syntax::identifier::is_identifier_part;

/// The per-alias reference scan [`underscore_call_sites`] must agree with.
pub(super) fn helper_call_position(text: &str, alias: &str) -> Option<usize> {
    debug_assert!(alias.starts_with('_'));
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

/// Every `_name(` call site in `text`, in order, under the same lexical
/// rules as [`helper_call_position`]: strings and comments are skipped,
/// the name must not follow an identifier character or `.`, and only
/// whitespace may sit between the name and `(`. `visit` receives the
/// position and the name and returns `true` to stop.
///
/// Every helper alias is an ASCII identifier starting with `_`, so the
/// first site a `visit` sees for an alias is the position
/// [`helper_call_position`] reports for it; one scan serves all aliases.
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
            b'_' if text.get(..position).is_some_and(|before| {
                before
                    .chars()
                    .next_back()
                    .is_none_or(|ch| !is_identifier_part(ch) && ch != '.')
            }) =>
            {
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
