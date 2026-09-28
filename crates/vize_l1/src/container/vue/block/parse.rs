use alloc::borrow::Cow;
use memchr::{memchr, memmem};

use super::compat::{can_start_string_literal, is_void_block};
use super::end::find_block_end;
use super::open;
use super::{
    AttrSink, BlockEndSearch, BlockParseResult, TAG_SCRIPT, TAG_STYLE, TAG_TEMPLATE, advance_line,
    build_malformed_error, build_unterminated_open_error, can_start_regex_literal,
    content_end_column, find_closing_tag_end, is_tag_name_char_fast, is_whitespace_fast,
    position_after, skip_regex_literal, skip_script_string_literal,
};

/// Parse a single block from source bytes with zero-copy borrowed `Cow` strings.
///
/// - `Ok(Some(...))` — successfully parsed block.
/// - `Ok(None)` — no SFC block starts at this position.
/// - `Err(...)` — a block starts here but is incomplete or malformed.
pub fn parse_block_fast<'a>(
    bytes: &'a [u8],
    source: &'a str,
    start: usize,
    start_line: usize,
    start_column: usize,
    attrs: &mut impl AttrSink<'a>,
) -> BlockParseResult<'a> {
    // This parser intentionally works on byte slices and returns borrowed `Cow`
    // values. SFC parsing sits on every compile/lint/check path, so avoiding
    // temporary strings for tag names, attrs, and block content has an outsized
    // effect on allocation profiles.
    let len = bytes.len();

    // Skip '<'
    let mut pos = start + 1;
    if pos >= len {
        return Ok(None);
    }

    // Parse tag name - find end of tag name
    let tag_start = pos;
    while bytes.get(pos).is_some_and(|&b| is_tag_name_char_fast(b)) {
        pos += 1;
    }

    if pos == tag_start {
        return Ok(None);
    }

    let tag_name = source.as_bytes().get(tag_start..pos).unwrap_or_default();

    open::parse_attrs(bytes, source, &mut pos, attrs);

    // Handle self-closing tag.
    let is_self_closing = bytes.get(pos) == Some(&b'/');

    if is_self_closing {
        pos += 1;
        while bytes.get(pos).is_some_and(|&b| is_whitespace_fast(b)) {
            pos += 1;
        }
        if bytes.get(pos) != Some(&b'>') {
            return Err(build_unterminated_open_error(
                tag_name,
                "the self-closing tag is incomplete",
            ));
        }
        pos += 1;
        let (content_line, content_column) = position_after(
            bytes.get(start..pos).unwrap_or_default(),
            start_line,
            start_column,
        );
        return Ok(Some((
            tag_name,
            Cow::Borrowed(""),
            pos,
            pos,
            pos,
            content_line,
            content_column,
        )));
    }

    // Skip '>'
    if bytes.get(pos) == Some(&b'>') {
        pos += 1;
    } else {
        return Err(build_unterminated_open_error(
            tag_name,
            "the opening tag is incomplete",
        ));
    }

    let content_start = pos;
    let (content_start_line, content_start_column) = position_after(
        bytes.get(start..content_start).unwrap_or_default(),
        start_line,
        start_column,
    );

    if is_void_block(tag_name) {
        return Ok(Some((
            tag_name,
            Cow::Borrowed(""),
            content_start,
            content_start,
            pos,
            content_start_line,
            content_start_column,
        )));
    }

    // Find closing tag based on tag type
    let mut line = content_start_line;
    let mut last_newline = content_start;

    // Handle known tags with static closing tags
    if tag_name.eq_ignore_ascii_case(TAG_TEMPLATE) {
        return super::super::template_boundary::find_template_block_end(BlockEndSearch {
            bytes,
            source,
            tag_name,
            pos,
            content_start,
            start_line: content_start_line,
            start_column: content_start_column,
            initial_last_newline: content_start,
        });
    }

    if tag_name.eq_ignore_ascii_case(TAG_STYLE) {
        return find_block_end(BlockEndSearch {
            bytes,
            source,
            tag_name,
            pos,
            content_start,
            start_line: content_start_line,
            start_column: content_start_column,
            initial_last_newline: content_start,
        });
    }

    // Custom block: need to find closing tag dynamically
    if !tag_name.eq_ignore_ascii_case(TAG_SCRIPT) {
        return find_block_end(BlockEndSearch {
            bytes,
            source,
            tag_name,
            pos,
            content_start,
            start_line: content_start_line,
            start_column: content_start_column,
            initial_last_newline: content_start,
        });
    }

    // For script blocks, we need to be aware of string literals to avoid
    // matching closing tags inside strings like: const x = `</script>`
    let is_script = tag_name.eq_ignore_ascii_case(TAG_SCRIPT);

    // Track the previous non-whitespace character to determine string context
    let mut prev_significant_char: u8 = b'\n'; // Start as if at beginning of line

    while let Some(&b) = bytes.get(pos) {
        if b == b'\n' {
            line += 1;
            last_newline = pos;
            prev_significant_char = b'\n';
            pos += 1;
            continue;
        }

        // Skip whitespace but don't update prev_significant_char
        if b == b' ' || b == b'\t' || b == b'\r' {
            pos += 1;
            continue;
        }

        // For script blocks, skip over comments and string literals
        if is_script {
            // Check for single-line comment
            if b == b'/' && pos + 1 < len && bytes.get(pos + 1) == Some(&b'/') {
                // Skip to end of line
                pos += 2;
                if let Some(newline_offset) = memchr(b'\n', bytes.get(pos..).unwrap_or_default()) {
                    pos += newline_offset;
                } else {
                    pos = len;
                }
                continue;
            }

            // Check for multi-line comment
            if b == b'/' && pos + 1 < len && bytes.get(pos + 1) == Some(&b'*') {
                pos += 2;
                if let Some(end_offset) = memmem::find(bytes.get(pos..).unwrap_or_default(), b"*/")
                {
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
                    skip_regex_literal(bytes, pos, len, &mut line, &mut last_newline)
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
        }

        // Check for closing tag (allows optional whitespace before '>')
        if b == b'<'
            && let Some(end_tag_pos) = find_closing_tag_end(bytes, pos, len, tag_name)
        {
            let content_end = pos;
            let col = content_end_column(
                content_start,
                content_start_line,
                content_start_column,
                content_end,
                line,
                last_newline,
            );
            let content = Cow::Borrowed(source.get(content_start..content_end).unwrap_or_default());
            return Ok(Some((
                tag_name,
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
