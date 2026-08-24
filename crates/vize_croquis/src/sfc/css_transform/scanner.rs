//! Byte scanners for CSS `v-bind()` expressions.

/// Find the matching closing parenthesis.
#[doc(hidden)]
pub fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 1u32;
    let mut quote = None;
    let mut escaped = false;
    let mut in_block_comment = false;
    let mut in_line_comment = false;
    let mut chars = s.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if in_block_comment {
            if c == '*' && chars.peek().is_some_and(|(_, next)| *next == '/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if in_line_comment {
            if c == '\n' || c == '\r' {
                in_line_comment = false;
            }
            continue;
        }

        if let Some(quote_char) = quote {
            if escaped {
                escaped = false;
                continue;
            }

            if c == '\\' {
                escaped = true;
                continue;
            }

            if c == quote_char {
                quote = None;
            }

            continue;
        }

        match c {
            '/' if chars.peek().is_some_and(|(_, next)| *next == '*') => {
                chars.next();
                in_block_comment = true;
            }
            '/' if chars.peek().is_some_and(|(_, next)| *next == '/') => {
                chars.next();
                in_line_comment = true;
            }
            '"' | '\'' | '`' => quote = Some(c),
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

pub(super) fn find_next_v_bind(css: &str, start: usize) -> Option<usize> {
    let bytes = css.as_bytes();
    let mut pos = start;
    let mut quote: Option<u8> = None;
    let mut escaped = false;
    let mut in_block_comment = false;
    let mut in_line_comment = false;

    while pos < bytes.len() {
        let byte = bytes[pos];

        if in_block_comment {
            if byte == b'*' && bytes.get(pos + 1) == Some(&b'/') {
                in_block_comment = false;
                pos += 2;
            } else {
                pos += 1;
            }
            continue;
        }

        if in_line_comment {
            if byte == b'\n' || byte == b'\r' {
                in_line_comment = false;
            }
            pos += 1;
            continue;
        }

        if let Some(quote_byte) = quote {
            if escaped {
                escaped = false;
                pos += 1;
                continue;
            }

            if byte == b'\\' {
                escaped = true;
                pos += 1;
                continue;
            }

            if byte == quote_byte {
                quote = None;
            }
            pos += 1;
            continue;
        }

        match byte {
            b'"' | b'\'' | b'`' => {
                quote = Some(byte);
                pos += 1;
            }
            b'/' if bytes.get(pos + 1) == Some(&b'*') => {
                in_block_comment = true;
                pos += 2;
            }
            b'/' if bytes.get(pos + 1) == Some(&b'/') => {
                in_line_comment = true;
                pos += 2;
            }
            b'v' if bytes[pos..].starts_with(b"v-bind(")
                && has_v_bind_left_boundary(bytes, pos) =>
            {
                return Some(pos);
            }
            _ => pos += 1,
        }
    }

    None
}

fn has_v_bind_left_boundary(bytes: &[u8], pos: usize) -> bool {
    pos == 0 || !is_css_identifier_byte(bytes[pos - 1])
}

fn is_css_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}
