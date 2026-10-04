//! Original interpolation and regular-expression boundary scanning.

use super::find_bytes;

pub(super) fn skip_interpolation(bytes: &[u8], from: usize) -> Option<usize> {
    let (mut pos, mut quote, mut braces) = (from, None, 0usize);
    let mut can_start_regex = true;
    while pos + 1 < bytes.len() {
        let byte = *bytes.get(pos)?;
        let next = *bytes.get(pos + 1)?;
        match (quote, byte) {
            (Some(_), b'\\') => pos += 2,
            (Some(b'`'), b'$') if next == b'{' => {
                // Nested template expressions need a full JS lexer. Refuse a
                // trusted block boundary until the native container owns one.
                return None;
            }
            (Some(active), byte) if active == byte => {
                quote = None;
                pos += 1;
                can_start_regex = false;
            }
            (None, b'\'' | b'"' | 0x60) => {
                quote = Some(byte);
                pos += 1;
            }
            (None, b'/') if next == b'*' => {
                pos = find_bytes(bytes, pos + 2, b"*/")? + 2;
            }
            (None, b'/') if next == b'/' => {
                pos = bytes
                    .get(pos + 2..)?
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .map_or(bytes.len(), |offset| pos + 3 + offset);
            }
            (None, b'{') => {
                braces += 1;
                pos += 1;
                can_start_regex = true;
            }
            (None, b'}') if braces != 0 => {
                braces -= 1;
                pos += 1;
                can_start_regex = false;
            }
            (None, b'}') if next == b'}' => return Some(pos + 2),
            (None, b'}') => return None,
            (None, b'/') if can_start_regex => {
                pos = skip_regex(bytes, pos)?;
                can_start_regex = false;
            }
            (None, b'/') => {
                pos += 1;
                can_start_regex = true;
            }
            (None, b')' | b']') => {
                pos += 1;
                can_start_regex = false;
            }
            (None, b'!') => {
                // Prefix JS negation keeps an operand pending; after an
                // operand, Vue+TS also permits a postfix non-null assertion.
                pos += 1;
            }
            (None, b'+' | b'-') if next == byte => {
                // JS increment/decrement can be prefix or postfix. Either
                // form preserves whether an operand was already present.
                pos += 2;
            }
            (None, byte) if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$') => {
                let start = pos;
                while bytes
                    .get(pos)
                    .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
                {
                    pos += 1;
                }
                can_start_regex = matches!(
                    bytes.get(start..pos)?,
                    b"return"
                        | b"throw"
                        | b"case"
                        | b"yield"
                        | b"await"
                        | b"delete"
                        | b"void"
                        | b"typeof"
                        | b"instanceof"
                        | b"new"
                        | b"in"
                        | b"of"
                );
            }
            (None, byte) if !byte.is_ascii_whitespace() => {
                pos += 1;
                can_start_regex = true;
            }
            _ => pos += 1,
        }
    }
    None
}

fn skip_regex(bytes: &[u8], from: usize) -> Option<usize> {
    let (mut pos, mut in_class) = (from + 1, false);
    while pos < bytes.len() {
        match bytes.get(pos).copied()? {
            b'\\' => pos += 2,
            b'[' => {
                in_class = true;
                pos += 1;
            }
            b']' => {
                in_class = false;
                pos += 1;
            }
            b'/' if !in_class => {
                pos += 1;
                while bytes.get(pos).is_some_and(u8::is_ascii_alphabetic) {
                    pos += 1;
                }
                return Some(pos);
            }
            b'\n' | b'\r' => return None,
            _ => pos += 1,
        }
    }
    None
}
