//! Keep an unfinished template expression from making the virtual module unparseable.

#[path = "incomplete_delimiters.rs"]
mod delimiters;
use delimiters::delimiter_imbalance;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{String, cstr};

pub(crate) enum IsolatedExpression<'a> {
    Borrowed(&'a str),
    Owned {
        text: String,
        mapped_start: usize,
        mapped_len: usize,
    },
}

impl<'a> IsolatedExpression<'a> {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(text) => text,
            Self::Owned { text, .. } => text.as_str(),
        }
    }

    pub(crate) fn mapped_str(&self) -> &str {
        match self {
            Self::Borrowed(text) => text,
            Self::Owned {
                text,
                mapped_start,
                mapped_len,
            } => {
                let end = mapped_start.saturating_add(*mapped_len);
                text.as_str()
                    .get(*mapped_start..end)
                    .unwrap_or(text.as_str())
            }
        }
    }

    pub(crate) fn mapped_bounds(&self) -> (usize, usize) {
        match self {
            Self::Borrowed(text) => (0, text.len()),
            Self::Owned {
                mapped_start,
                mapped_len,
                ..
            } => (*mapped_start, *mapped_len),
        }
    }

    pub(crate) fn is_owned(&self) -> bool {
        matches!(self, Self::Owned { .. })
    }
}

pub(crate) fn isolate_incomplete_expression(source: &str) -> IsolatedExpression<'_> {
    let trimmed = source.trim();
    if trimmed.is_empty() || !looks_incomplete(trimmed) || expression_parses(trimmed) {
        return IsolatedExpression::Borrowed(source);
    }
    let (text, mapped_start, mapped_len) = repair_incomplete(trimmed);
    IsolatedExpression::Owned {
        text,
        mapped_start,
        mapped_len,
    }
}

fn repair_incomplete(trimmed: &str) -> (String, usize, usize) {
    if let Some((open, close, inner)) = outermost_inner(trimmed) {
        let (repaired_inner, inner_start, inner_len) =
            if looks_incomplete(inner.as_str()) && !expression_parses(inner.as_str()) {
                repair_incomplete(inner.as_str())
            } else {
                let len = inner.len();
                (inner, 0, len)
            };
        let mut wrapped = String::from("");
        wrapped.push(open);
        wrapped.push_str(repaired_inner.as_str());
        wrapped.push(close);
        if expression_parses(wrapped.as_str()) {
            let placeholder = inner_start + inner_len < repaired_inner.len();
            if placeholder {
                return (wrapped, 1 + inner_start, inner_len);
            }
            let len = wrapped.len();
            return (wrapped, 0, len);
        }
    }
    if let Some(repaired) = member_placeholder(trimmed) {
        return repaired;
    }
    let mut current = String::from(trimmed);
    let limit = trimmed.len();
    for _ in 0..limit {
        let before = current.len();
        if !strip_incomplete_tail(&mut current) || current.len() >= before {
            break;
        }
        let candidate = current.trim();
        if candidate.is_empty() {
            break;
        }
        if expression_parses(candidate) {
            let repaired = String::from(candidate);
            let len = repaired.len();
            return (repaired, 0, len);
        }
        if candidate.len() != current.len() {
            current = String::from(candidate);
        }
    }
    let fallback = String::from("undefined");
    let len = fallback.len();
    (fallback, 0, len)
}

fn member_placeholder(trimmed: &str) -> Option<(String, usize, usize)> {
    let bytes = trimmed.as_bytes();
    if bytes.last() != Some(&b'.') {
        return None;
    }
    let prev = bytes.get(bytes.len().saturating_sub(2))?;
    if *prev == b'.' || prev.is_ascii_digit() {
        return None;
    }
    let mut candidate = String::from(trimmed);
    candidate.push_str(" x");
    if !expression_parses(candidate.as_str()) {
        return None;
    }
    let mapped_len = trimmed.len();
    Some((candidate, 0, mapped_len))
}

fn outermost_inner(source: &str) -> Option<(char, char, String)> {
    let mut chars = source.chars();
    let open = chars.next()?;
    let close = match open {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => return None,
    };
    if !source.ends_with(close) || !wrapper_covers_all(source, open, close) {
        return None;
    }
    let mut inner = String::from("");
    let mut body = source.chars();
    body.next();
    body.next_back();
    for ch in body {
        inner.push(ch);
    }
    let trimmed = inner.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some((open, close, String::from(trimmed)))
}

fn wrapper_covers_all(source: &str, open: char, close: char) -> bool {
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let last = source.chars().count().saturating_sub(1);
    for (index, ch) in source.chars().enumerate() {
        if let Some(active) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == active {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '\'' | '"' | '`') {
            quote = Some(ch);
            continue;
        }
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return index == last;
            }
            if depth < 0 {
                return false;
            }
        }
    }
    false
}

fn expression_parses(expr: &str) -> bool {
    let allocator = Allocator::default();
    let wrapped = cstr!("void ({expr});\n");
    let parsed = Parser::new(&allocator, wrapped.as_str(), SourceType::ts()).parse();
    !parsed.panicked && parsed.diagnostics.is_empty()
}

fn looks_incomplete(trimmed: &str) -> bool {
    delimiter_imbalance(trimmed)
        || ends_with_dangling_operator(trimmed)
        || has_trailing_keyword_before_closers(trimmed)
}

fn ends_with_dangling_operator(source: &str) -> bool {
    let mut reversed = source.chars().rev().peekable();
    while let Some(ch) = reversed.next() {
        if ch.is_whitespace() || matches!(ch, ')' | ']' | '}') {
            continue;
        }
        if ch == '.' {
            let previous_is_digit = reversed.peek().is_some_and(|prev| prev.is_ascii_digit());
            if previous_is_digit {
                return false;
            }
            return true;
        }
        return is_dangling_operator(ch);
    }
    false
}

fn is_dangling_operator(ch: char) -> bool {
    matches!(
        ch,
        '?' | '+'
            | '-'
            | '*'
            | '/'
            | '%'
            | '='
            | '!'
            | '<'
            | '>'
            | '&'
            | '|'
            | '^'
            | '~'
            | ':'
            | ','
    )
}

fn has_trailing_keyword_before_closers(source: &str) -> bool {
    has_trailing_keyword(
        source.trim_end_matches(|ch: char| ch.is_whitespace() || matches!(ch, ')' | ']' | '}')),
    )
}

fn has_trailing_keyword(trimmed: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "instanceof",
        "typeof",
        "await",
        "delete",
        "yield",
        "void",
        "new",
        "in",
        "of",
        "as",
    ];
    KEYWORDS.iter().any(|keyword| {
        if !trimmed.ends_with(keyword) {
            return false;
        }
        !trimmed
            .chars()
            .nth_back(keyword.chars().count())
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
    })
}

fn strip_incomplete_tail(current: &mut String) -> bool {
    let trimmed = current.trim_end();
    if trimmed.len() != current.len() {
        *current = String::from(trimmed);
    }
    if current.is_empty() {
        return false;
    }
    if let Some(stripped) = current.strip_suffix("?.") {
        *current = String::from(stripped);
        return true;
    }
    let Some(last) = current.chars().next_back() else {
        return false;
    };
    if is_tail_operator(last) {
        current.pop();
        return true;
    }
    if is_identifier_char(last) {
        while current.chars().next_back().is_some_and(is_identifier_char) {
            current.pop();
        }
        return true;
    }
    current.pop();
    true
}

fn is_tail_operator(ch: char) -> bool {
    matches!(
        ch,
        '.' | '?'
            | '+'
            | '-'
            | '*'
            | '/'
            | '%'
            | '='
            | '!'
            | '<'
            | '>'
            | '&'
            | '|'
            | '^'
            | '~'
            | ':'
            | ','
            | '('
            | '['
            | '{'
    )
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}

#[cfg(test)]
mod tests;
