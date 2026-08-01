//! Template-block indentation without paying for raw-region lexing when the
//! formatted template cannot contain a raw continuation line.

use super::raw_mask::{compute_raw_line_mask, starts_v_pre_attribute_at};
use memchr::{memchr, memchr2, memchr3};

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
    if memchr(b'`', source).is_some() {
        return true;
    }

    let mut cursor = 0;
    let mut in_tag = false;
    while cursor < source.len() {
        if !in_tag {
            let Some(offset) = memchr(b'<', &source[cursor..]) else {
                break;
            };
            cursor += offset;
            let tail = &source[cursor..];
            if tail.starts_with(b"<!--") || starts_raw_tag(tail) {
                return true;
            }
            in_tag = source
                .get(cursor + 1)
                .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'/');
            cursor += 1;
            continue;
        }

        let structural = memchr3(b'\'', b'"', b'>', &source[cursor..]);
        let maybe_v_pre = memchr2(b'v', b'V', &source[cursor..]);
        let Some(offset) = earliest(structural, maybe_v_pre) else {
            break;
        };
        cursor += offset;

        if matches!(source[cursor], b'v' | b'V') {
            if starts_v_pre_attribute_at(source, cursor) {
                return true;
            }
            cursor += 1;
            continue;
        }
        if source[cursor] == b'>' {
            in_tag = false;
            cursor += 1;
            continue;
        }

        let quote = source[cursor];
        let tail = &source[cursor + 1..];
        let close = memchr(quote, tail);
        let newline = memchr(b'\n', tail);
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
    for name in [b"pre".as_slice(), b"textarea".as_slice()] {
        let Some(head) = tail.get(1..1 + name.len()) else {
            continue;
        };
        if head.eq_ignore_ascii_case(name)
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
    output: &mut Vec<u8>,
    source: &str,
    indent: &[u8],
    newline: &[u8],
) {
    if !needs_raw_line_mask(source.as_bytes()) {
        for line in source.as_bytes().split(|byte| *byte == b'\n') {
            write_line(output, line, indent, newline, false);
        }
        return;
    }

    let lines: Vec<_> = source.as_bytes().split(|byte| *byte == b'\n').collect();
    let raw_mask = compute_raw_line_mask(&lines);
    for (line, raw) in lines.into_iter().zip(raw_mask) {
        write_line(output, line, indent, newline, raw);
    }
}

fn write_line(output: &mut Vec<u8>, line: &[u8], indent: &[u8], newline: &[u8], raw: bool) {
    if !line.is_empty() && line != b"\r" && !raw {
        output.extend_from_slice(indent);
    }
    output.extend_from_slice(line);
    output.extend_from_slice(newline);
}

#[cfg(test)]
mod tests {
    use super::{compute_raw_line_mask, needs_raw_line_mask, write_indented_template, write_line};

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
            "<TEXTAREA>\nraw\n</TEXTAREA>",
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
}
