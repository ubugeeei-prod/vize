//! JavaScript-aware Vue interpolation skipping for template boundary scanning.

use super::super::block::{
    advance_line, can_start_regex_literal, skip_regex_literal, skip_script_string_literal,
};
use memchr::{memchr, memchr2, memchr3, memmem};

/// Skip a Vue interpolation while ignoring delimiter-like text inside JavaScript
/// strings, template literals, comments, and regular expressions.
///
/// `bytes` may be a prefix of the source that ends at `len`, which the caller
/// uses to bound the scan; positions stay absolute and everything past `len` is
/// treated as unreachable, so a body that does not close within the prefix
/// reports failure instead of walking the rest of the file (#3275).
pub(super) fn skip_template_interpolation(
    bytes: &[u8],
    mut pos: usize,
    len: usize,
    line: &mut usize,
    last_newline: &mut usize,
) -> Option<usize> {
    debug_assert!(bytes[pos..].starts_with(b"{{"));
    let original_line = *line;
    let original_last_newline = *last_newline;
    let body_start = pos + 2;

    // Every way out of this scan needs a literal `}}` ahead — the delimiter
    // arm requires adjacent `}` bytes at depth zero and the malformed-string
    // recovery searches for one — so its absence proves the interpolation
    // cannot close. Returning before the JS recovery loop keeps a run of
    // unclosed `{{` linear instead of re-walking the rest of the source per
    // occurrence through the string/regex machinery (#3275).
    let close_offset = memmem::find(&bytes[body_start..], b"}}")?;

    // Most interpolations are identifiers or simple expressions. Accept the
    // first delimiter immediately when no token before it can hide `}}`; this
    // keeps the common path on SIMD searches instead of the JS recovery loop.
    {
        let close_start = body_start + close_offset;
        let body = &bytes[body_start..close_start];
        if memchr3(b'\'', b'"', b'`', body).is_none() && memchr2(b'/', b'{', body).is_none() {
            let interpolation_end = close_start + 2;
            advance_line(&bytes[pos..interpolation_end], pos, line, last_newline);
            return Some(interpolation_end);
        }
    }

    let mut brace_depth = 0usize;
    let mut prev_significant_char = b'{';
    pos = body_start;

    while pos < len {
        let b = bytes[pos];

        if b == b'\n' {
            *line += 1;
            *last_newline = pos;
            prev_significant_char = b'\n';
            pos += 1;
            continue;
        }

        if matches!(b, b' ' | b'\t' | b'\r') {
            pos += 1;
            continue;
        }

        if b == b'/' && pos + 1 < len && bytes[pos + 1] == b'/' {
            pos += 2;
            if let Some(newline_offset) = memchr(b'\n', &bytes[pos..]) {
                pos += newline_offset;
            } else {
                pos = len;
            }
            continue;
        }

        if b == b'/' && pos + 1 < len && bytes[pos + 1] == b'*' {
            pos += 2;
            if let Some(end_offset) = memmem::find(&bytes[pos..], b"*/") {
                advance_line(&bytes[pos..pos + end_offset], pos, line, last_newline);
                pos += end_offset + 2;
            } else {
                pos = len;
            }
            continue;
        }

        if b == b'/'
            && can_start_regex_literal(prev_significant_char)
            && let Some(next_pos) = skip_regex_literal(bytes, pos, len, line, last_newline)
        {
            prev_significant_char = b'/';
            pos = next_pos;
            continue;
        }

        if matches!(b, b'\'' | b'"' | b'`') {
            let string_start = pos;
            let line_before_string = *line;
            let last_newline_before_string = *last_newline;
            let string_end = skip_script_string_literal(bytes, pos, len, b, line, last_newline);
            let string_closed = string_end > string_start && bytes[string_end - 1] == b;

            // Recover a malformed JS string at the interpolation delimiter. A
            // valid string may contain `}}`, so only use this path when the
            // quote scanner did not find its real closing delimiter.
            if !string_closed
                && let Some(close_offset) =
                    memmem::find(&bytes[string_start + 1..string_end], b"}}")
            {
                let interpolation_end = string_start + 1 + close_offset + 2;
                *line = line_before_string;
                *last_newline = last_newline_before_string;
                advance_line(
                    &bytes[string_start..interpolation_end],
                    string_start,
                    line,
                    last_newline,
                );
                return Some(interpolation_end);
            }

            pos = string_end;
            prev_significant_char = b;
            continue;
        }

        match b {
            b'{' => {
                brace_depth += 1;
                prev_significant_char = b;
                pos += 1;
            }
            b'}' if brace_depth == 0 && pos + 1 < len && bytes[pos + 1] == b'}' => {
                return Some(pos + 2);
            }
            b'}' => {
                brace_depth = brace_depth.saturating_sub(1);
                prev_significant_char = b;
                pos += 1;
            }
            b'\\' => {
                if pos + 1 < len && bytes[pos + 1] == b'\n' {
                    *line += 1;
                    *last_newline = pos + 1;
                }
                pos = (pos + 2).min(len);
            }
            _ => {
                prev_significant_char = b;
                pos += 1;
            }
        }
    }

    // Leave line tracking untouched when the interpolation is malformed so the
    // caller can recover through the outer malformed-block diagnostic.
    *line = original_line;
    *last_newline = original_last_newline;
    None
}
