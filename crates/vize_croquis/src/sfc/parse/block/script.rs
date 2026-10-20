//! `<script>` boundary scanning that hides closing tags inside JavaScript
//! comments, strings and regex literals, e.g. ``const x = `</script>` ``.

use std::borrow::Cow;

use memchr::{memchr2, memchr3, memmem};

use super::compat::can_start_string_literal;
use super::literals::{can_start_regex_literal, script_regex_end, skip_script_string_literal};
use super::{
    BlockEndSearch, BlockParseResult, advance_line, build_malformed_error, content_end_column,
    find_closing_tag_end,
};

/// Find the end of a `<script>` block.
pub(super) fn find_script_block_end<'a>(search: BlockEndSearch<'a>) -> BlockParseResult<'a> {
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

    // Track the previous non-whitespace character to determine string context
    let mut prev_significant_char: u8 = b'\n'; // Start as if at beginning of line
    // Next quote and next `/` or `<` at or after `pos`, refreshed once passed.
    let find_from = |at: usize, found: Option<usize>| found.map_or(len, |offset| at + offset);
    let mut next_quote = find_from(
        pos,
        memchr3(b'\'', b'"', b'`', bytes.get(pos..).unwrap_or_default()),
    );
    let mut next_slash_or_lt = find_from(
        pos,
        memchr2(b'/', b'<', bytes.get(pos..).unwrap_or_default()),
    );

    while let Some(&b) = bytes.get(pos) {
        // Every other byte, newlines included, only counts lines and moves
        // `prev_significant_char` to the last non-whitespace byte of its run.
        if !matches!(b, b'/' | b'<' | b'\'' | b'"' | b'`') {
            let rest = bytes.get(pos..).unwrap_or_default();
            if next_quote < pos {
                next_quote = find_from(pos, memchr3(b'\'', b'"', b'`', rest));
            }
            if next_slash_or_lt < pos {
                next_slash_or_lt = find_from(pos, memchr2(b'/', b'<', rest));
            }
            let run = bytes
                .get(pos..next_quote.min(next_slash_or_lt))
                .unwrap_or_default();
            if let Some(&last) = run.iter().rev().find(|&&c| !matches!(c, b' ' | b'\t')) {
                prev_significant_char = if last == b'\r' { b'\n' } else { last };
            }
            advance_line(run, pos, &mut line, &mut last_newline);
            pos += run.len();
            continue;
        }

        // Check for single-line comment
        if b == b'/' && pos + 1 < len && bytes.get(pos + 1) == Some(&b'/') {
            // Skip to end of line
            pos += 2;
            if let Some(newline_offset) =
                memchr2(b'\n', b'\r', bytes.get(pos..).unwrap_or_default())
            {
                pos += newline_offset;
            } else {
                pos = len;
            }
            continue;
        }

        // Check for multi-line comment
        if b == b'/' && pos + 1 < len && bytes.get(pos + 1) == Some(&b'*') {
            pos += 2;
            if let Some(end_offset) = memmem::find(bytes.get(pos..).unwrap_or_default(), b"*/") {
                advance_line(
                    bytes.get(pos..pos + end_offset).unwrap_or_default(),
                    pos,
                    &mut line,
                    &mut last_newline,
                );
                pos += end_offset + 2;
            } else {
                advance_line(
                    bytes.get(pos..).unwrap_or_default(),
                    pos,
                    &mut line,
                    &mut last_newline,
                );
                pos = len;
            }
            continue;
        }

        if b == b'/'
            && can_start_regex_literal(prev_significant_char)
            && let Some(next_pos) =
                script_regex_end(bytes, pos, len, &mut line, &mut last_newline, &attrs)
        {
            prev_significant_char = b'/';
            pos = next_pos;
            continue;
        }

        // Check for string literals (', ", `)
        // Only treat as string if in a context where strings are expected
        // (after =, (, [, ,, :, {, or at start of expression)
        // This avoids treating quotes inside regex literals as strings
        //
        // For backticks specifically, also allow after alphanumeric characters
        // to handle tagged templates (e.g., html`...`) and keywords (e.g., return `...`)
        if (b == b'\'' || b == b'"' || b == b'`')
            && can_start_string_literal(prev_significant_char, b)
        {
            pos = skip_script_string_literal(bytes, pos, len, b, &mut line, &mut last_newline);
            prev_significant_char = b; // String ended with quote
            continue;
        }

        // Check for closing tag (allows optional whitespace before '>')
        if b == b'<'
            && let Some(end_tag_pos) = find_closing_tag_end(bytes, pos, len, tag_name)
        {
            let content_end = pos;
            let col = content_end_column(
                content_start,
                start_line,
                start_column,
                content_end,
                line,
                last_newline,
            );
            let content = Cow::Borrowed(source.get(content_start..content_end).unwrap_or_default());
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

        prev_significant_char = b;
        pos += 1;
    }

    Err(build_malformed_error(
        tag_name,
        "the closing tag is missing",
    ))
}
