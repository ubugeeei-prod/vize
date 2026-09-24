mod compat;
mod end;
mod literals;
mod regex;

use memchr::{memchr, memchr_iter, memmem};
use std::borrow::Cow;
use vize_carton::{FxHashMap, String, cstr};

use compat::{can_start_string_literal, is_void_block};
use end::find_block_end;
pub(super) use literals::{can_start_regex_literal, skip_script_string_literal};
pub(super) use regex::skip_regex_literal;

// Tag name bytes for fast comparison
pub(super) const TAG_TEMPLATE: &[u8] = b"template";
const TAG_SCRIPT: &[u8] = b"script";
const TAG_STYLE: &[u8] = b"style";

pub(super) type BlockAttrs<'a> = FxHashMap<Cow<'a, str>, Cow<'a, str>>;
pub(super) type BlockParseOutput<'a> = (
    &'a [u8],       // tag name as bytes
    BlockAttrs<'a>, // attrs with borrowed strings
    Cow<'a, str>,   // content as borrowed string
    usize,          // content start
    usize,          // content end
    usize,          // end position
    usize,          // content end line
    usize,          // content end column
);
pub(super) type BlockParseError = (&'static str, String);
pub(super) type BlockParseResult<'a> = Result<Option<BlockParseOutput<'a>>, BlockParseError>;

pub(super) struct BlockEndSearch<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) source: &'a str,
    pub(super) tag_name: &'a [u8],
    pub(super) pos: usize,
    pub(super) content_start: usize,
    pub(super) start_line: usize,
    pub(super) start_column: usize,
    pub(super) initial_last_newline: usize,
    pub(super) attrs: BlockAttrs<'a>,
}

/// Build a uniform `(code, message)` error for any malformed block.
pub(super) fn build_malformed_error(tag_name: &[u8], reason: &str) -> BlockParseError {
    let tag_str = std::str::from_utf8(tag_name).unwrap_or("unknown");
    (
        "MALFORMED_BLOCK",
        cstr!("Malformed <{tag_str}> block: {reason}."),
    )
}

/// Fast tag name comparison using byte slices
#[inline(always)]
pub(super) fn tag_name_eq(name: &[u8], expected: &[u8]) -> bool {
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

/// Parse a single block from the source using byte operations
/// Returns borrowed strings using Cow for zero-copy
///
/// - `Ok(Some(...))` — successfully parsed block.
/// - `Ok(None)` — no SFC block starts at this position.
/// - `Err(...)` — a block starts here but is incomplete or malformed.
pub(super) fn parse_block_fast<'a>(
    bytes: &'a [u8],
    source: &'a str,
    start: usize,
    start_line: usize,
    start_column: usize,
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

    // Parse attributes with zero-copy
    let mut attrs: BlockAttrs<'a> = FxHashMap::default();

    while bytes.get(pos).is_some_and(|&b| b != b'>') {
        // Skip whitespace
        while bytes.get(pos).is_some_and(|&b| is_whitespace_fast(b)) {
            pos += 1;
        }

        if bytes.get(pos).is_none_or(|&b| b == b'>' || b == b'/') {
            break;
        }

        // Parse attribute name
        let attr_start = pos;
        while let Some(&c) = bytes.get(pos) {
            if c == b'='
                || c == b' '
                || c == b'>'
                || c == b'/'
                || c == b'\t'
                || c == b'\n'
                || c == b'\r'
            {
                break;
            }
            pos += 1;
        }

        if pos == attr_start {
            pos += 1;
            continue;
        }

        // Zero-copy: borrow from source
        let attr_name: Cow<'a, str> =
            Cow::Borrowed(source.get(attr_start..pos).unwrap_or_default());

        // Skip whitespace
        while matches!(bytes.get(pos), Some(b' ' | b'\t')) {
            pos += 1;
        }

        let attr_value: Cow<'a, str> = if bytes.get(pos) == Some(&b'=') {
            pos += 1;

            // Skip whitespace
            while matches!(bytes.get(pos), Some(b' ' | b'\t')) {
                pos += 1;
            }

            if let Some(&quote_char) = bytes.get(pos).filter(|&&b| b == b'"' || b == b'\'') {
                pos += 1;
                let value_start = pos;

                // Use memchr for fast quote finding
                if let Some(quote_pos) = memchr(quote_char, bytes.get(pos..).unwrap_or_default()) {
                    pos += quote_pos;
                    let value = Cow::Borrowed(source.get(value_start..pos).unwrap_or_default());
                    pos += 1; // Skip closing quote
                    value
                } else {
                    // No closing quote found
                    while bytes.get(pos).is_some_and(|&b| b != quote_char) {
                        pos += 1;
                    }
                    let value = Cow::Borrowed(source.get(value_start..pos).unwrap_or_default());
                    if pos < len {
                        pos += 1;
                    }
                    value
                }
            } else {
                // Unquoted value
                let value_start = pos;
                while let Some(&c) = bytes.get(pos) {
                    if c == b' ' || c == b'>' || c == b'/' || c == b'\t' || c == b'\n' {
                        break;
                    }
                    pos += 1;
                }
                Cow::Borrowed(source.get(value_start..pos).unwrap_or_default())
            }
        } else {
            // Boolean attribute
            Cow::Borrowed("")
        };

        if !attr_name.is_empty() {
            attrs.insert(attr_name, attr_value);
        }
    }

    // Handle self-closing tag.
    let is_self_closing = bytes.get(pos) == Some(&b'/');

    if is_self_closing {
        pos += 1;
        while bytes.get(pos).is_some_and(|&b| is_whitespace_fast(b)) {
            pos += 1;
        }
        if bytes.get(pos) != Some(&b'>') {
            return Err(build_malformed_error(
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
            attrs,
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
        return Err(build_malformed_error(
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
            attrs,
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
        return super::template_boundary::find_template_block_end(BlockEndSearch {
            bytes,
            source,
            tag_name,
            pos,
            content_start,
            start_line: content_start_line,
            start_column: content_start_column,
            initial_last_newline: content_start,
            attrs,
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
            attrs,
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
            attrs,
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
