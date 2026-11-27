//! High-performance formatter implementation for Vue SFC.
//!
//! Uses arena allocation and zero-copy techniques for maximum performance.

mod block_indent;
mod blocks;
mod custom_block;
mod opening_tag;
mod raw_mask;
mod root_content;
mod script_block;
mod style_block;
mod template_block;
mod template_indent;

use crate::VueVersion;
use crate::error::FormatError;
use crate::options::{BlockSpacing, FormatOptions};
use blocks::{BlockAttrs, BlockKind, BlockLocation, SfcBlock};
use vize_l0::string_builder::TextBuf;
use vize_l0::{Allocator, FxHashMap, String};

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
    vue_version: VueVersion,
    sort_imports: Option<&'a crate::ImportSortOptions>,
    preserve_template_whitespace: bool,
}

impl<'a> GlyphFormatter<'a> {
    /// Create a new formatter with the given options and allocator
    #[inline]
    pub fn new(options: &'a FormatOptions, allocator: &'a Allocator) -> Self {
        Self::new_with_vue_version(options, allocator, VueVersion::V3)
    }

    /// Create a formatter using the project's explicit Vue version.
    pub fn new_with_vue_version(
        options: &'a FormatOptions,
        allocator: &'a Allocator,
        vue_version: VueVersion,
    ) -> Self {
        Self {
            options,
            allocator,
            vue_version,
            sort_imports: None,
            preserve_template_whitespace: false,
        }
    }

    /// Apply validated import sorting to both script blocks.
    pub fn with_sort_imports(mut self, options: Option<&'a crate::ImportSortOptions>) -> Self {
        self.sort_imports = options;
        self
    }

    /// Preserve the complete authored HTML template body when its compiler keeps
    /// whitespace. Template layout cannot then change rendered text boundaries.
    pub fn with_template_whitespace_preserved(mut self, preserve: bool) -> Self {
        self.preserve_template_whitespace = preserve;
        self
    }

    /// Format a Vue SFC source string
    pub fn format(&self, source: &str) -> Result<FormatResult, FormatError> {
        if source.starts_with('\u{feff}') {
            return self.format_bom(source);
        }
        self.format_document(source)
    }

    #[inline]
    fn format_document(&self, source: &str) -> Result<FormatResult, FormatError> {
        if self.options.end_of_line == crate::EndOfLine::Auto {
            return self
                .options
                .format_with_source_line_ending(source, |options| {
                    GlyphFormatter {
                        options,
                        allocator: self.allocator,
                        vue_version: self.vue_version,
                        sort_imports: self.sort_imports,
                        preserve_template_whitespace: self.preserve_template_whitespace,
                    }
                    .format_document(source)
                });
        }
        let blocks = blocks::split(source)?;
        let newline = self.options.newline_string();

        // Pre-calculate output size for efficient allocation
        let estimated_size = estimate_output_size(source, &blocks);
        let mut output = TextBuf::with_capacity(estimated_size);

        // Blocks do not own the top-level content between them. Attach each
        // gap to the following block before block sorting, so comments stay
        // with the block they document. The document prologue is separate and
        // remains at the start after sorting.
        let first_tag_start = blocks
            .first()
            .map_or(source.len(), |block| block.loc.tag_start);
        let original_prologue = source.get(..first_tag_start).unwrap_or_default();
        let (prologue, first_attachment) = if blocks.is_empty() {
            (original_prologue.trim(), None)
        } else {
            root_content::split_prologue(original_prologue)
        };
        // `BlankLine` writes one blank line between root nodes. `Preserve`
        // reads each separator from the source, as Prettier does: an authored
        // blank line (any run of them) becomes one, and none adds none.
        let preserve_spacing = self.options.block_spacing == BlockSpacing::Preserve;
        let prologue_blank = !preserve_spacing
            || first_attachment.is_some()
            || root_content::has_blank_line(trailing_whitespace(original_prologue));
        let mut leading_content = FxHashMap::default();
        let mut blank_before = FxHashMap::default();
        if let Some(content) = first_attachment {
            leading_content.insert(first_tag_start, (content, true));
        }
        let mut previous_end = first_tag_start;
        for (index, loc) in blocks.iter().map(|block| &block.loc).enumerate() {
            let gap = source.get(previous_end..loc.tag_start).unwrap_or_default();
            let content = gap.trim();
            if !content.is_empty() {
                let attached = if preserve_spacing {
                    !root_content::has_blank_line(trailing_whitespace(gap))
                } else {
                    root_content::is_attached_comment(gap)
                };
                leading_content.insert(loc.tag_start, (content, attached));
            }
            if preserve_spacing && index > 0 {
                let blank = root_content::has_blank_line(leading_whitespace(gap));
                blank_before.insert(loc.tag_start, blank);
            }
            previous_end = loc.tag_end;
        }
        let trailing_gap = source.get(previous_end..).unwrap_or_default();
        let trailing_content = trailing_gap.trim();
        let trailing_blank =
            !preserve_spacing || root_content::has_blank_line(leading_whitespace(trailing_gap));

        // Sorting is stable: style and custom blocks keep their authored order
        // within their group. CSS cascade order is semantic.
        let mut blocks: Vec<_> = blocks
            .iter()
            .map(|block| {
                let order = if self.options.sort_blocks {
                    match block.kind {
                        BlockKind::Script => 0,
                        BlockKind::ScriptSetup => 1,
                        BlockKind::Template => 2,
                        BlockKind::Style => 3,
                        BlockKind::Custom => 4,
                    }
                } else {
                    block.loc.tag_start
                };
                (order, block)
            })
            .collect();
        blocks.sort_by_key(|(order, _)| *order);

        if !prologue.is_empty() {
            output.push_str(prologue);
            if !blocks.is_empty() {
                output.push_str(newline);
                if prologue_blank {
                    output.push_str(newline);
                }
            }
        }

        // Format each block in order
        for (i, (_, block)) in blocks.iter().enumerate() {
            if i > 0 {
                output.push_str(newline);
                // A block that sorting moved off the source start has no
                // authored separator and takes the blank line.
                if blank_before
                    .get(&block.loc.tag_start)
                    .copied()
                    .unwrap_or(true)
                {
                    output.push_str(newline);
                }
            }
            if let Some((content, attached)) = leading_content.get(&block.loc.tag_start) {
                output.push_str(content);
                output.push_str(newline);
                if !attached {
                    output.push_str(newline);
                }
            }
            match block.kind {
                BlockKind::Script | BlockKind::ScriptSetup => script_block::write_script_block(
                    &mut output,
                    block,
                    self.options,
                    self.allocator,
                    source,
                    self.sort_imports,
                )?,
                BlockKind::Template => template_block::write_template_block(
                    &mut output,
                    block,
                    self.options,
                    source,
                    self.vue_version,
                    self.preserve_template_whitespace,
                )?,
                BlockKind::Style => {
                    style_block::write_style_block(&mut output, block, self.options, source)?
                }
                BlockKind::Custom => custom_block::format(
                    &mut output,
                    block,
                    self.options,
                    source,
                    self.vue_version,
                )?,
            }
        }

        if !trailing_content.is_empty() && !blocks.is_empty() {
            output.push_str(newline);
            if trailing_blank {
                output.push_str(newline);
            }
            output.push_str(trailing_content);
        }

        // Trim trailing whitespace efficiently
        let kept = output.trim_end_matches(['\n', '\r', ' ', '\t']).len();
        output.truncate(kept);
        if !output.is_empty() {
            output.push_str(newline);
        }

        let code = output.into_compact();
        let changed = code != source;

        Ok(FormatResult { code, changed })
    }

    // A document BOM is an encoding marker, not an SFC prologue block. Keep
    // the uncommon prefix outside the ordinary formatting allocation frame.
    #[cold]
    #[inline(never)]
    fn format_bom(&self, source: &str) -> Result<FormatResult, FormatError> {
        let result = self.format_document(source.strip_prefix('\u{feff}').unwrap_or(source))?;
        let mut code = String::with_capacity(result.code.len() + '\u{feff}'.len_utf8());
        code.push('\u{feff}');
        code.push_str(result.code.as_str());
        let changed = code.as_str() != source;
        Ok(FormatResult { code, changed })
    }
}

#[inline]
fn leading_whitespace(gap: &str) -> &str {
    gap.get(..gap.len() - gap.trim_start().len())
        .unwrap_or_default()
}

#[inline]
fn trailing_whitespace(gap: &str) -> &str {
    gap.get(gap.trim_end().len()..).unwrap_or_default()
}

/// Estimate output size for pre-allocation
#[inline]
fn estimate_output_size(source: &str, blocks: &[SfcBlock<'_>]) -> usize {
    let mut size = source.len();

    // Add extra space for potential formatting changes
    if blocks
        .iter()
        .any(|block| matches!(block.kind, BlockKind::Script | BlockKind::ScriptSetup))
    {
        size += 256; // Extra space for script formatting
    }
    if blocks.iter().any(|block| block.kind == BlockKind::Template) {
        size += 128; // Extra space for template indentation
    }

    size
}

/// Write block attributes in authored order when attribute sorting is off,
/// matching how template attributes behave. Returns `false` when the caller
/// should print its canonical order instead.
fn write_authored_attrs(
    output: &mut TextBuf,
    attrs: &BlockAttrs<'_>,
    source: &str,
    loc: &BlockLocation,
    options: &FormatOptions,
) -> bool {
    if options.sort_attributes {
        return false;
    }
    let Some(order) =
        opening_tag::authored_attr_order(source, loc, attrs.len(), |name| attrs.contains_key(name))
    else {
        return false;
    };
    for name in order {
        let value = attrs
            .get(name)
            .map(|value| value.as_ref())
            .filter(|value| !value.is_empty());
        write_attr(output, name, value);
    }
    true
}

fn write_remaining_attrs(output: &mut TextBuf, attrs: &BlockAttrs<'_>, handled: &[&str]) {
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

fn write_attr(output: &mut TextBuf, name: &str, value: Option<&str>) {
    output.push(' ');
    output.push_str(name);
    if let Some(value) = value {
        output.push('=');
        crate::attribute::write_attr_value(value, |s| output.push_str(s));
    }
}
