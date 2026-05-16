//! Style virtual code generation.
//!
//! Preserves style content with 1:1 source mapping for CSS features.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use vize_atelier_sfc::SfcStyleBlock;

use super::{
    MappingFeatures, SourceMap, SourceMapping, SourceRange, VirtualDocument, VirtualLanguage,
};
use vize_carton::cstr;

/// Style code generator.
pub struct StyleCodeGenerator {
    /// Block offset in original SFC
    block_offset: u32,
}

impl StyleCodeGenerator {
    /// Create a new style code generator.
    #[inline]
    pub fn new() -> Self {
        Self { block_offset: 0 }
    }

    /// Generate virtual CSS from a style block.
    ///
    /// This is optimized for minimal allocations - the content is used directly
    /// with a single 1:1 mapping for the entire block.
    #[inline]
    pub fn generate(&mut self, style: &SfcStyleBlock, index: usize) -> VirtualDocument {
        self.block_offset = style.loc.start as u32;

        let content = style.content.as_ref();
        let content_len = content.len() as u32;

        // Create a single 1:1 mapping for the entire style content
        // This is the most efficient approach for CSS
        let mappings = if content_len > 0 {
            vec![SourceMapping::with_features(
                SourceRange::new(0, content_len),
                SourceRange::new(0, content_len),
                MappingFeatures::all(),
            )]
        } else {
            Vec::new()
        };

        let mut source_map = SourceMap::from_mappings(mappings);
        source_map.set_block_offset(self.block_offset);

        // Determine the language based on the lang attribute
        let extension = style.lang.as_ref().map(|l| l.as_ref()).unwrap_or("css");

        VirtualDocument {
            uri: cstr!("__style_{index}.{extension}").to_string(),
            content: content.to_string(),
            language: VirtualLanguage::Style,
            source_map,
        }
    }

    /// Generate with scoped CSS transformation info.
    ///
    /// For scoped styles, we add metadata but keep the content intact.
    /// Actual scoping is done at compile time, not in the LSP.
    #[inline]
    pub fn generate_scoped(
        &mut self,
        style: &SfcStyleBlock,
        index: usize,
        _scope_id: &str,
    ) -> VirtualDocument {
        // For LSP purposes, scoped styles are handled the same way
        // The actual scoping happens during compilation
        self.generate(style, index)
    }
}

impl Default for StyleCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Style block metadata for LSP features.
#[derive(Debug, Clone)]
pub struct StyleMetadata {
    /// Whether the style is scoped
    pub scoped: bool,
    /// CSS module name (if using CSS modules)
    pub module: Option<String>,
    /// Language (css, scss, less, etc.)
    pub lang: String,
    /// Source range in original SFC
    pub range: SourceRange,
}

impl StyleMetadata {
    /// Create metadata from a style block.
    #[inline]
    pub fn from_block(style: &SfcStyleBlock) -> Self {
        Self {
            scoped: style.scoped,
            module: style.module.as_ref().map(|s| s.to_string()),
            lang: style
                .lang
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "css".to_string()),
            range: SourceRange::new(style.loc.start as u32, style.loc.end as u32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{StyleCodeGenerator, StyleMetadata};
    use std::borrow::Cow;
    use vize_atelier_sfc::{BlockLocation, SfcStyleBlock};

    fn make_style_block(content: &str, scoped: bool) -> SfcStyleBlock<'static> {
        SfcStyleBlock {
            content: Cow::Owned(content.to_string()),
            loc: BlockLocation {
                start: 0,
                end: content.len(),
                tag_start: 0,
                tag_end: content.len(),
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: content.len(),
            },
            lang: None,
            scoped,
            module: None,
            attrs: Default::default(),
            src: None,
        }
    }

    #[test]
    fn test_style_code_generator() {
        let content = ".container { color: red; }";
        let style = make_style_block(content, false);

        let mut gen = StyleCodeGenerator::new();
        let doc = gen.generate(&style, 0);

        assert_eq!(doc.content, content);
        assert_eq!(doc.source_map.len(), 1);
    }

    #[test]
    fn test_style_metadata() {
        let style = make_style_block(".test {}", true);
        let meta = StyleMetadata::from_block(&style);

        assert!(meta.scoped);
        assert_eq!(meta.lang, "css");
    }
}
