//! Borrowed CSS cursor classification; no stylesheet AST or buffer allocation.
//!
//! Lexical state starts at the resident style block's known beginning. A suffix
//! delimiter cannot prove that a quote/comment did not start outside a window,
//! so requests beyond the explicit prefix budget conservatively fall back.

#[path = "context/lex.rs"]
mod lex;
use lex::{
    MAX_LOOKAHEAD, comment_end, declaration_block, following_rule, is_name, property_name,
    token_span, trivia_end, url_end,
};

pub(super) const MAX_LOOKBACK: usize = 64 * 1024;
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
    classify(content, offset).unwrap_or(CssContext::Unknown)
}

fn classify(content: &str, offset: usize) -> Option<CssContext<'_>> {
    if offset > MAX_LOOKBACK || !content.is_char_boundary(offset) {
        return None;
    }
    let bytes = content.as_bytes();
    let mut blocks = [false; MAX_NESTING];
    let (mut depth, mut parens, mut start, mut colon, mut escaped) = (0, 0, 0, None, false);
    let mut i = 0;
    while i < offset {
        match *bytes.get(i)? {
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = comment_end(bytes, i + 2, offset)?;
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                let end = bytes.get(i + 2..offset)?.iter().position(|&b| b == b'\n')?;
                i += end + 3;
                continue;
            }
            quote @ (b'\'' | b'"') => {
                i += 1;
                loop {
                    if i >= offset {
                        return None;
                    }
                    if bytes.get(i) == Some(&b'\\') {
                        i += 2;
                    } else if bytes.get(i) == Some(&quote) {
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
                    return None;
                }
                continue;
            }
            b'(' => {
                let mut name = i;
                while name > start && bytes.get(name - 1).is_some_and(|&b| is_name(b)) {
                    name -= 1;
                }
                if bytes.get(name..i)?.eq_ignore_ascii_case(b"url") {
                    i = url_end(bytes, i + 1, offset)?;
                    continue;
                }
                parens += 1;
            }
            b')' if parens > 0 => parens -= 1,
            b')' => return None,
            b'{' if parens == 0 => {
                if depth == MAX_NESTING {
                    return None;
                }
                *blocks.get_mut(depth)? = declaration_block(content.get(start..i)?);
                depth += 1;
                start = i + 1;
                colon = None;
                escaped = false;
            }
            b'}' if parens == 0 => {
                if depth == 0 {
                    return None;
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
        return None;
    }
    let (mut token_start, token_end) = token_span(bytes, offset)?;
    let declarations = depth
        .checked_sub(1)
        .and_then(|index| blocks.get(index))
        .copied()
        .unwrap_or(false);
    let previous = token_start
        .checked_sub(1)
        .and_then(|index| bytes.get(index));
    if bytes.get(token_start..token_end)?.starts_with(b"--")
        || token_start > start
            && previous.is_some_and(|b| matches!(b, b'$' | b'#' | b'\\') || !b.is_ascii())
    {
        return None;
    }
    if let Some(at) = token_start
        .checked_sub(1)
        .filter(|&p| bytes.get(p) == Some(&b'@'))
    {
        if trivia_end(bytes, start, at) == Some(at) {
            token_start = at;
            return Some(CssContext::AtRule {
                prefix: content.get(token_start..offset)?,
                span: (token_start, token_end),
            });
        }
        return None;
    }
    if let Some(colon) = colon
        && declarations
        && !following_rule(bytes, token_end)
    {
        if let Some(property) = property_name(content.get(start..colon)?) {
            if token_start > colon + 1 && previous == Some(&b':') {
                return None;
            }
            return Some(CssContext::Value {
                property,
                prefix: content.get(token_start..offset)?,
                span: (token_start, token_end),
            });
        }
        let head = bytes.get(start..colon)?;
        if trivia_end(head, 0, head.len()).is_some_and(|p| {
            head.get(p..).is_some_and(|tail| tail.starts_with(b"--")) || head.get(p) == Some(&b'$')
        }) {
            return None;
        }
    }
    if token_start > start && previous == Some(&b':') {
        token_start -= 1;
        if token_start > start && bytes.get(token_start - 1) == Some(&b':') {
            token_start -= 1;
        }
        return Some(CssContext::Pseudo {
            prefix: content.get(token_start..offset)?,
            span: (token_start, token_end),
        });
    }
    if declarations && colon.is_none() && trivia_end(bytes, start, token_start) == Some(token_start)
    {
        let limit = token_end.saturating_add(MAX_LOOKAHEAD).min(bytes.len());
        let next = trivia_end(bytes, token_end, limit)?;
        return Some(CssContext::Property {
            prefix: content.get(token_start..offset)?,
            span: (token_start, token_end),
            has_colon: bytes.get(next) == Some(&b':'),
        });
    }
    None
}

#[cfg(test)]
#[path = "context/tests.rs"]
mod tests;
