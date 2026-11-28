//! Shared emission of already parsed template attributes.
use super::TemplateFormatter;
use super::suppression::LineJoiner;
use crate::template::attributes::{
    AttributeLayout, ParsedAttribute, render_attribute, should_use_multiline_attrs,
    write_rendered_attributes,
};
use unicode_width::UnicodeWidthStr;
use vize_l0::String;
use vize_l0::string_builder::TextBuf;

impl TemplateFormatter<'_> {
    /// Emit attributes from the existing parsed, normalized and sorted list.
    #[inline]
    pub(super) fn write_opening_attributes(
        &self,
        output: &mut TextBuf,
        tag_name: &str,
        sorted_attrs: &[ParsedAttribute],
        depth: usize,
    ) -> bool {
        self.write_opening_attributes_with_prefix(output, tag_name, sorted_attrs, depth, None, 1)
    }

    /// A continued tag (`continued` carries its line joiner) also owns the
    /// already emitted text on its physical line.
    #[inline]
    pub(super) fn write_opening_attributes_with_prefix(
        &self,
        output: &mut TextBuf,
        tag_name: &str,
        sorted_attrs: &[ParsedAttribute],
        depth: usize,
        continued: Option<&LineJoiner<'_>>,
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
                (depth + self.base_depth) * self.options.tab_width as usize,
                compact_suffix_width,
            ) || continued.is_some_and(|joiner| {
                self.continued_attributes_overflow(
                    output,
                    tag_name,
                    &rendered,
                    depth,
                    compact_suffix_width,
                    joiner,
                )
            });

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
                    output.push_str(self.newline);
                    self.write_indent(output, depth);
                    closing_bracket_on_own_line = true;
                }
            } else {
                for attr in &rendered {
                    output.push(' ');
                    output.push_str(attr);
                }
            }
        }
        closing_bracket_on_own_line
    }

    /// Whether a continued tag's attributes must wrap. An overflowing tag
    /// that directly follows its parent's `>` stays whole when moving that
    /// `>` down makes it fit, as Prettier does.
    fn continued_attributes_overflow(
        &self,
        output: &mut TextBuf,
        tag_name: &str,
        rendered: &[String],
        depth: usize,
        compact_suffix_width: usize,
        joiner: &LineJoiner<'_>,
    ) -> bool {
        let line = super::current_line(output);
        let indent_width = self.options.tab_width as usize;
        let column = line.split('\t').map(str::width).sum::<usize>()
            + line.bytes().filter(|byte| *byte == b'\t').count() * indent_width
            + self.base_depth * indent_width;
        let attributes = rendered.iter().map(|attr| 1 + attr.width()).sum::<usize>();
        column + attributes + compact_suffix_width > self.options.print_width as usize
            && !(joiner.follows_opening_tag()
                && self.move_parent_bracket(
                    output,
                    tag_name,
                    depth,
                    attributes + compact_suffix_width,
                ))
    }

    /// Move the parent's `>` that directly precedes `<tag_name` to its own
    /// line at `depth`, when the tag, `tail_width` columns wide after its
    /// name, then fits there.
    #[cold]
    #[inline(never)]
    fn move_parent_bracket(
        &self,
        output: &mut TextBuf,
        tag_name: &str,
        depth: usize,
        tail_width: usize,
    ) -> bool {
        let Some(bracket) = output.len().checked_sub(tag_name.len() + 2) else {
            return false;
        };
        let Some(head) = output.get(..bracket) else {
            return false;
        };
        let start_column = (depth + self.base_depth) * self.options.tab_width as usize;
        if output.as_bytes().get(bracket) != Some(&b'>')
            || super::current_line(head).trim().is_empty()
            || start_column + 2 + tag_name.width() + tail_width > self.options.print_width as usize
        {
            return false;
        }
        output.truncate(bracket);
        output.push_str(self.newline);
        self.write_indent(output, depth);
        output.push_str("><");
        output.push_str(tag_name);
        true
    }
}
