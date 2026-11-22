//! Format one checked template with the same writer as a full SFC.
use super::super::{FormatResult, GlyphFormatter, estimate_output_size, template_block};
use super::{TemplateOnlyDescriptor, template_only};
use crate::{EndOfLine, FormatError, FormatOptions};
use vize_l0::String;

impl GlyphFormatter<'_> {
    /// Format the complete original source selected by L1. The descriptor
    /// guarantees a single HTML template with only ASCII whitespace outside
    /// the tags, so no block sort or root-content attachments are needed.
    pub fn format_observed_template(
        &self,
        observed: &TemplateOnlyDescriptor<'_>,
    ) -> Result<FormatResult, FormatError> {
        let source = observed.block().root_source();
        let write = |options: &FormatOptions| {
            let block = template_only(observed);
            let mut output =
                Vec::with_capacity(estimate_output_size(source, core::slice::from_ref(&block)));
            template_block::write_template_block(
                &mut output,
                &block,
                options,
                source,
                self.vue_version,
                self.preserve_template_whitespace,
            )?;
            // The block writer ends with the closing tag even for empty content.
            output.extend_from_slice(options.newline_bytes());
            // SAFETY: the same template writer emits UTF-8 source and formatting
            // text; the only subsequent bytes are an ASCII line ending.
            let code = unsafe { String::from_utf8_unchecked(output) };
            let changed = code != source;
            Ok(FormatResult { code, changed })
        };
        if self.options.end_of_line == EndOfLine::Auto {
            self.options.format_with_source_line_ending(source, write)
        } else {
            write(self.options)
        }
    }
}
