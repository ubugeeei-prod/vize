//! Shared emission of already parsed template attributes.
use super::TemplateFormatter;
use crate::template::attributes::{
    AttributeLayout, ParsedAttribute, render_attribute, should_use_multiline_attrs,
    write_rendered_attributes,
};
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
            );

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
}
