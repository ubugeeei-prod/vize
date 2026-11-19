//! Original interpolation and regular-expression boundary scanning.

use super::find_bytes;

/// Bytes the interpolation scanner gives a meaning outside a string: every
/// other byte only moves it forward.
fn is_interpolation_delimiter(byte: u8) -> bool {
    matches!(byte, b'\'' | b'"' | b'`' | b'/' | b'{' | b'}')
}

pub(super) fn skip_interpolation(bytes: &[u8], from: usize) -> Option<usize> {
    // Up to the first delimiter the scanner is outside any string, brace or
    // comment, so a `}` there is the depth-zero close and answers at once.
    let offset = bytes
        .get(from..)?
        .iter()
        .position(|&byte| is_interpolation_delimiter(byte))?;
    let pos = from + offset;
    match bytes.get(pos) {
        Some(b'}') => (bytes.get(pos + 1) == Some(&b'}')).then_some(pos + 2),
        // A slash reads the operand before it to tell a regex from division.
        Some(b'/') => skip_interpolation_body(bytes, from),
        // A string or brace sets the regex state itself before it is read.
        _ => skip_interpolation_body(bytes, pos),
    }
}

fn skip_interpolation_body(bytes: &[u8], from: usize) -> Option<usize> {
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

#[cfg(test)]
mod tests {
    use super::{skip_interpolation, skip_interpolation_body};

    #[test]
    fn delimiter_fast_path_matches_the_full_scan_from_the_start() {
        for body in [
            "",
            "}",
            "}}",
            " title }}",
            " title } }}",
            " a.b ?? c }}x",
            " ok }",
            " x++ }}",
            " !done }}",
            " 'a}}' }}",
            " \"q\" + y }}",
            " `t${x}` }}",
            " { a: 1 }.a }}",
            " a / b }}",
            " return /}}/.test(x) }}",
            " typeof /x/ }}",
            " a // c\n }}",
            " a /* }} */ }}",
            " 雪 }}",
            " x ",
        ] {
            let source = ["{{", body].concat();
            assert_eq!(
                skip_interpolation(source.as_bytes(), 2),
                skip_interpolation_body(source.as_bytes(), 2),
                "{source:?}"
            );
        }
    }
}
