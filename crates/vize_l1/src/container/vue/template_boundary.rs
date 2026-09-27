//! Allocation-free structural boundary scanning for root `<template>` blocks.

mod fast_path;
mod interpolation;
mod tags;

#[cfg(test)]
mod tests;

use self::tags::{
    find_opening_tag_end, find_raw_text_element_end, is_opening_tag_named, raw_text_tag_name,
};
use self::{fast_path::find_flat_template_end, interpolation::skip_template_interpolation};
use super::block::{
    BlockEndSearch, BlockParseResult, TAG_TEMPLATE, advance_line, build_malformed_error,
    find_closing_tag_end,
};
use memchr::{memchr2, memmem};
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
            bytes.get(content_start..content_end).unwrap_or_default(),
            content_start,
            &mut line,
            &mut last_newline,
        );
        let col = if line == start_line {
            start_column + content_end - content_start
        } else {
            content_end - last_newline
        };
        let content = Cow::Borrowed(source.get(content_start..content_end).unwrap_or_default());
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
        let Some(candidate_offset) = memchr2(b'<', b'{', bytes.get(pos..).unwrap_or_default())
        else {
            advance_line(
                bytes.get(pos..).unwrap_or_default(),
                pos,
                &mut line,
                &mut last_newline,
            );
            break;
        };

        advance_line(
            bytes.get(pos..pos + candidate_offset).unwrap_or_default(),
            pos,
            &mut line,
            &mut last_newline,
        );
        pos += candidate_offset;

        if bytes.get(pos) == Some(&b'{') {
            if pos + 1 < len && bytes.get(pos + 1) == Some(&b'{') {
                let close_ahead = if interpolation_close_exhausted {
                    None
                } else {
                    let close_ahead = memmem::find(bytes.get(pos + 2..).unwrap_or_default(), b"}}");
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
                    bytes.get(..scan_limit).unwrap_or_default(),
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

        if bytes.get(pos..).unwrap_or_default().starts_with(b"<!--") {
            let comment_body_start = pos + 4;
            if let Some(comment_end_offset) =
                memmem::find(bytes.get(comment_body_start..).unwrap_or_default(), b"-->")
            {
                let comment_end = comment_body_start + comment_end_offset + 3;
                advance_line(
                    bytes.get(pos..comment_end).unwrap_or_default(),
                    pos,
                    &mut line,
                    &mut last_newline,
                );
                pos = comment_end;
                continue;
            }
            break;
        }

        if bytes
            .get(pos..)
            .unwrap_or_default()
            .starts_with(b"<![CDATA[")
        {
            let cdata_body_start = pos + 9;
            if let Some(cdata_end_offset) =
                memmem::find(bytes.get(cdata_body_start..).unwrap_or_default(), b"]]>")
            {
                let cdata_end = cdata_body_start + cdata_end_offset + 3;
                advance_line(
                    bytes.get(pos..cdata_end).unwrap_or_default(),
                    pos,
                    &mut line,
                    &mut last_newline,
                );
                pos = cdata_end;
                continue;
            }
            break;
        }

        if pos + 1 < len && matches!(bytes.get(pos + 1), Some(b'!' | b'?')) {
            if let Some((declaration_end, _)) = find_opening_tag_end(bytes, pos, len) {
                advance_line(
                    bytes.get(pos..declaration_end).unwrap_or_default(),
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
                let content =
                    Cow::Borrowed(source.get(content_start..content_end).unwrap_or_default());
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
            advance_line(
                bytes.get(pos..end_tag_pos).unwrap_or_default(),
                pos,
                &mut line,
                &mut last_newline,
            );
            pos = end_tag_pos;
            continue;
        }

        if pos + 1 < len && bytes.get(pos + 1).is_some_and(u8::is_ascii_alphabetic) {
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
                advance_line(
                    bytes.get(pos..scan_end).unwrap_or_default(),
                    pos,
                    &mut line,
                    &mut last_newline,
                );
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
