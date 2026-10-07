//! Attribute emission and literal continuation ownership.

use super::{ParsedAttribute, rendered_attribute_is_multiline};
use crate::template::literal_lines::LiteralLineState;
use vize_l0::String;

pub(crate) fn write_rendered_attributes(
    output: &mut Vec<u8>,
    attrs: &[ParsedAttribute],
    rendered: &[String],
    newline: &[u8],
    indent: &[u8],
    depth: usize,
    max_per_line: usize,
) {
    debug_assert_eq!(attrs.len(), rendered.len());
    let mut line_count = 0;
    for (attr, rendered) in attrs.iter().zip(rendered) {
        let attr_is_multiline = rendered_attribute_is_multiline(rendered);
        if line_count == 0 || attr_is_multiline {
            output.extend_from_slice(newline);
            write_indent(output, indent, depth);
        } else {
            output.push(b' ');
        }
        write_rendered_attribute(
            output,
            rendered,
            newline,
            indent,
            depth,
            attr.indent_multiline_value,
        );
        if attr_is_multiline || line_count + 1 >= max_per_line {
            line_count = 0;
        } else {
            line_count += 1;
        }
    }
}

pub(super) fn write_rendered_attribute(
    output: &mut Vec<u8>,
    attr: &str,
    newline: &[u8],
    indent: &[u8],
    continuation_depth: usize,
    indent_continuation: bool,
) {
    let mut lines = attr.split('\n');
    let mut literal = LiteralLineState::default();
    if let Some(first) = lines.next() {
        let first = first.trim_end_matches('\r');
        output.extend_from_slice(first.as_bytes());
        literal = LiteralLineState::from_attribute(first);
    }

    for line in lines {
        output.extend_from_slice(newline);
        let line = line.trim_end_matches('\r');
        // Quasi and legally continued quoted-string bytes are runtime value,
        // so a line that starts inside either kind of string is emitted
        // exactly as the expression formatter produced it — no attribute
        // indent in front of it, no leading whitespace stripped off it.
        //
        // Rewriting that whitespace was not only a rendered-output change: it
        // also moved the column at which every embedded `${…}` starts. The
        // expression formatter measures its line-break budget from that
        // column, so the next `vize fmt` pass — reading back the re-indented
        // literal — made a different wrap decision and produced a different
        // file. (#3379)
        if indent_continuation && !literal.line_is_raw() {
            write_indent(output, indent, continuation_depth);
        }
        output.extend_from_slice(line.as_bytes());
        literal.advance_line(line);
    }
}

fn write_indent(output: &mut Vec<u8>, indent: &[u8], depth: usize) {
    for _ in 0..depth {
        output.extend_from_slice(indent);
    }
}
