//! Authored attributes of one opening tag, read straight from the source.

/// One attribute as written; ranges are byte offsets into the tag source.
pub(super) struct AuthoredAttribute<'a> {
    pub(super) name: &'a str,
    pub(super) name_start: usize,
    /// The value with its quotes, when the attribute has `=`.
    pub(super) value: Option<(usize, usize)>,
}

/// Iterate the attributes of the opening tag that starts `element_source`.
///
/// `element_source` begins at `<`; scanning stops at the `>` that closes the
/// opening tag, so children and the end tag are never read.
pub(super) fn authored_attributes(
    element_source: &str,
) -> impl Iterator<Item = AuthoredAttribute<'_>> {
    let bytes = element_source.as_bytes();
    // Skip `<` and the tag name.
    let mut at = bytes
        .iter()
        .position(|byte| byte.is_ascii_whitespace() || matches!(byte, b'>' | b'/'))
        .unwrap_or(bytes.len());

    std::iter::from_fn(move || {
        at = skip_while(bytes, at, |byte| byte.is_ascii_whitespace() || byte == b'/');
        if bytes.get(at).is_none_or(|byte| *byte == b'>') {
            return None;
        }

        let name_start = at;
        at = skip_while(bytes, at, |byte| {
            !byte.is_ascii_whitespace() && !matches!(byte, b'=' | b'>' | b'/')
        });
        // A lone `/` that is not followed by `>` belongs to the name.
        if at == name_start {
            at += 1;
        }
        let name = element_source.get(name_start..at)?;

        let after_name = skip_while(bytes, at, |byte| byte.is_ascii_whitespace());
        let mut value = None;
        if bytes.get(after_name) == Some(&b'=') {
            let value_start = skip_while(bytes, after_name + 1, |byte| byte.is_ascii_whitespace());
            let value_end = match bytes.get(value_start) {
                Some(&quote @ (b'"' | b'\'')) => bytes
                    .get(value_start + 1..)
                    .and_then(|rest| rest.iter().position(|byte| *byte == quote))
                    .map_or(bytes.len(), |close| value_start + close + 2),
                _ => skip_while(bytes, value_start, |byte| {
                    !byte.is_ascii_whitespace() && byte != b'>'
                }),
            };
            value = Some((value_start, value_end));
            at = value_end;
        }

        Some(AuthoredAttribute {
            name,
            name_start,
            value,
        })
    })
}

fn skip_while(bytes: &[u8], mut at: usize, keep: impl Fn(u8) -> bool) -> usize {
    while bytes.get(at).is_some_and(|byte| keep(*byte)) {
        at += 1;
    }
    at
}
