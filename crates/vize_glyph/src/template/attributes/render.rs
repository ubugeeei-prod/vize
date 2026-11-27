//! Attribute emission and literal continuation ownership.

use super::{ParsedAttribute, rendered_attribute_is_multiline};
use crate::template::literal_spans::ValueLines;
use vize_l0::String;
use vize_l0::string_builder::TextBuf;

mod value_layout;

pub(crate) struct AttributeLayout<'a> {
    pub(crate) options: &'a crate::options::FormatOptions,
    pub(crate) base_depth: usize,
    pub(crate) max_per_line: usize,
}

pub(crate) fn write_rendered_attributes(
    output: &mut TextBuf,
    attrs: &[ParsedAttribute],
    rendered: &[String],
    newline: &str,
    indent: &str,
    depth: usize,
    layout: &AttributeLayout<'_>,
) {
    debug_assert_eq!(attrs.len(), rendered.len());
    let mut line_count = 0;
    for (attr, rendered) in attrs.iter().zip(rendered) {
        let attr_is_multiline = rendered_attribute_is_multiline(rendered, layout.options);
        if line_count == 0 || attr_is_multiline {
            output.push_str(newline);
            write_indent(output, indent, depth);
        } else {
            output.push(' ');
        }
        let wrapped = value_layout::write_overflowing_value(
            output, attr, rendered, newline, indent, depth, layout,
        );
        if !wrapped {
            write_rendered_attribute(
                output,
                rendered,
                newline,
                indent,
                depth,
                attr.indent_multiline_value,
            );
        }
        if wrapped || attr_is_multiline || line_count + 1 >= layout.max_per_line {
            line_count = 0;
        } else {
            line_count += 1;
        }
    }
}

pub(super) fn write_rendered_attribute(
    output: &mut TextBuf,
    attr: &str,
    newline: &str,
    indent: &str,
    continuation_depth: usize,
    indent_continuation: bool,
) {
    // Keep the ordinary LF/CRLF path on the original single-delimiter
    // iterator. Lone-CR output is rare and normalized once before emission.
    let normalized;
    let attr = if newline == "\r" && attr.contains('\r') {
        normalized = attr.replace('\r', "\n");
        normalized.as_str()
    } else {
        attr
    };
    // A one-line attribute has no continuation line whose literal ownership
    // could matter; most attributes take this path.
    if memchr::memchr(b'\n', attr.as_bytes()).is_none() {
        output.push_str(attr.trim_end_matches('\r'));
        return;
    }
    let mut lines = attr.split('\n');
    let mut literal = ValueLines::for_attribute(attr);
    if let Some(first) = lines.next() {
        output.push_str(first.trim_end_matches('\r'));
    }

    for line in lines {
        output.push_str(newline);
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
            write_line_indent(output, line, indent, continuation_depth);
        }
        output.push_str(line);
        literal.advance_line(line);
    }
}

/// Indent a formatter-owned continuation line of an attribute value. A blank
/// line holds nothing to indent: it is written empty, as the SFC block writers
/// write every other blank line, so it never carries trailing whitespace.
fn write_line_indent(output: &mut TextBuf, line: &str, indent: &str, depth: usize) {
    if !line.is_empty() {
        write_indent(output, indent, depth);
    }
}

fn write_indent(output: &mut TextBuf, indent: &str, depth: usize) {
    for _ in 0..depth {
        output.push_str(indent);
    }
}
