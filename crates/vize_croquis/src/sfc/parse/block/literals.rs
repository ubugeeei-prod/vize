//! Skipping JavaScript string, template and regex literals while scanning
//! `<script>` content for its closing tag.

use super::compat::can_start_string_literal;
use super::{BlockAttrs, advance_line, find_closing_tag_end, skip_regex_literal};
use memchr::{memchr, memmem};

pub(in crate::sfc::parse) fn can_start_regex_literal(prev_significant_char: u8) -> bool {
    matches!(
        prev_significant_char,
        b'=' | b'('
            | b'['
            | b','
            | b':'
            | b'{'
            | b';'
            | b'\n'
            | b'?'
            | b'&'
            | b'|'
            | b'+'
            | b'-'
            | b'*'
            | b'!'
            | b'>'
            | b'<'
            | b'%'
            | b'^'
    )
}

/// Do not let an explicit JSX closing element turn the real SFC closing tag
/// into the speculative regex terminator plus invalid `script` flags. Ordinary
/// scripts and genuine regex terminators keep the original scanner result.
pub(super) fn script_regex_end(
    bytes: &[u8],
    pos: usize,
    len: usize,
    line: &mut usize,
    last_newline: &mut usize,
    attrs: &BlockAttrs<'_>,
) -> Option<usize> {
    let before = (*line, *last_newline);
    let end = skip_regex_literal(bytes, pos, len, line, last_newline)?;
    if find_closing_tag_end(bytes, end.saturating_sub(b"</script".len()), len, b"script").is_some()
        && attrs
            .get("lang")
            .is_some_and(|lang| matches!(lang.as_ref(), "jsx" | "tsx"))
    {
        (*line, *last_newline) = before;
        return None;
    }
    Some(end)
}

pub(in crate::sfc::parse) fn skip_script_string_literal(
    bytes: &[u8],
    mut pos: usize,
    len: usize,
    quote: u8,
    line: &mut usize,
    last_newline: &mut usize,
) -> usize {
    debug_assert_eq!(bytes.get(pos), Some(&quote));
    pos += 1;

    while pos < len
        && let Some(&c) = bytes.get(pos)
    {
        if c == b'\n' {
            *line += 1;
            *last_newline = pos;
        }

        if c == b'\\' && pos + 1 < len {
            if bytes.get(pos + 1) == Some(&b'\n') {
                *line += 1;
                *last_newline = pos + 1;
            }
            pos += 2;
            continue;
        }

        if quote == b'`' && c == b'$' && pos + 1 < len && bytes.get(pos + 1) == Some(&b'{') {
            pos = skip_template_expression(bytes, pos + 2, len, line, last_newline);
            continue;
        }

        if c == quote {
            return pos + 1;
        }

        if quote != b'`' && c == b'\n' {
            return pos + 1;
        }

        pos += 1;
    }

    len
}

fn skip_template_expression(
    bytes: &[u8],
    mut pos: usize,
    len: usize,
    line: &mut usize,
    last_newline: &mut usize,
) -> usize {
    let mut brace_depth = 1;
    let mut prev_significant_char: u8 = b'{';

    while pos < len
        && brace_depth > 0
        && let Some(&b) = bytes.get(pos)
    {
        if b == b'\n' {
            *line += 1;
            *last_newline = pos;
            prev_significant_char = b'\n';
            pos += 1;
            continue;
        }

        if b == b' ' || b == b'\t' || b == b'\r' {
            pos += 1;
            continue;
        }

        if b == b'/' && pos + 1 < len && bytes.get(pos + 1) == Some(&b'/') {
            pos += 2;
            if let Some(newline_offset) = memchr(b'\n', bytes.get(pos..).unwrap_or_default()) {
                pos += newline_offset;
            } else {
                pos = len;
            }
            continue;
        }

        if b == b'/' && pos + 1 < len && bytes.get(pos + 1) == Some(&b'*') {
            pos += 2;
            if let Some(end_offset) = memmem::find(bytes.get(pos..).unwrap_or_default(), b"*/") {
                advance_line(
                    bytes.get(pos..pos + end_offset).unwrap_or_default(),
                    pos,
                    line,
                    last_newline,
                );
                pos += end_offset + 2;
            } else {
                advance_line(
                    bytes.get(pos..).unwrap_or_default(),
                    pos,
                    line,
                    last_newline,
                );
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

        if (b == b'\'' || b == b'"' || b == b'`')
            && can_start_string_literal(prev_significant_char, b)
        {
            pos = skip_script_string_literal(bytes, pos, len, b, line, last_newline);
            prev_significant_char = b;
            continue;
        }

        match b {
            b'{' => {
                brace_depth += 1;
                prev_significant_char = b;
                pos += 1;
            }
            b'}' => {
                brace_depth -= 1;
                prev_significant_char = b;
                pos += 1;
            }
            b'\\' => {
                pos = (pos + 2).min(len);
            }
            _ => {
                prev_significant_char = b;
                pos += 1;
            }
        }
    }

    pos
}

#[cfg(test)]
mod tests {
    use super::{BlockAttrs, script_regex_end, skip_regex_literal};
    use std::borrow::Cow;

    #[test]
    fn jsx_boundary_recovery_does_not_change_other_language_or_regex_results() {
        for language in [None, Some("js"), Some("ts"), Some("coffee")] {
            let mut attrs = BlockAttrs::default();
            if let Some(language) = language {
                attrs.insert(Cow::Borrowed("lang"), Cow::Borrowed(language));
            }
            let bytes = b"/p>;</script><template />";
            let (mut old_line, mut old_last) = (4, 17);
            let expected = skip_regex_literal(bytes, 0, bytes.len(), &mut old_line, &mut old_last);
            let (mut line, mut last) = (4, 17);
            assert_eq!(
                script_regex_end(bytes, 0, bytes.len(), &mut line, &mut last, &attrs),
                expected
            );
            assert_eq!((line, last), (old_line, old_last));
        }
        for language in ["jsx", "tsx"] {
            let mut attrs = BlockAttrs::default();
            attrs.insert(Cow::Borrowed("lang"), Cow::Borrowed(language));
            let bytes = b"/p>;</script><template />";
            let (mut line, mut last) = (4, 17);
            assert_eq!(
                script_regex_end(bytes, 0, bytes.len(), &mut line, &mut last, &attrs),
                None
            );
            assert_eq!((line, last), (4, 17));
            for bytes in [br"/p>[</script>]/g".as_slice(), br"/[</script>\/]/giu"] {
                let (mut old_line, mut old_last) = (4, 17);
                let expected =
                    skip_regex_literal(bytes, 0, bytes.len(), &mut old_line, &mut old_last);
                let (mut line, mut last) = (4, 17);
                assert_eq!(
                    script_regex_end(bytes, 0, bytes.len(), &mut line, &mut last, &attrs),
                    expected
                );
                assert_eq!((line, last), (old_line, old_last));
            }
        }
    }
}
