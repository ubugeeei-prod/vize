//! Low-level utility functions for template parsing.
//!
//! Provides byte-level helpers for tag parsing, whitespace detection,
//! and HTML void element recognition.

use vize_s0::{String, ToCompactString};

/// Find a byte subsequence in a slice.
pub(crate) fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Parse a closing tag, returns `(tag_name, end_pos)`.
pub(crate) fn parse_closing_tag(source: &[u8], start: usize) -> Option<(String, usize)> {
    let len = source.len();
    let mut pos = start + 2; // skip '</'

    let tag_start = pos;
    while pos < len && source.get(pos).copied().is_some_and(is_tag_name_char) {
        pos += 1;
    }
    if pos == tag_start {
        return None;
    }

    let tag_name = std::str::from_utf8(source.get(tag_start..pos).unwrap_or_default())
        .unwrap_or("")
        .to_compact_string();

    // Skip whitespace and find '>'
    while source.get(pos).is_some_and(|&byte| byte != b'>') {
        pos += 1;
    }
    if source.get(pos) == Some(&b'>') {
        pos += 1;
    }

    Some((tag_name, pos))
}

/// Check if a byte is a valid tag name character.
#[inline(always)]
pub(crate) fn is_tag_name_char(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b':' | b'.')
}

/// The byte at `index`, or NUL past the end: NUL matches none of the
/// delimiters or character classes the template scanners test for.
#[inline]
pub(crate) fn byte_at(bytes: &[u8], index: usize) -> u8 {
    bytes.get(index).copied().unwrap_or(0)
}

/// `bytes[range]`, or empty when the range is out of bounds.
#[inline]
pub(crate) fn sub_slice<R: std::slice::SliceIndex<[u8], Output = [u8]>>(
    bytes: &[u8],
    range: R,
) -> &[u8] {
    bytes.get(range).unwrap_or_default()
}

/// Check if a byte is whitespace.
#[inline(always)]
pub(crate) fn is_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

/// Check if an element is a void element (self-closing in HTML).
///
/// Alloc-free: void element names are 2..=6 ASCII bytes, so a length guard
/// plus case-insensitive comparison avoids the lowercasing allocation.
///
/// Vue treats tags that start with an uppercase ASCII letter as components
/// (e.g. `<Link>`, `<Input>`), never as their HTML void-element namesakes.
/// Without that guard, `<Link>` would collapse to a void element here, so
/// the formatter would skip the child-depth increment and emit the
/// component's children flush with their parent. (#2244)
pub(crate) fn is_void_element_str(tag: &str) -> bool {
    const VOID_ELEMENTS: [&str; 14] = [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr",
    ];
    if tag.as_bytes().first().is_some_and(u8::is_ascii_uppercase) {
        return false;
    }
    matches!(tag.len(), 2..=6) && VOID_ELEMENTS.iter().any(|v| tag.eq_ignore_ascii_case(v))
}

pub(crate) fn template_literal_state_after_line_from(
    mut in_template_literal: bool,
    line: &str,
) -> bool {
    let bytes = line.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'`' && !is_escaped(bytes, index) {
            in_template_literal = !in_template_literal;
        }
    }
    in_template_literal
}

fn is_escaped(line: &[u8], pos: usize) -> bool {
    let mut backslashes = 0;
    let mut cursor = pos;
    while cursor > 0 && line.get(cursor - 1) == Some(&b'\\') {
        backslashes += 1;
        cursor -= 1;
    }
    backslashes % 2 == 1
}
