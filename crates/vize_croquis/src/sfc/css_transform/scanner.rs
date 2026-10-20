//! Byte scanners for CSS `v-bind()` expressions.

use memchr::{memchr, memchr2, memchr3, memmem};

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

/// Find the next authored `v-bind(` outside CSS strings and comments.
///
/// Walks the same states as a byte-at-a-time scan, but jumps to each state's
/// next significant byte with `memchr`. Without a `v-bind(` ahead, nothing can
/// match, so the common style without one costs a single substring search.
pub(super) fn find_next_v_bind(css: &str, start: usize) -> Option<usize> {
    const NONE: usize = usize::MAX;
    let bytes = css.as_bytes();
    let tail = |at: usize| bytes.get(at..).unwrap_or_default();
    let next = |found: Option<usize>, at: usize| found.map_or(NONE, |offset| at + offset);
    let mut pos = start;
    // Absolute position of the next candidate of each kind at or after `pos`.
    let mut v_bind = next(memmem::find(tail(pos), b"v-bind("), pos);
    let mut quote = next(memchr3(b'"', b'\'', b'`', tail(pos)), pos);
    let mut slash = next(memchr(b'/', tail(pos)), pos);

    loop {
        if v_bind < pos {
            v_bind = next(memmem::find(tail(pos), b"v-bind("), pos);
        }
        if v_bind == NONE {
            return None;
        }
        if quote < pos {
            quote = next(memchr3(b'"', b'\'', b'`', tail(pos)), pos);
        }
        if slash < pos {
            slash = next(memchr(b'/', tail(pos)), pos);
        }
        let at = v_bind.min(quote).min(slash);
        match bytes.get(at) {
            Some(&quote_byte @ (b'"' | b'\'' | b'`')) => {
                // A backslash escapes the next byte, whatever it is.
                let mut cursor = at + 1;
                loop {
                    cursor += memchr2(quote_byte, b'\\', tail(cursor))?;
                    if bytes.get(cursor) == Some(&b'\\') {
                        cursor += 2;
                    } else {
                        break;
                    }
                }
                pos = cursor + 1;
            }
            Some(b'/') => match bytes.get(at + 1) {
                Some(b'*') => pos = at + 2 + memmem::find(tail(at + 2), b"*/")? + 2,
                Some(b'/') => pos = at + 2 + memchr2(b'\n', b'\r', tail(at + 2))? + 1,
                _ => pos = at + 1,
            },
            _ if has_v_bind_left_boundary(bytes, at) => return Some(at),
            _ => pos = at + 1,
        }
    }
}

fn has_v_bind_left_boundary(bytes: &[u8], pos: usize) -> bool {
    !pos.checked_sub(1)
        .and_then(|prev| bytes.get(prev))
        .is_some_and(|&byte| is_css_identifier_byte(byte))
}

fn is_css_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}

#[cfg(test)]
mod tests {
    use super::{find_next_v_bind, has_v_bind_left_boundary};

    /// The original byte-at-a-time walk, kept as the oracle.
    fn reference(css: &str, start: usize) -> Option<usize> {
        let bytes = css.as_bytes();
        let mut pos = start;
        let mut quote: Option<u8> = None;
        let mut escaped = false;
        let mut in_block_comment = false;
        let mut in_line_comment = false;

        while let Some(&byte) = bytes.get(pos) {
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
                b'v' if bytes.get(pos..).unwrap_or_default().starts_with(b"v-bind(")
                    && has_v_bind_left_boundary(bytes, pos) =>
                {
                    return Some(pos);
                }
                _ => pos += 1,
            }
        }

        None
    }

    #[test]
    fn matches_the_byte_walk_on_every_short_token_sequence() {
        const TOKENS: [&str; 12] = [
            "v-bind(", "\"", "'", "`", "/", "*", "\\", "\n", "\r", "a", "-", "v",
        ];
        let mut css = std::string::String::new();
        let mut digits = [0usize; 5];
        for len in 0..=digits.len() {
            digits = [0; 5];
            loop {
                css.clear();
                for &digit in digits.iter().take(len) {
                    css.push_str(TOKENS[digit]);
                }
                for start in 0..=css.len() + 1 {
                    assert_eq!(
                        find_next_v_bind(&css, start),
                        reference(&css, start),
                        "{css:?} from {start}"
                    );
                }
                let Some(carry) = digits.iter().take(len).position(|&d| d + 1 < TOKENS.len())
                else {
                    break;
                };
                digits[carry] += 1;
                digits.iter_mut().take(carry).for_each(|digit| *digit = 0);
            }
        }
    }
}
