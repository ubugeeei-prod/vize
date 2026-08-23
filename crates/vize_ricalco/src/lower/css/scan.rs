//! Locate CSS `v-bind()` calls the way the shipped extractor does
//! (`vize_atelier_sfc::css::transform`): strings and comments are not
//! calls, `my-v-bind(` / `-webkit-v-bind(` need a left boundary, and an
//! unmatched `(` stops the scan. Kept as a conversion-local copy so
//! ricalco never grows an atelier_sfc edge (that crate is published;
//! this one is not).

/// One `v-bind(...)` whose argument survived trim + outer-quote strip.
pub(super) struct Hit<'a> {
    /// Byte offset of the `v` in `v-bind`.
    pub call_start: usize,
    /// Byte offset after the matching `)`.
    pub call_end: usize,
    /// Argument text, a slice of the CSS (quotes stripped).
    pub expr: &'a str,
}

pub(super) fn next<'a>(css: &'a str, from: usize) -> Option<Hit<'a>> {
    let start = find_next_v_bind(css, from)?;
    let inner = start + 7;
    let after_open = css.get(inner..)?;
    let end = find_matching_paren(after_open)?;
    let inside = after_open.get(..end)?;
    let expr = trim_outer_quotes(inside.trim());
    Some(Hit {
        call_start: start,
        call_end: inner + end + 1,
        expr,
    })
}

fn trim_outer_quotes(expr: &str) -> &str {
    let bytes = expr.as_bytes();
    if bytes.len() >= 2
        && matches!(bytes.first(), Some(b'"' | b'\''))
        && bytes.first() == bytes.last()
    {
        &expr[1..expr.len() - 1]
    } else {
        expr
    }
}

fn find_matching_paren(s: &str) -> Option<usize> {
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

fn find_next_v_bind(css: &str, start: usize) -> Option<usize> {
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
