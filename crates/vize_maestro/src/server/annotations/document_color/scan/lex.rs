//! Small CSS lexical helpers used by the colour scanner.

#[inline]
pub(super) fn pair_at(bytes: &[u8], start: usize, pair: &[u8; 2]) -> bool {
    bytes.get(start) == Some(&pair[0]) && bytes.get(start + 1) == Some(&pair[1])
}

#[inline]
pub(super) fn is_identifier_byte(byte: Option<&u8>) -> bool {
    byte.is_some_and(|&byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'\\') || byte >= 0x80
    })
}

pub(super) fn is_identifier_boundary(bytes: &[u8], start: usize, region_start: usize) -> bool {
    if is_identifier_byte(bytes.get(start.wrapping_sub(1)))
        || bytes
            .get(start.wrapping_sub(1))
            .is_some_and(|byte| matches!(byte, b'$' | b'@' | b'#' | b'.'))
    {
        return false;
    }
    let Some(previous) = start.checked_sub(1) else {
        return true;
    };
    let whitespace_start = match bytes[previous] {
        b'\n' if previous > region_start && bytes[previous - 1] == b'\r' => previous - 1,
        b'\t' | b'\n' | b'\x0c' | b'\r' | b' ' => previous,
        _ => return true,
    };
    let Some(prefix) = bytes.get(region_start..whitespace_start) else {
        return true;
    };
    let hex_digits = prefix
        .iter()
        .rev()
        .take(6)
        .take_while(|byte| byte.is_ascii_hexdigit())
        .count();
    hex_digits == 0
        || prefix
            .len()
            .checked_sub(hex_digits + 1)
            .and_then(|escape| prefix.get(escape))
            != Some(&b'\\')
}

pub(super) fn is_declaration_name(
    bytes: &[u8],
    start: usize,
    end: usize,
    allow_variable_prefix: bool,
) -> bool {
    #[cfg(test)]
    super::record_declaration_name_work(end.saturating_sub(start));
    let mut cursor = skip_trivia(bytes, start, end);
    if allow_variable_prefix
        && bytes
            .get(cursor)
            .is_some_and(|byte| matches!(byte, b'$' | b'@'))
    {
        cursor += 1;
    }
    let name_start = cursor;
    cursor = identifier_end(bytes, cursor, end);
    cursor > name_start && skip_trivia(bytes, cursor, end) == end
}

fn skip_trivia(bytes: &[u8], mut cursor: usize, limit: usize) -> usize {
    loop {
        while cursor < limit && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if pair_at(bytes, cursor, b"/*") {
            cursor = skip_comment(bytes, cursor, limit);
        } else {
            return cursor;
        }
    }
}

pub(super) fn skip_comment(bytes: &[u8], start: usize, limit: usize) -> usize {
    comment_end(bytes, start, limit).unwrap_or(limit)
}

fn comment_end(bytes: &[u8], start: usize, limit: usize) -> Option<usize> {
    let mut cursor = start + 2;
    while cursor + 1 < limit {
        if pair_at(bytes, cursor, b"*/") {
            return Some(cursor + 2);
        }
        cursor += 1;
    }
    None
}

pub(super) fn skip_string(bytes: &[u8], start: usize, limit: usize) -> usize {
    let quote = bytes[start];
    let mut cursor = start + 1;
    while cursor < limit {
        match bytes[cursor] {
            b'\\' => cursor = (cursor + 2).min(limit),
            byte if byte == quote => return cursor + 1,
            _ => cursor += 1,
        }
    }
    limit
}

pub(super) fn identifier_end(bytes: &[u8], start: usize, limit: usize) -> usize {
    let mut cursor = start;
    while cursor < limit {
        if bytes[cursor] == b'\\' {
            cursor = escape_end(bytes, cursor, limit);
        } else if is_identifier_byte(bytes.get(cursor)) {
            cursor += 1;
        } else {
            break;
        }
    }
    cursor
}

/// Decode one complete CSS identifier into a caller-provided buffer. The raw
/// token end is found separately, so a non-ASCII or overlong result can fail
/// lookup without exposing a suffix as a second identifier.
pub(super) fn decode_identifier(
    bytes: &[u8],
    start: usize,
    end: usize,
    output: &mut [u8],
) -> Option<usize> {
    let mut cursor = start;
    let mut output_len = 0;
    while cursor < end {
        let (byte, next) = if bytes[cursor] == b'\\' {
            decoded_escape(bytes, cursor, end)
        } else {
            (Some(bytes[cursor]), cursor + 1)
        };
        let byte = byte?;
        *output.get_mut(output_len)? = byte;
        output_len += 1;
        cursor = next;
    }
    Some(output_len)
}

/// Functions whose arguments cannot contain a CSS colour value. URLs need
/// opaque treatment; math functions only accept numeric calculations.
pub(super) fn skipped_function_end(content: &str, start: usize, limit: usize) -> Option<usize> {
    const SKIPPED: &[&[u8]] = &[
        b"url", b"calc", b"min", b"max", b"clamp", b"round", b"mod", b"rem", b"sin", b"cos",
        b"tan", b"asin", b"acos", b"atan", b"atan2", b"pow", b"sqrt", b"hypot", b"log", b"exp",
        b"abs", b"sign",
    ];
    let bytes = content.as_bytes();
    let identifier_end = identifier_end(bytes, start, limit);
    if bytes.get(identifier_end) != Some(&b'(')
        || !SKIPPED
            .iter()
            .any(|expected| identifier_eq(bytes, start, identifier_end, expected))
    {
        return None;
    }
    Some(skip_function(bytes, identifier_end, limit))
}

fn escape_end(bytes: &[u8], slash: usize, limit: usize) -> usize {
    let mut cursor = slash + 1;
    let mut digits = 0;
    while cursor < limit && digits < 6 && bytes[cursor].is_ascii_hexdigit() {
        cursor += 1;
        digits += 1;
    }
    if digits > 0 {
        cursor = css_escape_whitespace_end(bytes, cursor, limit);
    } else {
        cursor = (cursor + 1).min(limit);
    }
    cursor
}

fn identifier_eq(bytes: &[u8], start: usize, end: usize, expected: &[u8]) -> bool {
    let mut cursor = start;
    let mut expected_cursor = 0;
    while cursor < end {
        let (byte, next) = if bytes[cursor] == b'\\' {
            decoded_escape(bytes, cursor, end)
        } else {
            (Some(bytes[cursor]), cursor + 1)
        };
        let Some(byte) = byte else {
            return false;
        };
        if expected.get(expected_cursor) != Some(&byte.to_ascii_lowercase()) {
            return false;
        }
        expected_cursor += 1;
        cursor = next;
    }
    expected_cursor == expected.len()
}

fn decoded_escape(bytes: &[u8], slash: usize, limit: usize) -> (Option<u8>, usize) {
    let mut cursor = slash + 1;
    let mut value = 0u32;
    let mut digits = 0;
    while cursor < limit && digits < 6 && bytes[cursor].is_ascii_hexdigit() {
        value = value * 16 + u32::from(hex_value(bytes[cursor]));
        cursor += 1;
        digits += 1;
    }
    if digits > 0 {
        cursor = css_escape_whitespace_end(bytes, cursor, limit);
        (u8::try_from(value).ok(), cursor)
    } else {
        (bytes.get(cursor).copied(), (cursor + 1).min(limit))
    }
}

fn css_escape_whitespace_end(bytes: &[u8], cursor: usize, limit: usize) -> usize {
    match bytes.get(cursor).copied() {
        Some(b'\r') if bytes.get(cursor + 1) == Some(&b'\n') => (cursor + 2).min(limit),
        Some(b'\t' | b'\n' | b'\x0c' | b'\r' | b' ') => cursor + 1,
        _ => cursor,
    }
}

fn hex_value(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => 0,
    }
}

pub(super) fn skip_function(bytes: &[u8], open: usize, limit: usize) -> usize {
    function_end(bytes, open, limit).unwrap_or(limit)
}

pub(super) fn function_end(bytes: &[u8], open: usize, limit: usize) -> Option<usize> {
    let mut cursor = open;
    let mut depth = 0usize;
    while cursor < limit {
        if pair_at(bytes, cursor, b"/*") {
            cursor = comment_end(bytes, cursor, limit)?;
            continue;
        }
        if matches!(bytes[cursor], b'\'' | b'"') {
            cursor = skip_string(bytes, cursor, limit);
            continue;
        }
        match bytes[cursor] {
            b'\\' => cursor = escape_end(bytes, cursor, limit),
            b'(' => {
                depth += 1;
                cursor += 1;
            }
            b')' => {
                depth = depth.saturating_sub(1);
                cursor += 1;
                if depth == 0 {
                    return Some(cursor);
                }
            }
            _ => cursor += 1,
        }
    }
    None
}
