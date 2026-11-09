//! HTML tag boundaries inside a root `<template>` block.

use super::super::block::{find_closing_tag_end, is_whitespace_fast, starts_with_bytes};
use memchr::{memchr, memchr3};

#[inline]
pub(super) fn is_opening_tag_named(
    bytes: &[u8],
    pos: usize,
    len: usize,
    expected_name: &[u8],
) -> bool {
    let name_start = pos + 1;
    let name_end = name_start + expected_name.len();
    name_end < len
        && starts_with_bytes(bytes.get(name_start..).unwrap_or_default(), expected_name)
        && bytes
            .get(name_end)
            .is_some_and(|&b| is_whitespace_fast(b) || b == b'/' || b == b'>')
}

#[inline]
pub(super) fn raw_text_tag_name(bytes: &[u8], pos: usize, len: usize) -> Option<&'static [u8]> {
    match bytes.get(pos + 1)?.to_ascii_lowercase() {
        b's' if is_opening_tag_named(bytes, pos, len, b"script") => Some(b"script"),
        b's' if is_opening_tag_named(bytes, pos, len, b"style") => Some(b"style"),
        b't' if is_opening_tag_named(bytes, pos, len, b"textarea") => Some(b"textarea"),
        b't' if is_opening_tag_named(bytes, pos, len, b"title") => Some(b"title"),
        _ => None,
    }
}

pub(super) fn find_raw_text_element_end(
    bytes: &[u8],
    mut pos: usize,
    len: usize,
    tag_name: &[u8],
) -> Option<usize> {
    while pos < len {
        let lt_offset = memchr(b'<', bytes.get(pos..).unwrap_or_default())?;
        pos += lt_offset;
        if let Some(end_tag_pos) = find_closing_tag_end(bytes, pos, len, tag_name) {
            return Some(end_tag_pos);
        }
        pos += 1;
    }
    None
}

/// Find the end of an HTML opening tag without treating `>` inside a quoted
/// attribute as the tag boundary. The returned position is immediately after
/// `>` and the boolean records whether the tag is self-closing.
#[inline]
pub(super) fn find_opening_tag_end(bytes: &[u8], pos: usize, len: usize) -> Option<(usize, bool)> {
    debug_assert_eq!(bytes.get(pos), Some(&b'<'));
    let mut cursor = pos + 2;

    while cursor < len {
        let candidate = memchr3(b'>', b'"', b'\'', bytes.get(cursor..).unwrap_or_default())?;
        cursor += candidate;

        match bytes.get(cursor) {
            Some(b'>') => {
                let mut before_end = cursor;
                while before_end > pos + 1
                    && bytes
                        .get(before_end - 1)
                        .is_some_and(|&b| is_whitespace_fast(b))
                {
                    before_end -= 1;
                }
                let self_closing = before_end > pos + 1 && bytes.get(before_end - 1) == Some(&b'/');
                return Some((cursor + 1, self_closing));
            }
            Some(&quote @ (b'"' | b'\'')) => {
                cursor += 1;
                let closing_quote = memchr(quote, bytes.get(cursor..).unwrap_or_default())?;
                cursor += closing_quote + 1;
            }
            _ => return None,
        }
    }

    None
}
