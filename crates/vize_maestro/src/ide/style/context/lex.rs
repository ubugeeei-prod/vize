//! Bounded lexical helpers for borrowed CSS cursor classification.

pub(super) const MAX_LOOKAHEAD: usize = 1024;

pub(super) fn is_name(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
}

/// Unquoted URLs contain literal `//` and `/*`, not comment openers. Quoted
/// URLs and escaped closing parentheses still have their normal lexical rules.
pub(super) fn url_end(bytes: &[u8], mut i: usize, limit: usize) -> Option<usize> {
    let mut quote = None;
    while i < limit {
        match *bytes.get(i)? {
            b'\\' => i += 2,
            byte if quote == Some(byte) => {
                quote = None;
                i += 1;
            }
            b')' if quote.is_none() => return Some(i + 1),
            byte @ (b'\'' | b'"') if quote.is_none() => {
                quote = Some(byte);
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

pub(super) fn token_span(bytes: &[u8], offset: usize) -> Option<(usize, usize)> {
    let mut start = offset;
    while start > 0 && bytes.get(start - 1).is_some_and(|&b| is_name(b)) {
        start -= 1;
    }
    let mut end = offset;
    let limit = offset.saturating_add(MAX_LOOKAHEAD).min(bytes.len());
    while end < limit && bytes.get(end).is_some_and(|&b| is_name(b)) {
        end += 1;
    }
    if bytes.get(end).is_some_and(|&b| is_name(b) || !b.is_ascii()) {
        return None;
    }
    Some((start, end))
}

pub(super) fn comment_end(bytes: &[u8], mut i: usize, limit: usize) -> Option<usize> {
    while i + 1 < limit {
        if bytes.get(i) == Some(&b'*') && bytes.get(i + 1) == Some(&b'/') {
            return Some(i + 2);
        }
        i += 1;
    }
    None
}

pub(super) fn trivia_end(bytes: &[u8], mut i: usize, limit: usize) -> Option<usize> {
    while i < limit {
        if bytes.get(i)?.is_ascii_whitespace() {
            i += 1;
        } else if bytes.get(i) == Some(&b'/') && bytes.get(i + 1) == Some(&b'*') {
            i = comment_end(bytes, i + 2, limit)?;
        } else if bytes.get(i) == Some(&b'/') && bytes.get(i + 1) == Some(&b'/') {
            i += 2;
            while i < limit && bytes.get(i) != Some(&b'\n') {
                i += 1;
            }
            if i >= limit {
                return None;
            }
        } else {
            break;
        }
    }
    if i == limit && limit < bytes.len() && bytes.get(i)?.is_ascii_whitespace() {
        return None;
    }
    Some(i)
}

pub(super) fn property_name(head: &str) -> Option<&str> {
    let bytes = head.as_bytes();
    let start = trivia_end(bytes, 0, bytes.len())?;
    if !bytes
        .get(start)
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'-')
        || bytes.get(start..)?.starts_with(b"--")
    {
        return None;
    }
    let mut end = start;
    while end < bytes.len() && bytes.get(end).is_some_and(|&b| is_name(b)) {
        end += 1;
    }
    (trivia_end(bytes, end, bytes.len())? == bytes.len())
        .then(|| head.get(start..end))
        .flatten()
}

pub(super) fn declaration_block(head: &str) -> bool {
    let bytes = head.as_bytes();
    let Some(start) = trivia_end(bytes, 0, bytes.len()) else {
        return false;
    };
    if bytes.get(start) != Some(&b'@') {
        return !head.ends_with('#');
    }
    let mut end = start + 1;
    while end < bytes.len() && bytes.get(end).is_some_and(|&b| is_name(b)) {
        end += 1;
    }
    ["font-face", "page", "property", "counter-style"]
        .iter()
        .any(|name| {
            head.get(start + 1..end)
                .is_some_and(|header| header.eq_ignore_ascii_case(name))
        })
}

/// A completed nested selector is distinguishable from `property: value`.
/// Incomplete ambiguous type selectors retain the conservative value context.
pub(super) fn following_rule(bytes: &[u8], mut i: usize) -> bool {
    let limit = i.saturating_add(MAX_LOOKAHEAD).min(bytes.len());
    while i < limit {
        let Some(&byte) = bytes.get(i) else {
            return false;
        };
        match byte {
            b'{' => return true,
            b';' | b'}' | b'\'' | b'"' | b'\\' => return false,
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let Some(end) = comment_end(bytes, i + 2, limit) else {
                    return false;
                };
                i = end;
                continue;
            }
            _ => i += 1,
        }
    }
    false
}
