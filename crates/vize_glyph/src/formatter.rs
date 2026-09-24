//! High-performance formatter implementation for Vue SFC.
//!
//! Uses arena allocation and zero-copy techniques for maximum performance.

mod block_indent;
mod custom_block;
mod opening_tag;
mod raw_mask;
mod script_block;
mod style_block;
mod template_block;
mod template_indent;

use crate::error::FormatError;
use crate::options::FormatOptions;
use std::borrow::Cow;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_s0::{Allocator, FxHashMap, String};

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

impl Block<'_> {
    fn location(&self) -> &vize_atelier_sfc::BlockLocation {
        match self {
            Self::Script(block) => &block.loc,
            Self::Template(block) => &block.loc,
            Self::Style(block) => &block.loc,
            Self::Custom(block) => &block.loc,
        }
    }
}

impl<'a> GlyphFormatter<'a> {
    /// Create a new formatter with the given options and allocator
    #[inline]
    pub fn new(options: &'a FormatOptions, allocator: &'a Allocator) -> Self {
        Self { options, allocator }
    }

    /// Format a Vue SFC source string
    pub fn format(&self, source: &str) -> Result<FormatResult, FormatError> {
        // Parse the SFC
        let descriptor = parse_sfc(source, SfcParseOptions::default())?;
        let newline = self.options.newline_bytes();

        // Pre-calculate output size for efficient allocation
        let estimated_size = self.estimate_output_size(source, &descriptor);
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
                // CSS cascade order is semantic. Canonicalize the style group
                // after template, but let stable sorting retain authored order
                // across scoped/module/plain style blocks.
                3
            } else {
                style.loc.tag_start
            };
            blocks.push((order, Block::Style(style)));
        }
        for block in &descriptor.custom_blocks {
            let order = if self.options.sort_blocks {
                4
            } else {
                block.loc.tag_start
            };
            blocks.push((order, Block::Custom(block)));
        }

        // The SFC descriptor contains blocks but not the top-level content
        // between them. Attach each gap to the following block before block
        // sorting, so comments stay with the block they document. The document
        // prologue is separate and remains at the start after sorting.
        let mut source_order: Vec<_> = blocks.iter().map(|(_, block)| block.location()).collect();
        source_order.sort_by_key(|loc| loc.tag_start);
        let first_tag_start = source_order
            .first()
            .map_or(source.len(), |loc| loc.tag_start);
        let prologue = source.get(..first_tag_start).unwrap_or_default().trim();
        let mut leading_content = FxHashMap::default();
        let mut previous_end = first_tag_start;
        for loc in source_order {
            if let Some(content) = source
                .get(previous_end..loc.tag_start)
                .map(str::trim)
                .filter(|content| !content.is_empty())
            {
                leading_content.insert(loc.tag_start, content);
            }
            previous_end = loc.tag_end;
        }
        let trailing_content = source.get(previous_end..).unwrap_or_default().trim();

        blocks.sort_by_key(|(order, _)| *order);

        if !prologue.is_empty() {
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
            if let Some(content) = leading_content.get(&block.location().tag_start) {
                output.extend_from_slice(content.as_bytes());
                output.extend_from_slice(newline);
                output.extend_from_slice(newline);
            }
            match block {
                Block::Script(script) => script_block::write_script_block(
                    &mut output,
                    script,
                    self.options,
                    self.allocator,
                    source,
                )?,
                Block::Template(template) => template_block::write_template_block(
                    &mut output,
                    template,
                    self.options,
                    source,
                )?,
                Block::Style(style) => {
                    style_block::write_style_block(&mut output, style, self.options, source)?
                }
                Block::Custom(block) => {
                    custom_block::format(&mut output, block, self.options, source)?
                }
            }
        }

        if !trailing_content.is_empty() && !blocks.is_empty() {
            output.extend_from_slice(newline);
            output.extend_from_slice(newline);
            output.extend_from_slice(trailing_content.as_bytes());
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
}

fn write_remaining_attrs(
    output: &mut Vec<u8>,
    attrs: &FxHashMap<Cow<'_, str>, Cow<'_, str>>,
    handled: &[&str],
) {
    let mut remaining_attrs: Vec<_> = attrs
        .iter()
        .filter(|(name, _)| !handled.contains(&name.as_ref()))
        .collect();
    remaining_attrs.sort_by(|(a, _), (b, _)| a.as_ref().cmp(b.as_ref()));

    for (name, value) in remaining_attrs {
        let value = if value.is_empty() {
            None
        } else {
            Some(value.as_ref())
        };
        write_attr(output, name, value);
    }
}

fn write_attr(output: &mut Vec<u8>, name: &str, value: Option<&str>) {
    output.push(b' ');
    output.extend_from_slice(name.as_bytes());
    if let Some(value) = value {
        output.push(b'=');
        crate::attribute::write_attr_value(value, |s| output.extend_from_slice(s.as_bytes()));
    }
}
