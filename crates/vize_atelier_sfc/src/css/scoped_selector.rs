//! Selector helpers for scoped CSS rewriting.

use super::transform::find_matching_paren;

pub(crate) fn split_before_trailing_universal_or_pseudo(
    selector: &str,
) -> Option<(&str, &str, &str)> {
    let (prefix_end, suffix_start) = trailing_compound_boundary(selector)?;
    let (prefix, rest) = selector.split_at_checked(prefix_end)?;
    let (boundary, suffix) = rest.split_at_checked(suffix_start.checked_sub(prefix_end)?)?;
    let suffix = suffix.trim_start();
    if suffix.is_empty() || !scopes_previous_compound(suffix) {
        return None;
    }

    Some((prefix, boundary, suffix))
}

fn trailing_compound_boundary(selector: &str) -> Option<(usize, usize)> {
    let bytes = selector.as_bytes();
    let mut depth = 0i32;
    let mut i = 0usize;
    let mut last_boundary = None;

    while let Some(&byte) = bytes.get(i) {
        match byte {
            b'(' | b'[' => {
                depth += 1;
                i += 1;
            }
            b')' | b']' => {
                depth -= 1;
                i += 1;
            }
            b' ' | b'\t' | b'\n' | b'\r' if depth == 0 => {
                let boundary_start = trim_ascii_whitespace_end(selector, i);
                let suffix_start = skip_ascii_whitespace(selector, i + 1);
                if !prefix_is_blank(selector, boundary_start)
                    && !prefix_ends_with_combinator(
                        selector.get(..boundary_start).unwrap_or_default(),
                    )
                    && suffix_start < bytes.len()
                {
                    last_boundary = Some((boundary_start, suffix_start));
                }
                i = suffix_start;
            }
            b'>' | b'+' | b'~' if depth == 0 => {
                let boundary_start = trim_ascii_whitespace_end(selector, i);
                let suffix_start = skip_ascii_whitespace(selector, i + 1);
                if !prefix_is_blank(selector, boundary_start) && suffix_start < bytes.len() {
                    last_boundary = Some((boundary_start, suffix_start));
                }
                i += 1;
            }
            b'|' if depth == 0 && bytes.get(i + 1) == Some(&b'|') => {
                let boundary_start = trim_ascii_whitespace_end(selector, i);
                let suffix_start = skip_ascii_whitespace(selector, i + 2);
                if !prefix_is_blank(selector, boundary_start) && suffix_start < bytes.len() {
                    last_boundary = Some((boundary_start, suffix_start));
                }
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    last_boundary
}

fn prefix_is_blank(selector: &str, end: usize) -> bool {
    selector
        .get(..end)
        .is_none_or(|prefix| prefix.trim().is_empty())
}

fn prefix_ends_with_combinator(value: &str) -> bool {
    let trimmed = value.trim_end();
    trimmed.ends_with('>')
        || trimmed.ends_with('+')
        || trimmed.ends_with('~')
        || trimmed.ends_with("||")
}

fn trim_ascii_whitespace_end(value: &str, end: usize) -> usize {
    let bytes = value.as_bytes().get(..end).unwrap_or_default();
    bytes.len()
        - bytes
            .iter()
            .rev()
            .take_while(|b| b.is_ascii_whitespace())
            .count()
}

fn skip_ascii_whitespace(value: &str, start: usize) -> usize {
    let bytes = value.as_bytes();
    let tail = bytes.get(start..).unwrap_or_default();
    start + tail.iter().take_while(|b| b.is_ascii_whitespace()).count()
}

fn scopes_previous_compound(selector: &str) -> bool {
    if let Some(end) = leading_universal_selector_end(selector) {
        let rest = selector.get(end..).unwrap_or_default();
        return rest.is_empty() || parse_pseudo_sequence(rest);
    }

    parse_pseudo_sequence(selector)
}

pub(super) fn leading_universal_selector_end(selector: &str) -> Option<usize> {
    let bytes = selector.as_bytes();
    if bytes.first() == Some(&b'*') {
        if bytes.get(1) == Some(&b'|') && bytes.get(2) == Some(&b'*') {
            return Some(3);
        }
        return Some(1);
    }

    if bytes.first() == Some(&b'|') && bytes.get(1) == Some(&b'*') {
        return Some(2);
    }

    let mut i = 0usize;
    while let Some(&byte) = bytes.get(i) {
        match byte {
            b'|' if bytes.get(i + 1) == Some(&b'*') => return Some(i + 2),
            b'_' | b'-' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => i += 1,
            _ => return None,
        }
    }

    None
}

fn parse_pseudo_sequence(selector: &str) -> bool {
    if selector.is_empty() {
        return false;
    }

    let bytes = selector.as_bytes();
    let mut i = 0usize;
    while let Some(&byte) = bytes.get(i) {
        if byte != b':' {
            return false;
        }

        i += 1;
        if bytes.get(i) == Some(&b':') {
            i += 1;
        }

        let ident_start = i;
        while let Some(&byte) = bytes.get(i) {
            match byte {
                b'_' | b'-' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => i += 1,
                _ => break,
            }
        }
        if i == ident_start {
            return false;
        }

        if bytes.get(i) == Some(&b'(') {
            let Some(end) = selector.get(i + 1..).and_then(find_matching_paren) else {
                return false;
            };
            i += end + 2;
        }
    }

    true
}

pub(super) fn find_top_level_pseudo(selector: &str) -> Option<usize> {
    let mut depth: i32 = 0;
    for (i, byte) in selector.bytes().enumerate() {
        match byte {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b':' if depth == 0 => return Some(i),
            _ => {}
        }
    }

    None
}
