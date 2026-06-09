//! Block and cursor-position resolution for SFC and art (`*.art.vue`) files.

use vize_atelier_sfc::SfcDescriptor;

use super::VirtualLanguage;

/// Information about cursor position within an art variant template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtVariantInfo {
    /// Index of the variant in the art descriptor
    pub variant_index: usize,
    /// Byte offset where the variant template content starts in the art file
    pub template_start: usize,
    /// Byte offset where the variant template content ends in the art file
    pub template_end: usize,
    /// Cursor offset relative to the start of the variant template content
    pub relative_offset: usize,
}

/// Where the cursor is within an art block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtCursorPosition {
    /// In `<art ...>` tag attributes
    ArtTag,
    /// In `<variant ...>` tag attributes (variant index)
    VariantTag(usize),
    /// Inside variant template content
    VariantTemplate(ArtVariantInfo),
    /// Between variants (art content area)
    ArtContent,
}

/// Helper to determine the virtual language from a block position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Template,
    Script,
    ScriptSetup,
    Style(usize),
    Art(ArtCursorPosition),
}

impl BlockType {
    /// Get the virtual language for this block type.
    #[inline]
    pub fn language(&self) -> VirtualLanguage {
        match self {
            BlockType::Template => VirtualLanguage::Template,
            BlockType::Script => VirtualLanguage::Script,
            BlockType::ScriptSetup => VirtualLanguage::ScriptSetup,
            BlockType::Style(_) => VirtualLanguage::Style,
            BlockType::Art(_) => VirtualLanguage::Template,
        }
    }
}

/// Find which block contains the given offset in an SFC.
pub fn find_block_at_offset(descriptor: &SfcDescriptor, offset: usize) -> Option<BlockType> {
    // Check template
    if let Some(ref template) = descriptor.template
        && offset >= template.loc.start
        && offset < template.loc.end
    {
        return Some(BlockType::Template);
    }

    // Check script
    if let Some(ref script) = descriptor.script
        && offset >= script.loc.start
        && offset < script.loc.end
    {
        return Some(BlockType::Script);
    }

    // Check script setup
    if let Some(ref script_setup) = descriptor.script_setup
        && offset >= script_setup.loc.start
        && offset < script_setup.loc.end
    {
        return Some(BlockType::ScriptSetup);
    }

    // Check styles
    for (i, style) in descriptor.styles.iter().enumerate() {
        if offset >= style.loc.start && offset < style.loc.end {
            return Some(BlockType::Style(i));
        }
    }

    // Check custom blocks (art, i18n, etc.)
    for custom in descriptor.custom_blocks.iter() {
        if custom.block_type == "art" && offset >= custom.loc.start && offset < custom.loc.end {
            return Some(BlockType::Art(ArtCursorPosition::ArtContent));
        }
    }

    None
}

/// Find which block contains the given offset in an art file (*.art.vue).
///
/// Uses `vize_musea::parse_art()` to determine cursor position within art variant templates.
pub fn find_art_block_at_offset(source: &str, offset: usize) -> Option<BlockType> {
    // First check SFC blocks (script, style)
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: Default::default(),
        ..Default::default()
    };

    if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(source, options) {
        // Check script/script_setup/style blocks
        if let Some(ref script) = descriptor.script
            && offset >= script.loc.start
            && offset < script.loc.end
        {
            return Some(BlockType::Script);
        }
        if let Some(ref script_setup) = descriptor.script_setup
            && offset >= script_setup.loc.start
            && offset < script_setup.loc.end
        {
            return Some(BlockType::ScriptSetup);
        }
        for (i, style) in descriptor.styles.iter().enumerate() {
            if offset >= style.loc.start && offset < style.loc.end {
                return Some(BlockType::Style(i));
            }
        }
    }

    // Parse as art file to determine variant position
    let allocator = vize_carton::Bump::new();
    let Ok(art_desc) =
        vize_musea::parse_art(&allocator, source, vize_musea::ArtParseOptions::default())
    else {
        return None;
    };

    for (i, variant) in art_desc.variants.iter().enumerate() {
        if let Some(ref loc) = variant.loc {
            let variant_start = loc.start as usize;
            let variant_end = loc.end as usize;

            if offset >= variant_start && offset < variant_end {
                let template_ptr = variant.template.as_ptr() as usize;
                let source_ptr = source.as_ptr() as usize;
                let trimmed_template_start = if variant.template.is_empty() {
                    find_variant_template_body_range(source, variant_start, variant_end)
                        .map(|(body_start, _)| body_start)
                        .unwrap_or(variant_start)
                } else {
                    template_ptr.saturating_sub(source_ptr)
                };
                let trimmed_template_end = trimmed_template_start + variant.template.len();
                let (body_start, body_end) =
                    find_variant_template_body_range(source, variant_start, variant_end)
                        .unwrap_or((trimmed_template_start, trimmed_template_end));

                if offset >= body_start && offset < body_end {
                    let relative_offset = if offset <= trimmed_template_start {
                        0
                    } else if offset >= trimmed_template_end {
                        variant.template.len()
                    } else {
                        offset - trimmed_template_start
                    };

                    return Some(BlockType::Art(ArtCursorPosition::VariantTemplate(
                        ArtVariantInfo {
                            variant_index: i,
                            template_start: trimmed_template_start,
                            template_end: trimmed_template_end,
                            relative_offset,
                        },
                    )));
                }

                return Some(BlockType::Art(ArtCursorPosition::VariantTag(i)));
            }
        }
    }

    Some(BlockType::Art(ArtCursorPosition::ArtContent))
}

fn find_variant_template_body_range(
    source: &str,
    variant_start: usize,
    variant_end: usize,
) -> Option<(usize, usize)> {
    if variant_start >= variant_end || variant_end > source.len() {
        return None;
    }

    let tag_end = source[variant_start..variant_end].find('>')? + variant_start;
    let body_start = tag_end + 1;
    let close_start = source[body_start..variant_end].rfind("</variant>")? + body_start;

    Some((body_start, close_start))
}
