//! Selector helpers for scoped CSS rewriting.

use super::transform::find_matching_paren;

pub(super) fn split_before_trailing_universal_or_pseudo(
    selector: &str,
) -> Option<(&str, &str, &str)> {
    let (prefix_end, suffix_start) = trailing_compound_boundary(selector)?;
    let suffix = selector[suffix_start..].trim_start();
    if suffix.is_empty() || !scopes_previous_compound(suffix) {
        return None;
    }

    Some((
        &selector[..prefix_end],
        &selector[prefix_end..suffix_start],
        suffix,
    ))
}

fn trailing_compound_boundary(selector: &str) -> Option<(usize, usize)> {
    let bytes = selector.as_bytes();
    let mut depth = 0i32;
    let mut i = 0usize;
    let mut last_boundary = None;

    while i < bytes.len() {
        match bytes[i] {
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
                if !selector[..boundary_start].trim().is_empty()
                    && !prefix_ends_with_combinator(&selector[..boundary_start])
                    && suffix_start < bytes.len()
                {
                    last_boundary = Some((boundary_start, suffix_start));
                }
                i = suffix_start;
            }
            b'>' | b'+' | b'~' if depth == 0 => {
                let boundary_start = trim_ascii_whitespace_end(selector, i);
                let suffix_start = skip_ascii_whitespace(selector, i + 1);
                if !selector[..boundary_start].trim().is_empty() && suffix_start < bytes.len() {
                    last_boundary = Some((boundary_start, suffix_start));
                }
                i += 1;
            }
            b'|' if depth == 0 && i + 1 < bytes.len() && bytes[i + 1] == b'|' => {
                let boundary_start = trim_ascii_whitespace_end(selector, i);
                let suffix_start = skip_ascii_whitespace(selector, i + 2);
                if !selector[..boundary_start].trim().is_empty() && suffix_start < bytes.len() {
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

fn prefix_ends_with_combinator(value: &str) -> bool {
    let trimmed = value.trim_end();
    trimmed.ends_with('>')
        || trimmed.ends_with('+')
        || trimmed.ends_with('~')
        || trimmed.ends_with("||")
}

fn trim_ascii_whitespace_end(value: &str, end: usize) -> usize {
    let bytes = value.as_bytes();
    let mut cursor = end;
    while cursor > 0 && bytes[cursor - 1].is_ascii_whitespace() {
        cursor -= 1;
    }
    cursor
}

fn skip_ascii_whitespace(value: &str, start: usize) -> usize {
    let bytes = value.as_bytes();
    let mut cursor = start;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    cursor
}

fn scopes_previous_compound(selector: &str) -> bool {
    if let Some(end) = leading_universal_selector_end(selector) {
        let rest = &selector[end..];
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
    while i < bytes.len() {
        match bytes[i] {
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
    while i < bytes.len() {
        if bytes[i] != b':' {
            return false;
        }

        i += 1;
        if bytes.get(i) == Some(&b':') {
            i += 1;
        }

        let ident_start = i;
        while i < bytes.len() {
            match bytes[i] {
                b'_' | b'-' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => i += 1,
                _ => break,
            }
        }
        if i == ident_start {
            return false;
        }

        if bytes.get(i) == Some(&b'(') {
            let Some(end) = find_matching_paren(&selector[i + 1..]) else {
                return false;
            };
            i += end + 2;
        }
    }

    true
}

pub(super) fn find_top_level_pseudo(selector: &str) -> Option<usize> {
    let bytes = selector.as_bytes();
    let mut depth: i32 = 0;
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b':' if depth == 0 => return Some(i),
            _ => {}
        }
        i += 1;
    }

    None
}
