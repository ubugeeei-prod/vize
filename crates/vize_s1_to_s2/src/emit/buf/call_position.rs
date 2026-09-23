use oxc_syntax::identifier::is_identifier_part;

pub(super) fn helper_call_position(text: &str, alias: &str) -> Option<usize> {
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
            _ if bytes
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
