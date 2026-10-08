//! Borrowed CSS cursor classification; no stylesheet AST or buffer allocation.
//!
//! Lexical state starts at the resident style block's known beginning. A suffix
//! delimiter cannot prove that a quote/comment did not start outside a window,
//! so requests beyond the explicit prefix budget conservatively fall back.

pub(super) const MAX_LOOKBACK: usize = 64 * 1024;
const MAX_LOOKAHEAD: usize = 1024;
const MAX_NESTING: usize = 64;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum CssContext<'a> {
    Property {
        prefix: &'a str,
        span: (usize, usize),
        has_colon: bool,
    },
    Value {
        property: &'a str,
        prefix: &'a str,
        span: (usize, usize),
    },
    Pseudo {
        prefix: &'a str,
        span: (usize, usize),
    },
    AtRule {
        prefix: &'a str,
        span: (usize, usize),
    },
    Unknown,
}

/// Classify a relative byte cursor. Spans include the whole replacement token;
/// prefixes stop at the cursor. `:`/`::` and `@` are part of catalog tokens.
pub(super) fn at(content: &str, offset: usize) -> CssContext<'_> {
    if offset > MAX_LOOKBACK || !content.is_char_boundary(offset) {
        return CssContext::Unknown;
    }
    let bytes = content.as_bytes();
    let mut blocks = [false; MAX_NESTING];
    let (mut depth, mut parens, mut start, mut colon, mut escaped) = (0, 0, 0, None, false);
    let mut i = 0;
    while i < offset {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let Some(end) = comment_end(bytes, i + 2, offset) else {
                    return CssContext::Unknown;
                };
                i = end;
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                if i + 2 > offset {
                    return CssContext::Unknown;
                }
                let Some(end) = bytes[i + 2..offset].iter().position(|&b| b == b'\n') else {
                    return CssContext::Unknown;
                };
                i += end + 3;
                continue;
            }
            quote @ (b'\'' | b'"') => {
                i += 1;
                loop {
                    if i >= offset {
                        return CssContext::Unknown;
                    }
                    if bytes[i] == b'\\' {
                        i += 2;
                    } else if bytes[i] == quote {
                        i += 1;
                        break;
                    } else {
                        i += 1;
                    }
                }
                continue;
            }
            b'\\' => {
                escaped = true;
                i += 2;
                if i > offset {
                    return CssContext::Unknown;
                }
                continue;
            }
            b'(' => {
                let mut name = i;
                while name > start && is_name(bytes[name - 1]) {
                    name -= 1;
                }
                if bytes[name..i].eq_ignore_ascii_case(b"url") {
                    let Some(end) = url_end(bytes, i + 1, offset) else {
                        return CssContext::Unknown;
                    };
                    i = end;
                    continue;
                }
                parens += 1;
            }
            b')' if parens > 0 => parens -= 1,
            b')' => return CssContext::Unknown,
            b'{' if parens == 0 => {
                if depth == MAX_NESTING {
                    return CssContext::Unknown;
                }
                blocks[depth] = declaration_block(&content[start..i]);
                depth += 1;
                start = i + 1;
                colon = None;
                escaped = false;
            }
            b'}' if parens == 0 => {
                if depth == 0 {
                    return CssContext::Unknown;
                }
                depth -= 1;
                start = i + 1;
                colon = None;
                escaped = false;
            }
            b';' if parens == 0 => {
                start = i + 1;
                colon = None;
                escaped = false;
            }
            b':' if parens == 0 && colon.is_none() => colon = Some(i),
            _ => {}
        }
        i += 1;
    }
    if parens > 0 || escaped {
        return CssContext::Unknown;
    }
    let Some((mut token_start, token_end)) = token_span(bytes, offset) else {
        return CssContext::Unknown;
    };
    let declarations = depth > 0 && blocks[depth - 1];
    if bytes[token_start..token_end].starts_with(b"--")
        || token_start > start
            && bytes[token_start - 1].is_ascii()
            && matches!(bytes[token_start - 1], b'$' | b'#' | b'\\')
        || token_start > start && !bytes[token_start - 1].is_ascii()
    {
        return CssContext::Unknown;
    }
    if let Some(at) = token_start.checked_sub(1).filter(|&p| bytes[p] == b'@') {
        if trivia_end(bytes, start, at) == Some(at) {
            token_start = at;
            return CssContext::AtRule {
                prefix: &content[token_start..offset],
                span: (token_start, token_end),
            };
        }
        return CssContext::Unknown;
    }
    if let Some(colon) = colon {
        if declarations && !following_rule(bytes, token_end) {
            if let Some(property) = property_name(&content[start..colon]) {
                if token_start > colon + 1 && bytes[token_start - 1] == b':' {
                    return CssContext::Unknown;
                }
                return CssContext::Value {
                    property,
                    prefix: &content[token_start..offset],
                    span: (token_start, token_end),
                };
            }
            let head = &bytes[start..colon];
            if trivia_end(head, 0, head.len())
                .is_some_and(|p| head[p..].starts_with(b"--") || head.get(p) == Some(&b'$'))
            {
                return CssContext::Unknown;
            }
        }
    }
    if token_start > start && bytes[token_start - 1] == b':' {
        token_start -= 1;
        if token_start > start && bytes[token_start - 1] == b':' {
            token_start -= 1;
        }
        return CssContext::Pseudo {
            prefix: &content[token_start..offset],
            span: (token_start, token_end),
        };
    }
    if declarations && colon.is_none() && trivia_end(bytes, start, token_start) == Some(token_start)
    {
        let limit = token_end.saturating_add(MAX_LOOKAHEAD).min(bytes.len());
        let Some(next) = trivia_end(bytes, token_end, limit) else {
            return CssContext::Unknown;
        };
        return CssContext::Property {
            prefix: &content[token_start..offset],
            span: (token_start, token_end),
            has_colon: bytes.get(next) == Some(&b':'),
        };
    }
    CssContext::Unknown
}

fn is_name(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
}

/// Unquoted URLs contain literal `//` and `/*`, not comment openers. Quoted
/// URLs and escaped closing parentheses still have their normal lexical rules.
fn url_end(bytes: &[u8], mut i: usize, limit: usize) -> Option<usize> {
    let mut quote = None;
    while i < limit {
        match bytes[i] {
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

fn token_span(bytes: &[u8], offset: usize) -> Option<(usize, usize)> {
    let mut start = offset;
    while start > 0 && is_name(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = offset;
    let limit = offset.saturating_add(MAX_LOOKAHEAD).min(bytes.len());
    while end < limit && is_name(bytes[end]) {
        end += 1;
    }
    if bytes.get(end).is_some_and(|&b| is_name(b) || !b.is_ascii()) {
        return None;
    }
    Some((start, end))
}

fn comment_end(bytes: &[u8], mut i: usize, limit: usize) -> Option<usize> {
    while i + 1 < limit {
        if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            return Some(i + 2);
        }
        i += 1;
    }
    None
}

fn trivia_end(bytes: &[u8], mut i: usize, limit: usize) -> Option<usize> {
    while i < limit {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
        } else if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            i = comment_end(bytes, i + 2, limit)?;
        } else if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'/') {
            i += 2;
            while i < limit && bytes[i] != b'\n' {
                i += 1;
            }
            if i >= limit {
                return None;
            }
        } else {
            break;
        }
    }
    if i == limit && limit < bytes.len() && bytes[i].is_ascii_whitespace() {
        return None;
    }
    Some(i)
}

fn property_name(head: &str) -> Option<&str> {
    let bytes = head.as_bytes();
    let start = trivia_end(bytes, 0, bytes.len())?;
    if !bytes
        .get(start)
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'-')
        || bytes[start..].starts_with(b"--")
    {
        return None;
    }
    let mut end = start;
    while end < bytes.len() && is_name(bytes[end]) {
        end += 1;
    }
    (trivia_end(bytes, end, bytes.len())? == bytes.len()).then_some(&head[start..end])
}

fn declaration_block(head: &str) -> bool {
    let bytes = head.as_bytes();
    let Some(start) = trivia_end(bytes, 0, bytes.len()) else {
        return false;
    };
    if bytes.get(start) != Some(&b'@') {
        return !head.ends_with('#');
    }
    let mut end = start + 1;
    while end < bytes.len() && is_name(bytes[end]) {
        end += 1;
    }
    ["font-face", "page", "property", "counter-style"]
        .iter()
        .any(|name| head[start + 1..end].eq_ignore_ascii_case(name))
}

/// A completed nested selector is distinguishable from `property: value`.
/// Incomplete ambiguous type selectors retain the conservative value context.
fn following_rule(bytes: &[u8], mut i: usize) -> bool {
    let limit = i.saturating_add(MAX_LOOKAHEAD).min(bytes.len());
    while i < limit {
        match bytes[i] {
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

#[cfg(test)]
#[path = "context/tests.rs"]
mod tests;
