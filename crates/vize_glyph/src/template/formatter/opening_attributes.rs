//! Shared emission of already parsed template attributes.
use super::TemplateFormatter;
use crate::template::attributes::{
    AttributeLayout, ParsedAttribute, render_attribute, should_use_multiline_attrs,
    write_rendered_attributes,
};
use unicode_width::UnicodeWidthStr;
use vize_l0::String;

impl TemplateFormatter<'_> {
    /// Emit attributes from the existing parsed, normalized and sorted list.
    #[inline]
    pub(super) fn write_opening_attributes(
        &self,
        output: &mut Vec<u8>,
        tag_name: &str,
        sorted_attrs: &[ParsedAttribute],
        depth: usize,
    ) -> bool {
        self.write_opening_attributes_with_prefix(output, tag_name, sorted_attrs, depth, false, 1)
    }

    /// A continued tag also owns the already emitted text on its physical line.
    #[inline]
    pub(super) fn write_opening_attributes_with_prefix(
        &self,
        output: &mut Vec<u8>,
        tag_name: &str,
        sorted_attrs: &[ParsedAttribute],
        depth: usize,
        measure_prefix: bool,
        compact_suffix_width: usize,
    ) -> bool {
        let mut closing_bracket_on_own_line = false;
        if !sorted_attrs.is_empty() {
            // Render each attribute exactly once; both the
            // multiline decision and emission below reuse this.
            let mut rendered: Vec<String> = Vec::with_capacity(sorted_attrs.len());
            rendered.extend(sorted_attrs.iter().map(render_attribute));

            let use_multiline = should_use_multiline_attrs(
                self.options,
                tag_name,
                sorted_attrs,
                &rendered,
                depth,
                self.indent,
            ) || (measure_prefix
                && self.continued_attributes_overflow(output, &rendered, compact_suffix_width));

            if use_multiline {
                let max_per_line = if self.options.single_attribute_per_line {
                    1
                } else {
                    self.options
                        .max_attributes_per_line
                        .unwrap_or(1) // default 1 when multiline
                        .max(1) as usize
                };

                write_rendered_attributes(
                    output,
                    sorted_attrs,
                    &rendered,
                    self.newline,
                    self.indent,
                    depth + 1,
                    &AttributeLayout {
                        options: self.options,
                        base_depth: self.base_depth,
                        max_per_line,
                    },
                );
                if !self.options.bracket_same_line {
                    output.extend_from_slice(self.newline);
                    self.write_indent(output, depth);
                    closing_bracket_on_own_line = true;
                }
            } else {
                for attr in &rendered {
                    output.push(b' ');
                    output.extend_from_slice(attr.as_bytes());
                }
            }
        }
        closing_bracket_on_own_line
    }

    fn continued_attributes_overflow(
        &self,
        output: &[u8],
        rendered: &[String],
        compact_suffix_width: usize,
    ) -> bool {
        let line = output
            .rsplit(|byte| matches!(byte, b'\r' | b'\n'))
            .next()
            .unwrap_or_default();
        let line = core::str::from_utf8(line).unwrap_or_default();
        let indent_width = self.options.tab_width as usize;
        let column = line.split('\t').map(str::width).sum::<usize>()
            + line.bytes().filter(|byte| *byte == b'\t').count() * indent_width
            + self.base_depth * indent_width;
        let attributes = rendered.iter().map(|attr| 1 + attr.width()).sum::<usize>();
        column + attributes + compact_suffix_width > self.options.print_width as usize
    }
}
