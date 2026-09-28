mod compat;
mod end;
mod literals;
mod open;
mod parse;
mod regex;

use alloc::borrow::Cow;
use memchr::memchr_iter;
use vize_l0::{String, cstr};

pub use super::attrs::AttrSink;

pub(super) use literals::{can_start_regex_literal, skip_script_string_literal};
pub use parse::parse_block_fast;
pub(super) use regex::skip_regex_literal;

// Tag name bytes for fast comparison
pub(super) const TAG_TEMPLATE: &[u8] = b"template";
const TAG_SCRIPT: &[u8] = b"script";
const TAG_STYLE: &[u8] = b"style";

pub type BlockParseOutput<'a> = (
    &'a [u8],     // tag name as bytes
    Cow<'a, str>, // content as borrowed string
    usize,        // content start
    usize,        // content end
    usize,        // end position
    usize,        // content end line
    usize,        // content end column
);
pub type BlockParseError = (&'static str, String);
pub type BlockParseResult<'a> = Result<Option<BlockParseOutput<'a>>, BlockParseError>;

pub(super) struct BlockEndSearch<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) source: &'a str,
    pub(super) tag_name: &'a [u8],
    pub(super) pos: usize,
    pub(super) content_start: usize,
    pub(super) start_line: usize,
    pub(super) start_column: usize,
    pub(super) initial_last_newline: usize,
}

/// Build a uniform `(code, message)` error for any malformed block.
pub(super) fn build_malformed_error(tag_name: &[u8], reason: &str) -> BlockParseError {
    let tag_str = core::str::from_utf8(tag_name).unwrap_or("unknown");
    (
        "MALFORMED_BLOCK",
        cstr!("Malformed <{tag_str}> block: {reason}."),
    )
}

/// Distinguish an incomplete opening tag from a missing block close.
pub(super) fn build_unterminated_open_error(tag_name: &[u8], reason: &str) -> BlockParseError {
    let (_, message) = build_malformed_error(tag_name, reason);
    ("UNTERMINATED_OPEN_TAG", message)
}

/// Fast tag name comparison using byte slices
#[inline(always)]
pub fn tag_name_eq(name: &[u8], expected: &[u8]) -> bool {
    name.len() == expected.len() && name.eq_ignore_ascii_case(expected)
}

/// Fast byte slice prefix check
#[inline(always)]
pub(super) fn starts_with_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .get(..needle.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(needle))
}

/// Fast tag name character check
#[inline(always)]
fn is_tag_name_char_fast(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_')
}

/// Fast whitespace check
#[inline(always)]
pub(super) fn is_whitespace_fast(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

#[inline]
pub(super) fn advance_line(bytes: &[u8], base: usize, line: &mut usize, last_newline: &mut usize) {
    for offset in memchr_iter(b'\n', bytes) {
        *line += 1;
        *last_newline = base + offset;
    }
}

#[inline]
fn position_after(bytes: &[u8], line: usize, column: usize) -> (usize, usize) {
    let mut end_line = line;
    let mut last_newline = None;
    for offset in memchr_iter(b'\n', bytes) {
        end_line += 1;
        last_newline = Some(offset);
    }
    let end_column = last_newline.map_or(column + bytes.len(), |offset| bytes.len() - offset);
    (end_line, end_column)
}

#[inline]
fn content_end_column(
    content_start: usize,
    start_line: usize,
    start_column: usize,
    content_end: usize,
    end_line: usize,
    last_newline: usize,
) -> usize {
    if end_line == start_line {
        start_column + content_end - content_start
    } else {
        content_end - last_newline
    }
}

/// Find the end of a closing tag `</tag_name` followed by optional whitespace and `>`.
/// Returns the position immediately after `>`, or `None` if no valid closing tag at `pos`.
#[inline]
pub(super) fn find_closing_tag_end(
    bytes: &[u8],
    pos: usize,
    len: usize,
    tag_name: &[u8],
) -> Option<usize> {
    // Need at least "</" + tag_name + ">"
    if pos + 2 + tag_name.len() >= len {
        return None;
    }
    if bytes.get(pos..pos + 2) != Some(b"</") {
        return None;
    }
    let name_start = pos + 2;
    if !bytes
        .get(name_start..name_start + tag_name.len())
        .is_some_and(|name| name.eq_ignore_ascii_case(tag_name))
    {
        return None;
    }
    let mut check_pos = name_start + tag_name.len();
    while check_pos < len
        && let Some(&byte) = bytes.get(check_pos)
    {
        match byte {
            b'>' => return Some(check_pos + 1),
            b' ' | b'\t' | b'\n' | b'\r' => check_pos += 1,
            _ => return None,
        }
    }
    None
}
