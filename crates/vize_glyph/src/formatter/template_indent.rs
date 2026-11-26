//! Template-block indentation without paying for raw-region lexing when the
//! formatted template cannot contain a raw continuation line.

mod raw_lines;

#[cfg(test)]
use super::raw_mask::compute_raw_line_mask;
use super::raw_mask::starts_v_pre_attribute_at;
use crate::template::WHITESPACE_SIGNIFICANT_NATIVE_ELEMENTS;
use memchr::{memchr, memchr2, memchr3};
use vize_l0::string_builder::TextBuf;

/// Whether the full raw-region lexer can mark any continuation line.
///
/// The common formatter corpus has only single-line tags and attribute values.
/// This preflight skips text and complete attribute values with SIMD searches;
/// it deliberately falls back for every whitespace-significant tag, comment,
/// `v-pre` region, backtick, or quoted value crossing a line boundary.
fn needs_raw_line_mask(source: &[u8]) -> bool {
    // A backtick is uncommon in templates and may open a multi-line literal in
    // either an interpolation or directive value. Keep that hostile path on
    // the real lexer instead of trying to duplicate its JS state machine here.
    let mut raw_cursor = 0;
    while let Some(offset) = memchr2(b'`', b'\\', source.get(raw_cursor..).unwrap_or_default()) {
        let at = raw_cursor + offset;
        if source.get(at) == Some(&b'`') || matches!(source.get(at + 1), Some(b'\r' | b'\n')) {
            return true;
        }
        raw_cursor = at + 1;
    }

    let mut cursor = 0;
    let mut in_tag = false;
    while cursor < source.len() {
        if !in_tag {
            let Some(offset) = memchr(b'<', source.get(cursor..).unwrap_or_default()) else {
                break;
            };
            cursor += offset;
            let tail = source.get(cursor..).unwrap_or_default();
            if tail.starts_with(b"<!--") || starts_raw_tag(tail) {
                return true;
            }
            in_tag = source
                .get(cursor + 1)
                .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'/');
            cursor += 1;
            continue;
        }

        let structural = memchr3(b'\'', b'"', b'>', source.get(cursor..).unwrap_or_default());
        let maybe_v_pre = memchr2(b'v', b'V', source.get(cursor..).unwrap_or_default());
        let Some(offset) = earliest(structural, maybe_v_pre) else {
            break;
        };
        cursor += offset;

        if matches!(source.get(cursor), Some(b'v' | b'V')) {
            if starts_v_pre_attribute_at(source, cursor) {
                return true;
            }
            cursor += 1;
            continue;
        }
        if source.get(cursor) == Some(&b'>') {
            in_tag = false;
            cursor += 1;
            continue;
        }

        let quote = source.get(cursor).copied().unwrap_or_default();
        let tail = source.get(cursor + 1..).unwrap_or_default();
        let close = memchr(quote, tail);
        let newline = memchr2(b'\r', b'\n', tail);
        if newline.is_some_and(|line| close.is_none_or(|end| line < end)) {
            return true;
        }
        cursor += close.map_or(tail.len() + 1, |end| end + 2);
    }

    false
}

fn earliest(left: Option<usize>, right: Option<usize>) -> Option<usize> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(offset), None) | (None, Some(offset)) => Some(offset),
        (None, None) => None,
    }
}

fn starts_raw_tag(tail: &[u8]) -> bool {
    for name in WHITESPACE_SIGNIFICANT_NATIVE_ELEMENTS.map(str::as_bytes) {
        let Some(head) = tail.get(1..1 + name.len()) else {
            continue;
        };
        if head == name
            && tail
                .get(1 + name.len())
                .is_none_or(|byte| matches!(byte, b'>' | b' ' | b'\t' | b'\r' | b'\n' | b'/'))
        {
            return true;
        }
    }
    false
}

pub(super) fn write_indented_template(
    output: &mut TextBuf,
    source: &str,
    indent: &str,
    newline: &str,
) {
    // LF uses a known one-byte copy; non-LF storage and normalization stay cold.
    if newline != "\n" {
        write_indented_non_lf_template(output, source, indent, newline);
        return;
    }

    if !needs_raw_line_mask(source.as_bytes()) {
        for line in source.split('\n') {
            write_line(output, line, indent, "\n", false);
        }
        return;
    }

    raw_lines::write(output, source, indent, "\n");
}

#[cold]
#[inline(never)]
fn write_indented_non_lf_template(output: &mut TextBuf, source: &str, indent: &str, newline: &str) {
    if needs_raw_line_mask(source.as_bytes()) {
        raw_lines::write(output, source, indent, newline);
        return;
    }
    if newline == "\r" {
        for line in source.split('\r') {
            let line = line.strip_prefix('\n').unwrap_or(line);
            write_line(output, line, indent, newline, false);
        }
        return;
    }
    for line in source.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        write_line(output, line, indent, newline, false);
    }
}

/// Rebase an indentation-sensitive, non-HTML template without interpreting it.
///
/// Only the exact whitespace prefix shared by every non-blank line belongs to
/// the surrounding SFC. Replacing that prefix with the configured one-level
/// indent is semantics-preserving for Pug-like languages; all relative
/// indentation, comments, pipe text, filters, and trailing bytes stay intact.
pub(super) fn write_rebased_opaque_template(
    output: &mut TextBuf,
    source: &str,
    indent: &str,
    newline: &str,
) {
    // LF, CRLF and a lone CR each end a line.
    let raw_lines: Vec<_> = source
        .split('\n')
        .flat_map(|line| line.strip_suffix('\r').unwrap_or(line).split('\r'))
        .collect();
    let Some(first) = raw_lines.iter().position(|line| !is_blank(line)) else {
        return;
    };
    let Some(last) = raw_lines.iter().rposition(|line| !is_blank(line)) else {
        return;
    };
    let lines = raw_lines.get(first..=last).unwrap_or_default();
    let common = common_whitespace_prefix(lines);

    for raw_line in lines {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if is_blank(line) {
            output.push_str(newline);
            continue;
        }
        output.push_str(indent);
        output.push_str(line.get(common..).unwrap_or_default());
        output.push_str(newline);
    }
}

fn common_whitespace_prefix(lines: &[&str]) -> usize {
    let mut non_blank = lines
        .iter()
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .filter(|line| !is_blank(line))
        .map(|line| line.as_bytes());
    let Some(first) = non_blank.next() else {
        return 0;
    };
    let mut common = leading_whitespace(first);
    for line in non_blank {
        let limit = common.min(leading_whitespace(line));
        common = first
            .get(..limit)
            .unwrap_or_default()
            .iter()
            .zip(line.get(..limit).unwrap_or_default())
            .take_while(|(left, right)| left == right)
            .count();
        if common == 0 {
            break;
        }
    }
    common
}

fn leading_whitespace(line: &[u8]) -> usize {
    line.iter()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count()
}

fn is_blank(line: &str) -> bool {
    line.bytes()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\r'))
}

fn write_line(output: &mut TextBuf, line: &str, indent: &str, newline: &str, raw: bool) {
    if !line.is_empty() && line != "\r" && !raw {
        output.push_str(indent);
    }
    output.push_str(line);
    output.push_str(newline);
}

#[cfg(test)]
mod tests {
    use super::compute_raw_line_mask;
    use super::needs_raw_line_mask;
    use vize_l0::string_builder::TextBuf;

    // The writers take `&mut TextBuf` and `&str`; these keep the retained
    // byte-level test bodies unchanged.
    fn text(bytes: &[u8]) -> &str {
        core::str::from_utf8(bytes).unwrap()
    }

    fn write_line(output: &mut Vec<u8>, line: &[u8], indent: &[u8], newline: &[u8], raw: bool) {
        let mut text_buf = TextBuf::new();
        super::write_line(&mut text_buf, text(line), text(indent), text(newline), raw);
        output.extend_from_slice(text_buf.as_bytes());
    }

    fn write_indented_template(output: &mut Vec<u8>, source: &str, indent: &[u8], newline: &[u8]) {
        let mut text_buf = TextBuf::new();
        super::write_indented_template(&mut text_buf, source, text(indent), text(newline));
        output.extend_from_slice(text_buf.as_bytes());
    }

    fn write_rebased_opaque_template(
        output: &mut Vec<u8>,
        source: &str,
        indent: &[u8],
        newline: &[u8],
    ) {
        let mut text_buf = TextBuf::new();
        super::write_rebased_opaque_template(&mut text_buf, source, text(indent), text(newline));
        output.extend_from_slice(text_buf.as_bytes());
    }

    fn full_lexer_output(source: &str) -> Vec<u8> {
        let lines: Vec<_> = source.as_bytes().split(|byte| *byte == b'\n').collect();
        let mask = compute_raw_line_mask(&lines);
        let mut output = Vec::new();
        for (line, raw) in lines.into_iter().zip(mask) {
            write_line(&mut output, line, b"  ", b"\n", raw);
        }
        output
    }

    fn selected_output(source: &str) -> Vec<u8> {
        let mut output = Vec::new();
        write_indented_template(&mut output, source, b"  ", b"\n");
        output
    }

    #[test]
    fn ordinary_templates_bypass_the_raw_line_mask() {
        let ordinary = [
            "<div>{{ message }}</div>",
            "<main>\n  <button :disabled=\"loading\" @click=\"save\">Save</button>\n</main>",
            "<ul>\n  <li v-for=\"item in items\" :key=\"item.id\">{{ item.name }}</li>\n</ul>",
            "<div :title=\"'quoted'\" data-note=\"v-prepare\">{{ oneLine }}</div>",
        ];

        for source in ordinary {
            assert!(
                !needs_raw_line_mask(source.as_bytes()),
                "ordinary template must stay on the allocation-free path: {source}"
            );
            assert_eq!(
                selected_output(source),
                full_lexer_output(source),
                "the fast path must stay byte-identical to the full lexer"
            );
        }
    }

    #[test]
    fn every_raw_continuation_shape_keeps_the_full_lexer() {
        let raw = [
            "<pre>\nraw\n</pre>",
            "<textarea>\nraw\n</TEXTAREA>",
            "<listing>\nraw\n</listing>",
            "<code V-PRE>\n  {{ raw }}\n</code>",
            "<!--\nraw\n-->",
            "<div title=\"first\nsecond\">x</div>",
            "<div>{{ `first\nsecond` }}</div>",
        ];

        for source in raw {
            assert!(
                needs_raw_line_mask(source.as_bytes()),
                "raw template must keep the full lexer: {source}"
            );
            assert_eq!(selected_output(source), full_lexer_output(source));
        }
    }

    #[test]
    fn opaque_templates_rebase_only_the_shared_prefix() {
        let mut output = Vec::new();
        write_rebased_opaque_template(
            &mut output,
            "\n\tmain\r\n\t  //- keep\r\n\t  :plain\r\n\t    a  b  \r\n\r\n",
            b"  ",
            b"\r\n",
        );
        assert_eq!(
            output,
            b"  main\r\n    //- keep\r\n    :plain\r\n      a  b  \r\n"
        );
    }

    #[test]
    fn empty_opaque_templates_emit_no_phantom_body_line() {
        let mut output = Vec::new();
        write_rebased_opaque_template(&mut output, "\n \t\r\n", b"  ", b"\n");
        assert!(output.is_empty());
    }
}
