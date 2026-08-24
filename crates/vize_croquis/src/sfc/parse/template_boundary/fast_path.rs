//! Fast-path detection for flat root templates.

use super::super::block::{find_closing_tag_end, is_whitespace_fast};
use super::{super::block::TAG_TEMPLATE, interpolation::skip_template_interpolation};
use super::{is_opening_tag_named, raw_text_tag_name};
use memchr::{memchr, memmem};

fn last_quote_opens_value(prefix: &[u8], quote: u8) -> bool {
    let Some(quote_pos) = memchr::memrchr(quote, prefix) else {
        return false;
    };
    let mut before_quote = quote_pos;
    while before_quote > 0 && is_whitespace_fast(prefix[before_quote - 1]) {
        before_quote -= 1;
    }
    before_quote > 0 && prefix[before_quote - 1] == b'='
}

fn template_close_has_ambiguous_context(
    bytes: &[u8],
    content_start: usize,
    close_start: usize,
    len: usize,
) -> bool {
    let prefix = &bytes[content_start..close_start];
    if last_quote_opens_value(prefix, b'"') || last_quote_opens_value(prefix, b'\'') {
        return true;
    }

    if let Some(interpolation_offset) = memmem::rfind(prefix, b"{{") {
        let interpolation_start = content_start + interpolation_offset;
        let mut line = 0;
        let mut last_newline = 0;
        if skip_template_interpolation(
            bytes,
            interpolation_start,
            len,
            &mut line,
            &mut last_newline,
        )
        .is_none_or(|interpolation_end| interpolation_end > close_start)
        {
            return true;
        }
    }

    false
}

/// Flat templates dominate production SFCs. Detect the common case with the
/// same `<`-only scan as the original parser and reserve the lexical state
/// machine for nested or ambiguous template candidates.
pub(super) fn find_flat_template_end(
    bytes: &[u8],
    content_start: usize,
    len: usize,
) -> Option<(usize, usize)> {
    let mut pos = content_start;
    while pos < len {
        let lt_offset = memchr(b'<', &bytes[pos..])?;
        pos += lt_offset;

        if bytes[pos..].starts_with(b"<!--")
            || bytes[pos..].starts_with(b"<![CDATA[")
            || (pos + 1 < len && matches!(bytes[pos + 1], b'!' | b'?'))
            || (pos + 1 < len
                && bytes[pos + 1].is_ascii_alphabetic()
                && raw_text_tag_name(bytes, pos, len).is_some())
        {
            return None;
        }

        if let Some(end_tag_pos) = find_closing_tag_end(bytes, pos, len, TAG_TEMPLATE) {
            if template_close_has_ambiguous_context(bytes, content_start, pos, len) {
                return None;
            }
            return Some((pos, end_tag_pos));
        }

        if is_opening_tag_named(bytes, pos, len, TAG_TEMPLATE) {
            return None;
        }

        pos += 1;
    }
    None
}
