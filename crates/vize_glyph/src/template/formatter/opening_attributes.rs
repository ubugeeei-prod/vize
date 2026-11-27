//! Shared emission of already parsed template attributes.
use super::TemplateFormatter;
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
        self.write_opening_attributes_with_prefix(
            output,
            tag_name,
            sorted_attrs,
            depth,
            Continued::No,
            1,
        )
        .bracket_on_own_line
    }

    /// A continued tag also owns the already emitted text on its physical line.
    #[inline]
    pub(super) fn write_opening_attributes_with_prefix(
        &self,
        output: &mut TextBuf,
        tag_name: &str,
        sorted_attrs: &[ParsedAttribute],
        depth: usize,
        continued: Continued,
        compact_suffix_width: usize,
    ) -> OpeningLine {
        let mut line = OpeningLine::default();
        if !sorted_attrs.is_empty() {
            // Render each attribute exactly once; both the
            // multiline decision and emission below reuse this.
            let mut rendered: Vec<String> = Vec::with_capacity(sorted_attrs.len());
            rendered.extend(sorted_attrs.iter().map(render_attribute));

            let start_column = (depth + self.base_depth) * self.options.tab_width as usize;
            let mut use_multiline = should_use_multiline_attrs(
                self.options,
                tag_name,
                sorted_attrs,
                &rendered,
                start_column,
                compact_suffix_width,
            );
            if !use_multiline
                && continued != Continued::No
                && self.continued_attributes_overflow(output, &rendered, compact_suffix_width)
            {
                // Prettier keeps a hugged child tag whole and moves its
                // parent's `>` down instead, when the tag then fits there.
                line.moved_parent_bracket = continued == Continued::AfterOpeningBracket
                    && self.move_parent_bracket(
                        output,
                        tag_name,
                        depth,
                        start_column + 1 + attributes_width(&rendered) + compact_suffix_width,
                    );
                use_multiline = !line.moved_parent_bracket;
            }

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
                    line.bracket_on_own_line = true;
                }
            } else {
                for attr in &rendered {
                    output.push(' ');
                    output.push_str(attr);
                }
            }
        }
        line
    }

    /// Move the parent's `>` that directly precedes `<tag_name` to its own line
    /// at `depth`, when the line it then starts fits `line_width` plus the tag.
    fn move_parent_bracket(
        &self,
        output: &mut TextBuf,
        tag_name: &str,
        depth: usize,
        line_width: usize,
    ) -> bool {
        let Some(bracket) = output.len().checked_sub(tag_name.len() + 2) else {
            return false;
        };
        let Some(head) = output.get(..bracket) else {
            return false;
        };
        if output.as_bytes().get(bracket) != Some(&b'>')
            || super::current_line(head).trim().is_empty()
            || line_width + 1 + tag_name.width() > self.options.print_width as usize
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

    fn continued_attributes_overflow(
        &self,
        output: &str,
        rendered: &[String],
        compact_suffix_width: usize,
    ) -> bool {
        let line = super::current_line(output);
        let indent_width = self.options.tab_width as usize;
        let column = line.split('\t').map(str::width).sum::<usize>()
            + line.bytes().filter(|byte| *byte == b'\t').count() * indent_width
            + self.base_depth * indent_width;
        column + attributes_width(rendered) + compact_suffix_width
            > self.options.print_width as usize
    }
}

/// How the attribute writer laid out an opening tag's line.
#[derive(Clone, Copy, Default)]
pub(super) struct OpeningLine {
    /// The tag's own `>` starts a new line below its attributes.
    pub(super) bracket_on_own_line: bool,
    /// The parent's `>` moved down so the tag could stay whole.
    pub(super) moved_parent_bracket: bool,
}

/// How an opening tag sits on its output line.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Continued {
    /// The tag starts its own line.
    No,
    /// The tag continues a line that other output already fills.
    Line,
    /// The tag directly follows its parent's opening `>`.
    AfterOpeningBracket,
}

#[inline]
fn attributes_width(rendered: &[String]) -> usize {
    rendered.iter().map(|attr| 1 + attr.width()).sum()
}
