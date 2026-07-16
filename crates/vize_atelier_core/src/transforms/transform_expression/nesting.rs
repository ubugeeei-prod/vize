/// Maximum expression nesting depth accepted before parsing.
///
/// OXC recurses for nested brackets; stack overflow (#956) and a parser timeout
/// at depth 32 (#2944) cannot be caught, so every entry point shares this guard.
pub const MAX_EXPRESSION_NESTING_DEPTH: usize = 31;

/// Returns the maximum parser-recursion depth in `content`.
///
/// Brackets and TypeScript angles are paired, while decorator markers accumulate
/// for OXC's recursive parser. Strings, templates, comments, and regexes are skipped.
pub fn expression_nesting_depth(content: &str) -> usize {
    let bytes = content.as_bytes();
    let (mut bracket_depth, mut angle_depth, mut decorator_depth) = (0usize, 0usize, 0usize);
    let mut max_depth = 0usize;
    let mut can_start_regex = true;
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b' ' | b'\t' | b'\r' | b'\n' => {
                i += 1;
                continue;
            }
            b'"' | b'\'' | b'`' => {
                i = skip_quoted(bytes, i + 1, b);
                can_start_regex = false;
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                i = skip_line_comment(bytes, i + 2);
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = skip_block_comment(bytes, i + 2);
                continue;
            }
            b'/' if can_start_regex => {
                i = skip_regex(bytes, i + 1);
                can_start_regex = false;
                continue;
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => {
                let start = i;
                i = skip_identifier(bytes, i + 1);
                can_start_regex = keyword_allows_regex_after(&bytes[start..i]);
                continue;
            }
            b'0'..=b'9' => {
                i = skip_number(bytes, i + 1);
                can_start_regex = false;
                continue;
            }
            b'(' | b'[' | b'{' => {
                bracket_depth += 1;
                can_start_regex = true;
            }
            b')' | b']' | b'}' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                can_start_regex = false;
            }
            b'<' => {
                angle_depth += 1;
                can_start_regex = true;
            }
            b'>' => {
                angle_depth = angle_depth.saturating_sub(1);
                can_start_regex = true;
            }
            b'@' => {
                decorator_depth += 1;
                can_start_regex = true;
            }
            b'.' => can_start_regex = false,
            b'+' | b'-' if bytes.get(i + 1) == Some(&b) => {
                i += 1;
                can_start_regex = false;
            }
            b',' | b';' | b':' | b'?' | b'!' | b'=' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&'
            | b'|' | b'^' | b'~' => can_start_regex = true,
            _ => can_start_regex = b < 0x80,
        }

        max_depth = max_depth.max(bracket_depth + angle_depth + decorator_depth);
        i += 1;
    }

    max_depth
}

/// Returns true if `content` exceeds [`MAX_EXPRESSION_NESTING_DEPTH`].
#[inline]
pub fn expression_exceeds_max_depth(content: &str) -> bool {
    expression_nesting_depth(content) > MAX_EXPRESSION_NESTING_DEPTH
}

fn skip_quoted(bytes: &[u8], mut i: usize, quote: u8) -> usize {
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = i.saturating_add(2);
        } else if bytes[i] == quote {
            return i + 1;
        } else {
            i += 1;
        }
    }
    i
}

fn skip_line_comment(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    i
}

fn skip_block_comment(bytes: &[u8], mut i: usize) -> usize {
    while i + 1 < bytes.len() {
        if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            return i + 2;
        }
        i += 1;
    }
    bytes.len()
}

fn skip_regex(bytes: &[u8], mut i: usize) -> usize {
    let mut in_character_class = false;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i = i.saturating_add(2),
            b'[' => {
                in_character_class = true;
                i += 1;
            }
            b']' => {
                in_character_class = false;
                i += 1;
            }
            b'/' if !in_character_class => return skip_identifier(bytes, i + 1),
            b'\n' | b'\r' => return i,
            _ => i += 1,
        }
    }
    i
}

fn skip_identifier(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len()
        && matches!(bytes[i], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$')
    {
        i += 1;
    }
    i
}

fn skip_number(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len()
        && matches!(bytes[i], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'.')
    {
        i += 1;
    }
    i
}

fn keyword_allows_regex_after(identifier: &[u8]) -> bool {
    matches!(
        identifier,
        b"await"
            | b"case"
            | b"delete"
            | b"do"
            | b"else"
            | b"in"
            | b"instanceof"
            | b"new"
            | b"of"
            | b"return"
            | b"throw"
            | b"typeof"
            | b"void"
            | b"yield"
    )
}
