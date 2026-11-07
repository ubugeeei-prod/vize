//! Retained SFC block boundary implementations owned by L1.
//!
//! Compatibility consumers keep their existing descriptor policy and public
//! parse API while opening, attribute, script, template and raw-block scanning
//! share this implementation. Native descriptor admission remains separate.

use alloc::borrow::Cow;
use memchr::memrchr;
use vize_l0::{FxHashMap, String, cstr};

mod block;
mod raw;
mod split;
mod template_boundary;

pub use block::{parse_block_fast, tag_name_eq};
pub use raw::find_block_end;
pub use split::{CompatBlock, CompatBlockLocation, CompatSplitError, split_blocks};

pub type BlockAttrs<'a> = FxHashMap<Cow<'a, str>, Cow<'a, str>>;
pub type BlockParseOutput<'a> = (
    &'a [u8],       // tag name as bytes
    BlockAttrs<'a>, // attrs with borrowed strings
    Cow<'a, str>,   // content as borrowed string
    usize,          // content start
    usize,          // content end
    usize,          // end position
    usize,          // content end line
    usize,          // content end column
);
pub type BlockParseError = (&'static str, String);
pub type BlockParseResult<'a> = Result<Option<BlockParseOutput<'a>>, BlockParseError>;

pub struct BlockEndSearch<'a> {
    pub bytes: &'a [u8],
    pub source: &'a str,
    pub tag_name: &'a [u8],
    pub pos: usize,
    pub content_start: usize,
    pub start_line: usize,
    pub start_column: usize,
    pub initial_last_newline: usize,
    pub attrs: BlockAttrs<'a>,
}

/// Build a uniform `(code, message)` error for any malformed block.
pub fn build_malformed_error(tag_name: &[u8], reason: &str) -> BlockParseError {
    let tag_str = core::str::from_utf8(tag_name).unwrap_or("unknown");
    (
        "MALFORMED_BLOCK",
        cstr!("Malformed <{tag_str}> block: {reason}."),
    )
}

/// Newline count and the offset of the last newline. A vectorized count beats
/// stepping `memchr_iter` through every line of a block.
#[inline]
pub fn newlines(bytes: &[u8]) -> (usize, Option<usize>) {
    let count = bytes.iter().filter(|&&b| b == b'\n').count();
    let last = if count == 0 {
        None
    } else {
        memrchr(b'\n', bytes)
    };
    (count, last)
}

#[inline]
pub fn advance_line(bytes: &[u8], base: usize, line: &mut usize, last_newline: &mut usize) {
    let (count, last) = newlines(bytes);
    *line += count;
    if let Some(offset) = last {
        *last_newline = base + offset;
    }
}

#[inline]
pub fn content_end_column(
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
pub fn find_closing_tag_end(
    bytes: &[u8],
    pos: usize,
    len: usize,
    tag_name: &[u8],
) -> Option<usize> {
    // Need at least "</" + tag_name + ">"
    if pos + 2 + tag_name.len() >= len {
        return None;
    }
    if bytes.get(pos..pos + 2) != Some(b"</".as_slice()) {
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
