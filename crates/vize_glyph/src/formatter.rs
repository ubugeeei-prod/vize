//! High-performance formatter implementation for Vue SFC.
//!
//! Uses arena allocation and zero-copy techniques for maximum performance.

mod artifact;
mod custom_block;
mod output;
mod raw_mask;

pub use artifact::{
    GlyphFormatProduct, GlyphFormatProvider, GlyphFormatSettingsInput,
    register_glyph_atlas_provider, register_glyph_format_provider,
};

pub(crate) fn format_with_atlas(
    source: &str,
    options: &FormatOptions,
) -> Result<FormatResult, FormatError> {
    artifact::FormatterArtifactGraph::new(source, options)?.format()
}

use crate::error::FormatError;
use crate::options::FormatOptions;
use crate::script;
use crate::style;
use crate::template;
use output::{document_prologue, write_attr, write_remaining_attrs};
use raw_mask::compute_raw_line_mask;
use vize_carton::{Allocator, String, ToCompactString};

/// Result of formatting a Vue SFC
#[derive(Debug, Clone)]
pub struct FormatResult {
    /// The formatted code
    pub code: String,

    /// Whether the code was changed
    pub changed: bool,
}

/// High-performance formatter for Vue Single File Components
///
/// Uses arena allocation for efficient memory management during formatting.
pub struct GlyphFormatter<'a> {
    options: &'a FormatOptions,
    allocator: &'a Allocator,
}

enum Block<'b> {
    Script(&'b vize_atelier_sfc::SfcScriptBlock<'b>),
    Template(&'b vize_atelier_sfc::SfcTemplateBlock<'b>),
    Style(&'b vize_atelier_sfc::SfcStyleBlock<'b>),
    Custom(&'b vize_atelier_sfc::SfcCustomBlock<'b>),
}

impl<'a> GlyphFormatter<'a> {
    /// Create a new formatter with the given options and allocator
    #[inline]
    pub fn new(options: &'a FormatOptions, allocator: &'a Allocator) -> Self {
        Self { options, allocator }
    }

    /// Format a Vue SFC source string
    pub fn format(&self, source: &str) -> Result<FormatResult, FormatError> {
        let arena_source = self.allocator.alloc_str(source);
        let descriptor = vize_atelier_sfc::parse_sfc(arena_source, Default::default())?;
        self.format_descriptor_core(arena_source, &descriptor)
    }

    fn format_descriptor_core(
        &self,
        source: &str,
        descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    ) -> Result<FormatResult, FormatError> {
        let newline = self.options.newline_bytes();

        // Pre-calculate output size for efficient allocation
        let estimated_size = self.estimate_output_size(source, descriptor);
        let mut output = Vec::with_capacity(estimated_size);

        // Collect all blocks with their sort keys
        let mut blocks: Vec<(usize, Block<'_>)> = Vec::new();

        if let Some(script) = &descriptor.script {
            let order = if self.options.sort_blocks {
                0
            } else {
                script.loc.tag_start
            };
            blocks.push((order, Block::Script(script)));
        }
        if let Some(script_setup) = &descriptor.script_setup {
            let order = if self.options.sort_blocks {
                1
            } else {
                script_setup.loc.tag_start
            };
            blocks.push((order, Block::Script(script_setup)));
        }
        if let Some(template) = &descriptor.template {
            let order = if self.options.sort_blocks {
                2
            } else {
                template.loc.tag_start
            };
            blocks.push((order, Block::Template(template)));
        }
        for style in &descriptor.styles {
            let order = if self.options.sort_blocks {
                if style.scoped { 3 } else { 4 }
            } else {
                style.loc.tag_start
            };
            blocks.push((order, Block::Style(style)));
        }
        for block in &descriptor.custom_blocks {
            let order = if self.options.sort_blocks {
                5
            } else {
                block.loc.tag_start
            };
            blocks.push((order, Block::Custom(block)));
        }

        blocks.sort_by_key(|(order, _)| *order);

        if let Some(prologue) = document_prologue(source, &blocks) {
            output.extend_from_slice(prologue.as_bytes());
            if !blocks.is_empty() {
                output.extend_from_slice(newline);
                output.extend_from_slice(newline);
            }
        }

        // Format each block in order
        for (i, (_, block)) in blocks.iter().enumerate() {
            if i > 0 {
                output.extend_from_slice(newline);
                output.extend_from_slice(newline);
            }
            match block {
                Block::Script(script) => {
                    self.format_script_block_fast(&mut output, script)?;
                }
                Block::Template(template) => {
                    self.format_template_block_fast(&mut output, template)?;
                }
                Block::Style(style) => {
                    self.format_style_block_fast(&mut output, style)?;
                }
                Block::Custom(block) => custom_block::format(&mut output, block, self.options)?,
            }
        }

        // Trim trailing whitespace efficiently
        while output
            .last()
            .is_some_and(|&b| b == b'\n' || b == b'\r' || b == b' ' || b == b'\t')
        {
            output.pop();
        }
        output.extend_from_slice(newline);

        // SAFETY: `output` is composed from the original UTF-8 SFC source plus
        // formatter output returned as `&str` and ASCII whitespace/newlines. All
        // byte slicing uses block ranges produced by the SFC parser, which are
        // source-owned UTF-8 boundaries. We keep the conversion unchecked to avoid
        // revalidating the full formatted document on every format run.
        let code = unsafe { String::from_utf8_unchecked(output) };
        let changed = code != source;

        Ok(FormatResult { code, changed })
    }

    /// Estimate output size for pre-allocation
    #[inline]
    fn estimate_output_size(
        &self,
        source: &str,
        descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    ) -> usize {
        let mut size = source.len();

        // Add extra space for potential formatting changes
        if descriptor.script_setup.is_some() || descriptor.script.is_some() {
            size += 256; // Extra space for script formatting
        }
        if descriptor.template.is_some() {
            size += 128; // Extra space for template indentation
        }

        size
    }

    /// Format a script block using byte operations
    #[inline]
    fn format_script_block_fast(
        &self,
        output: &mut Vec<u8>,
        block: &vize_atelier_sfc::SfcScriptBlock<'_>,
    ) -> Result<(), FormatError> {
        // Degrade gracefully on a script parse error: emit the original script
        // body trimmed but otherwise unchanged, rather than failing the whole
        // SFC format and dropping the template/style work. The script formatter
        // delegates to oxc, which round-trips decorated class components
        // (`@Component`/`@Prop()`/`@Emit()`) fine; this fallback only triggers
        // on genuinely unparseable TS, mirroring the style block's fallback to
        // trimmed content. (#1391)
        let trimmed = block.content.trim();
        let source_type =
            script::source_type_for_script_lang(block.lang.as_ref().map(|lang| lang.as_ref()));
        let formatted_content = script::format_script_content_stable(
            trimmed,
            self.options,
            self.allocator,
            source_type,
        )
        .unwrap_or_else(|_| trimmed.to_compact_string());

        // Build the opening tag using byte operations
        output.extend_from_slice(b"<script");
        if block.setup {
            write_attr(output, "setup", None);
        }
        if let Some(lang) = &block.lang {
            write_attr(output, "lang", Some(lang));
        }
        write_remaining_attrs(output, &block.attrs, &["setup", "lang"]);
        output.push(b'>');
        output.extend_from_slice(self.options.newline_bytes());

        // Add content with indentation if configured
        if self.options.vue_indent_script_and_style {
            let indent = self.options.indent_bytes();
            let trimmed = formatted_content
                .trim_end_matches('\n')
                .trim_end_matches('\r');
            for line in trimmed.as_bytes().split(|&b| b == b'\n') {
                if !line.is_empty() && line != b"\r" {
                    output.extend_from_slice(indent);
                }
                output.extend_from_slice(line);
                output.extend_from_slice(self.options.newline_bytes());
            }
        } else {
            output.extend_from_slice(formatted_content.as_bytes());
            if !formatted_content.ends_with('\n') {
                output.extend_from_slice(self.options.newline_bytes());
            }
        }

        output.extend_from_slice(b"</script>");

        Ok(())
    }

    /// Format a template block using byte operations
    #[inline]
    fn format_template_block_fast(
        &self,
        output: &mut Vec<u8>,
        block: &vize_atelier_sfc::SfcTemplateBlock<'_>,
    ) -> Result<(), FormatError> {
        let formatted_content = template::format_template_content(&block.content, self.options)?;

        // Build the opening tag
        output.extend_from_slice(b"<template");
        if let Some(lang) = &block.lang {
            write_attr(output, "lang", Some(lang));
        }
        write_remaining_attrs(output, &block.attrs, &["lang"]);
        output.push(b'>');
        output.extend_from_slice(self.options.newline_bytes());

        // Template content is always indented by one level from the template
        // tag — except inside whitespace-significant regions (`<pre>`,
        // `<textarea>`, `v-pre`) where the inner content must round-trip
        // byte-for-byte. The inner template formatter already preserves
        // those regions verbatim; here we make sure the SFC layer doesn't
        // re-indent each of their inner lines on top. (#963)
        let indent = self.options.indent_bytes();
        let trimmed = formatted_content
            .trim_end_matches('\n')
            .trim_end_matches('\r');
        let lines: Vec<&[u8]> = trimmed.as_bytes().split(|&b| b == b'\n').collect();
        let raw_mask = compute_raw_line_mask(&lines);
        for (i, line) in lines.iter().enumerate() {
            if !line.is_empty() && line != b"\r" && !raw_mask[i] {
                output.extend_from_slice(indent);
            }
            output.extend_from_slice(line);
            output.extend_from_slice(self.options.newline_bytes());
        }

        output.extend_from_slice(b"</template>");

        Ok(())
    }

    /// Format a style block using lightningcss for CSS
    #[inline]
    fn format_style_block_fast(
        &self,
        output: &mut Vec<u8>,
        block: &vize_atelier_sfc::SfcStyleBlock<'_>,
    ) -> Result<(), FormatError> {
        // Use lightningcss for plain CSS; for preprocessor languages, just trim
        let is_plain_css = block.lang.as_ref().is_none_or(|l| l.as_ref() == "css");
        let formatted_content = if is_plain_css {
            style::format_style_content(&block.content, self.options)
                .unwrap_or_else(|_| block.content.trim().to_compact_string())
        } else {
            block.content.trim().to_compact_string()
        };
        let formatted_content = formatted_content.as_str();

        // Build the opening tag
        output.extend_from_slice(b"<style");
        if block.scoped {
            write_attr(output, "scoped", None);
        }
        if let Some(lang) = &block.lang {
            write_attr(output, "lang", Some(lang));
        }
        write_remaining_attrs(output, &block.attrs, &["scoped", "lang"]);
        output.push(b'>');
        output.extend_from_slice(self.options.newline_bytes());

        // Add content with indentation if configured
        if self.options.vue_indent_script_and_style {
            let indent = self.options.indent_bytes();
            let trimmed = formatted_content
                .trim_end_matches('\n')
                .trim_end_matches('\r');
            for line in trimmed.as_bytes().split(|&b| b == b'\n') {
                if !line.is_empty() && line != b"\r" {
                    output.extend_from_slice(indent);
                }
                output.extend_from_slice(line);
                output.extend_from_slice(self.options.newline_bytes());
            }
        } else {
            output.extend_from_slice(formatted_content.as_bytes());
            if !formatted_content.ends_with('\n') {
                output.extend_from_slice(self.options.newline_bytes());
            }
        }

        output.extend_from_slice(b"</style>");

        Ok(())
    }
}
