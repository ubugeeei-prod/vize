//! Low-level utility functions for template parsing.
//!
//! Provides byte-level helpers for tag parsing, whitespace detection,
//! and HTML void element recognition.

use vize_carton::{String, ToCompactString};

/// Find a byte subsequence in a slice.
pub(crate) fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Parse a closing tag, returns `(tag_name, end_pos)`.
pub(crate) fn parse_closing_tag(source: &[u8], start: usize) -> Option<(String, usize)> {
    let len = source.len();
    let mut pos = start + 2; // skip '</'

    let tag_start = pos;
    while pos < len && is_tag_name_char(source[pos]) {
        pos += 1;
    }
    if pos == tag_start {
        return None;
    }

    let tag_name = std::str::from_utf8(&source[tag_start..pos])
        .unwrap_or("")
        .to_compact_string();

    // Skip whitespace and find '>'
    while pos < len && source[pos] != b'>' {
        pos += 1;
    }
    if pos < len && source[pos] == b'>' {
        pos += 1;
    }

    Some((tag_name, pos))
}

/// Check if a byte is a valid tag name character.
#[inline(always)]
pub(crate) fn is_tag_name_char(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b':')
}

/// Check if a byte is whitespace.
#[inline(always)]
pub(crate) fn is_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

/// Check if an element is a void element (self-closing in HTML).
pub(crate) fn is_void_element_str(tag: &str) -> bool {
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
