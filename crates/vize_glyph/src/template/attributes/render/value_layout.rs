//! Quote layout when a single-line directive value overflows its attribute line.

use super::{AttributeLayout, ParsedAttribute, write_indent, write_line_indent};
use crate::template::directives::should_format_expression;
use crate::template::literal_lines::Representation;
use crate::template::literal_spans::ValueLines;
use unicode_width::UnicodeWidthStr;
use vize_l0::string_builder::TextBuf;

pub(super) fn write_overflowing_value(
    output: &mut TextBuf,
    attr: &ParsedAttribute,
    rendered: &str,
    newline: &str,
    indent: &str,
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
    let normalized;
    let value = if newline == "\r" && value.contains('\r') {
        normalized = value.replace('\r', "\n");
        normalized.as_str()
    } else {
        value
    };
    let line = crate::template::formatter::current_line(output);
    let indent_width = layout.options.tab_width as usize;
    let width = layout.options.print_width as usize;
    let column = line.split('\t').map(str::width).sum::<usize>()
        + line.bytes().filter(|byte| *byte == b'\t').count() * indent_width
        + layout.base_depth * indent_width;
    if !attr.owns_value_lines && column + rendered.width() <= width {
        return false;
    }

    output.push_str(&attr.name);
    output.push_str("=\"");
    output.push_str(newline);
    // The existing SFC mask owns all lines after a quote with no same-line
    // value. Emit their final indentation here; its outer writer keeps them raw.
    let mut literal = ValueLines::new(value, Representation::Html);
    for line in value.split('\n') {
        let line = line.trim_end_matches('\r');
        if !literal.line_is_raw() {
            write_line_indent(output, line, indent, depth + layout.base_depth + 1);
        }
        output.push_str(line);
        output.push_str(newline);
        literal.advance_line(line);
    }
    write_indent(output, indent, depth + layout.base_depth);
    output.push('"');
    true
}
