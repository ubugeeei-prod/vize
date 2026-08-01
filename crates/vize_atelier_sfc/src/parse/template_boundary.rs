//! Allocation-free structural boundary scanning for root `<template>` blocks.

mod fast_path;
mod interpolation;

#[cfg(test)]
mod tests;

use self::{fast_path::find_flat_template_end, interpolation::skip_template_interpolation};
use super::block::{
    BlockEndSearch, BlockParseResult, TAG_TEMPLATE, advance_line, build_malformed_error,
    find_closing_tag_end, is_whitespace_fast, starts_with_bytes,
};
use memchr::{memchr, memchr2, memchr3, memmem};
use std::borrow::Cow;

/// Failed JS-aware interpolation scans tolerated per template block before the
/// boundary scanner bounds every later scan to
/// `BOUNDED_INTERPOLATION_SCAN_WINDOW`. A failed scan means the interpolation
/// never closes anywhere in the rest of the file, so real documents see at most
/// a few; the cap bounds adversarial inputs at `MAX_FAILED_INTERPOLATION_SCANS`
/// tail walks (#3275).
const MAX_FAILED_INTERPOLATION_SCANS: usize = 8;

/// Bytes a JS-aware interpolation scan may walk once the failed-scan budget is
/// spent. Interpolations that close inside the window keep full opacity, so
/// strings, comments, and regexes still hide markup from the structural
/// scanner; only bodies that fail to close within the window degrade to the
/// structural handling an unclosed interpolation already gets (#3275).
const BOUNDED_INTERPOLATION_SCAN_WINDOW: usize = 4096;

#[inline]
fn is_opening_tag_named(bytes: &[u8], pos: usize, len: usize, expected_name: &[u8]) -> bool {
    let name_start = pos + 1;
    let name_end = name_start + expected_name.len();
    name_end < len
        && starts_with_bytes(&bytes[name_start..], expected_name)
        && (is_whitespace_fast(bytes[name_end])
            || bytes[name_end] == b'/'
            || bytes[name_end] == b'>')
}

#[inline]
fn raw_text_tag_name(bytes: &[u8], pos: usize, len: usize) -> Option<&'static [u8]> {
    match bytes[pos + 1].to_ascii_lowercase() {
        b's' if is_opening_tag_named(bytes, pos, len, b"script") => Some(b"script"),
        b's' if is_opening_tag_named(bytes, pos, len, b"style") => Some(b"style"),
        b't' if is_opening_tag_named(bytes, pos, len, b"textarea") => Some(b"textarea"),
        b't' if is_opening_tag_named(bytes, pos, len, b"title") => Some(b"title"),
        _ => None,
    }
}

fn find_raw_text_element_end(
    bytes: &[u8],
    mut pos: usize,
    len: usize,
    tag_name: &[u8],
) -> Option<usize> {
    while pos < len {
        let lt_offset = memchr(b'<', &bytes[pos..])?;
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
fn find_opening_tag_end(bytes: &[u8], pos: usize, len: usize) -> Option<(usize, bool)> {
    debug_assert_eq!(bytes[pos], b'<');
    let mut cursor = pos + 2;

    while cursor < len {
        let candidate = memchr3(b'>', b'"', b'\'', &bytes[cursor..])?;
        cursor += candidate;

        match bytes[cursor] {
            b'>' => {
                let mut before_end = cursor;
                while before_end > pos + 1 && is_whitespace_fast(bytes[before_end - 1]) {
                    before_end -= 1;
                }
                let self_closing = before_end > pos + 1 && bytes[before_end - 1] == b'/';
                return Some((cursor + 1, self_closing));
            }
            quote @ (b'"' | b'\'') => {
                cursor += 1;
                let closing_quote = memchr(quote, &bytes[cursor..])?;
                cursor += closing_quote + 1;
            }
            _ => unreachable!("memchr3 returned an unexpected byte"),
        }
    }

    None
}

/// Find the structural end of a root `<template>` block.
///
/// The slow path jumps between `<`/`{` candidates and skips actual HTML tags as
/// a unit, so template-shaped text in attributes, comments, raw-text elements,
/// and Vue interpolations cannot mutate nesting depth.
pub(super) fn find_template_block_end<'a>(search: BlockEndSearch<'a>) -> BlockParseResult<'a> {
    let BlockEndSearch {
        bytes,
        source,
        tag_name,
        mut pos,
        content_start,
        start_line,
        start_column,
        initial_last_newline,
        attrs,
    } = search;
    let len = bytes.len();
    let mut line = start_line;
    let mut last_newline = initial_last_newline;
    let mut depth = 1usize;
    // Set once no literal `}}` remains ahead. Both interpolation exits need
    // one, and later `{{` scan strict subranges of the proven-empty range, so
    // no interpolation in the rest of the block can close. This keeps inputs
    // dense in unclosed `{{` linear (#3275: a 45KB fuzz input with 1162 `{{`
    // against 44 `}}` re-walked the tail through the string/regex machinery
    // per occurrence).
    let mut interpolation_close_exhausted = false;
    let mut failed_interpolation_scans = 0usize;

    if let Some((content_end, end_pos)) = find_flat_template_end(bytes, content_start, len) {
        advance_line(
            &bytes[content_start..content_end],
            content_start,
            &mut line,
            &mut last_newline,
        );
        let col = if line == start_line {
            start_column + content_end - content_start
        } else {
            content_end - last_newline
        };
        let content = Cow::Borrowed(&source[content_start..content_end]);
        return Ok(Some((
            tag_name,
            attrs,
            content,
            content_start,
            content_end,
            end_pos,
            line,
            col,
        )));
    }

    while pos < len {
        let Some(candidate_offset) = memchr2(b'<', b'{', &bytes[pos..]) else {
            advance_line(&bytes[pos..], pos, &mut line, &mut last_newline);
            break;
        };

        advance_line(
            &bytes[pos..pos + candidate_offset],
            pos,
            &mut line,
            &mut last_newline,
        );
        pos += candidate_offset;

        if bytes[pos] == b'{' {
            if pos + 1 < len && bytes[pos + 1] == b'{' {
                let close_ahead = if interpolation_close_exhausted {
                    None
                } else {
                    let close_ahead = memmem::find(&bytes[pos + 2..], b"}}");
                    interpolation_close_exhausted = close_ahead.is_none();
                    close_ahead
                };

                // A failed scan already walked to the end of the source, so
                // every one costs the rest of the input. Real documents hold at
                // most a handful — each is an interpolation that never closes
                // again anywhere in the file — while the #3275 fuzz shape packs
                // hundreds whose scans stay expensive because a brace-consumed
                // or string-hidden `}}` survives near the tail. Once the budget
                // is spent, keep scanning JS-aware but only within a window, so
                // a later well-formed interpolation stays opaque instead of
                // exposing a quoted `</template>` to the structural scanner.
                let scan_limit = if failed_interpolation_scans < MAX_FAILED_INTERPOLATION_SCANS {
                    len
                } else {
                    (pos + 2)
                        .saturating_add(BOUNDED_INTERPOLATION_SCAN_WINDOW)
                        .min(len)
                };

                // Both interpolation exits need a literal `}}` within reach, so
                // a nearest delimiter beyond the limit rules the scan out before
                // it walks a single byte.
                let close_within_limit =
                    close_ahead.is_some_and(|offset| pos + 2 + offset + 2 <= scan_limit);

                if !close_within_limit {
                    // An unclosed interpolation is a template-parser error, not
                    // an SFC block-boundary error. Resume structural scanning so
                    // the root closing tag remains visible to that later stage.
                    pos += 2;
                } else if let Some(interpolation_end) = skip_template_interpolation(
                    &bytes[..scan_limit],
                    pos,
                    scan_limit,
                    &mut line,
                    &mut last_newline,
                ) {
                    pos = interpolation_end;
                } else {
                    failed_interpolation_scans += 1;
                    pos += 2;
                }
                continue;
            }
            pos += 1;
            continue;
        }

        if bytes[pos..].starts_with(b"<!--") {
            let comment_body_start = pos + 4;
            if let Some(comment_end_offset) = memmem::find(&bytes[comment_body_start..], b"-->") {
                let comment_end = comment_body_start + comment_end_offset + 3;
                advance_line(&bytes[pos..comment_end], pos, &mut line, &mut last_newline);
                pos = comment_end;
                continue;
            }
            break;
        }

        if bytes[pos..].starts_with(b"<![CDATA[") {
            let cdata_body_start = pos + 9;
            if let Some(cdata_end_offset) = memmem::find(&bytes[cdata_body_start..], b"]]>") {
                let cdata_end = cdata_body_start + cdata_end_offset + 3;
                advance_line(&bytes[pos..cdata_end], pos, &mut line, &mut last_newline);
                pos = cdata_end;
                continue;
            }
            break;
        }

        if pos + 1 < len && matches!(bytes[pos + 1], b'!' | b'?') {
            if let Some((declaration_end, _)) = find_opening_tag_end(bytes, pos, len) {
                advance_line(
                    &bytes[pos..declaration_end],
                    pos,
                    &mut line,
                    &mut last_newline,
                );
                pos = declaration_end;
                continue;
            }
            break;
        }

        if let Some(end_tag_pos) = find_closing_tag_end(bytes, pos, len, TAG_TEMPLATE) {
            depth -= 1;
            if depth == 0 {
                let content_end = pos;
                let col = if line == start_line {
                    start_column + content_end - content_start
                } else {
                    content_end - last_newline
                };
                let content = Cow::Borrowed(&source[content_start..content_end]);
                return Ok(Some((
                    tag_name,
                    attrs,
                    content,
                    content_start,
                    content_end,
                    end_tag_pos,
                    line,
                    col,
                )));
            }
            advance_line(&bytes[pos..end_tag_pos], pos, &mut line, &mut last_newline);
            pos = end_tag_pos;
            continue;
        }

        if pos + 1 < len && bytes[pos + 1].is_ascii_alphabetic() {
            let nested_template = is_opening_tag_named(bytes, pos, len, TAG_TEMPLATE);
            let raw_text_tag = raw_text_tag_name(bytes, pos, len);
            if let Some((tag_end, self_closing)) = find_opening_tag_end(bytes, pos, len) {
                let scan_end = if !self_closing && let Some(raw_text_tag) = raw_text_tag {
                    let Some(raw_text_end) =
                        find_raw_text_element_end(bytes, tag_end, len, raw_text_tag)
                    else {
                        break;
                    };
                    raw_text_end
                } else {
                    tag_end
                };
                advance_line(&bytes[pos..scan_end], pos, &mut line, &mut last_newline);
                if nested_template && !self_closing {
                    depth += 1;
                }
                pos = scan_end;
                continue;
            }
        }

        pos += 1;
    }

    Err(build_malformed_error(
        tag_name,
        "the closing tag is missing",
    ))
}
