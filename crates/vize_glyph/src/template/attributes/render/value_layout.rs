//! Quote layout when a single-line directive value overflows its attribute line.

use super::{AttributeLayout, ParsedAttribute, write_indent};
use crate::template::directives::should_format_expression;
use crate::template::literal_lines::LiteralLineState;
use unicode_width::UnicodeWidthStr;

pub(super) fn write_overflowing_value(
    output: &mut Vec<u8>,
    attr: &ParsedAttribute,
    rendered: &str,
    newline: &[u8],
    indent: &[u8],
    depth: usize,
    layout: &AttributeLayout<'_>,
) -> bool {
    if !should_format_expression(&attr.name)
        || attr.name == "v-for"
        || (rendered.contains(['\r', '\n']) && !attr.owns_value_lines)
    {
        return false;
    }
    let Some(value) = rendered
        .get(attr.name.len()..)
        .and_then(|tail| tail.strip_prefix("=\""))
        .and_then(|tail| tail.strip_suffix('"'))
        .filter(|value| !value.trim().is_empty() && !value.trim_start().starts_with("//"))
    else {
        return false;
    };
    let Some(line) = output
        .rsplit(|byte| matches!(byte, b'\r' | b'\n'))
        .next()
        .and_then(|line| core::str::from_utf8(line).ok())
    else {
        return false;
    };
    let indent_width = layout.options.tab_width as usize;
    let width = layout.options.print_width as usize;
    let column = line.split('\t').map(str::width).sum::<usize>()
        + line.bytes().filter(|byte| *byte == b'\t').count() * indent_width
        + layout.base_depth * indent_width;
    if !attr.owns_value_lines && column + rendered.width() <= width {
        return false;
    }

    output.extend_from_slice(attr.name.as_bytes());
    output.extend_from_slice(b"=\"");
    output.extend_from_slice(newline);
    // The existing SFC mask owns all lines after a quote with no same-line
    // value. Emit their final indentation here; its outer writer keeps them raw.
    let mut literal = LiteralLineState::rendered();
    for line in value.split('\n') {
        let line = line.trim_end_matches('\r');
        if !literal.line_is_raw() {
            write_indent(output, indent, depth + layout.base_depth + 1);
        }
        output.extend_from_slice(line.as_bytes());
        output.extend_from_slice(newline);
        literal.advance_line(line);
    }
    write_indent(output, indent, depth + layout.base_depth);
    output.push(b'"');
    true
}
