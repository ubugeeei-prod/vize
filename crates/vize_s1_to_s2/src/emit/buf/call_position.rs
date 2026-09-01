use oxc_syntax::identifier::is_identifier_part;

pub(super) fn helper_call_position(text: &str, alias: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let alias = alias.as_bytes();
    let mut position = 0;
    while position < bytes.len() {
        match bytes[position] {
            b'\'' | b'"' | b'`' => position = quoted_end(bytes, position),
            b'/' if bytes.get(position + 1) == Some(&b'/') => {
                position = bytes[position + 2..]
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .map_or(bytes.len(), |end| position + 2 + end);
            }
            b'/' if bytes.get(position + 1) == Some(&b'*') => {
                position = bytes[position + 2..]
                    .windows(2)
                    .position(|pair| pair == b"*/")
                    .map_or(bytes.len(), |end| position + 4 + end);
            }
            _ if bytes[position..].starts_with(alias)
                && text[..position]
                    .chars()
                    .next_back()
                    .is_none_or(|ch| !is_identifier_part(ch) && ch != '.') =>
            {
                let after = position + alias.len();
                if bytes[after..]
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
    let quote = bytes[start];
    let mut position = start + 1;
    while position < bytes.len() {
        match bytes[position] {
            b'\\' => position += 2,
            byte if byte == quote => return position + 1,
            _ => position += 1,
        }
    }
    bytes.len()
}
