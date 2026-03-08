//! Virtual code generator that transforms SFC into virtual documents.
//!
//! Uses arena allocation from vize_carton for optimal performance.
#![allow(clippy::disallowed_methods)]

use vize_atelier_sfc::SfcDescriptor;
use vize_carton::cstr;
use vize_carton::Bump;

use super::{
    ScriptCodeGenerator, StyleCodeGenerator, TemplateCodeGenerator, VirtualDocument,
    VirtualDocuments, VirtualLanguage,
};

/// Virtual code generator for SFC files.
///
/// This generator transforms Vue SFC files into virtual documents for each
/// embedded language (template, script, style). It uses arena allocation
/// for temporary parsing data to minimize allocations.
pub struct VirtualCodeGenerator {
    /// Template code generator (reusable)
    template_gen: TemplateCodeGenerator,
    /// Script code generator (reusable)
    script_gen: ScriptCodeGenerator,
    /// Style code generator (reusable)
    style_gen: StyleCodeGenerator,
}

impl VirtualCodeGenerator {
    /// Create a new virtual code generator.
    #[inline]
    pub fn new() -> Self {
        Self {
            template_gen: TemplateCodeGenerator::new(),
            script_gen: ScriptCodeGenerator::new(),
            style_gen: StyleCodeGenerator::new(),
        }
    }

    /// Generate virtual documents from an SFC descriptor.
    ///
    /// Uses the provided arena allocator for temporary parsing data,
    /// minimizing heap allocations during generation.
    pub fn generate<'a>(
        &mut self,
        descriptor: &SfcDescriptor<'a>,
        base_uri: &str,
    ) -> VirtualDocuments {
        // Create arena for temporary parsing data
        let allocator = Bump::new();

        let mut docs = VirtualDocuments::new();

        // Generate template virtual code
        if let Some(ref template) = descriptor.template {
            let template_content = template.content.as_ref();

            // Parse template with arena allocation
            let (ast, _errors) = vize_armature::parse(&allocator, template_content);

            // Set block offset for source mapping
            self.template_gen
                .set_block_offset(template.loc.start as u32);

            // Generate virtual TypeScript
            let mut template_doc = self.template_gen.generate(&ast, template_content);
            template_doc.uri = cstr!("{base_uri}.__template.ts").to_string();

            docs.template = Some(template_doc);
        }

        // Generate script virtual code
        if let Some(ref script) = descriptor.script {
            let mut script_doc = self.script_gen.generate(script, false);
            script_doc.uri = cstr!("{base_uri}.__script.ts").to_string();
            docs.script = Some(script_doc);
        }

        // Generate script setup virtual code
        if let Some(ref script_setup) = descriptor.script_setup {
            let mut script_doc = self.script_gen.generate(script_setup, true);
            script_doc.uri = cstr!("{base_uri}.__script_setup.ts").to_string();
            docs.script_setup = Some(script_doc);
        }

        // Generate style virtual codes
        for (i, style) in descriptor.styles.iter().enumerate() {
            let mut style_doc = self.style_gen.generate(style, i);
            let ext = style.lang.as_ref().map(|l| l.as_ref()).unwrap_or("css");
            style_doc.uri = cstr!("{base_uri}.__style_{i}.{ext}").to_string();
            docs.styles.push(style_doc);
        }

        // Arena is dropped here, freeing all temporary allocations

        docs
    }

    /// Generate virtual documents with explicit allocator.
    ///
    /// Use this when you want to control the allocator lifetime,
    /// for example when processing multiple files in a batch.
    pub fn generate_with_allocator<'a, 'alloc>(
        &mut self,
        descriptor: &SfcDescriptor<'a>,
        base_uri: &str,
        allocator: &'alloc Bump,
    ) -> VirtualDocuments {
        let mut docs = VirtualDocuments::new();

        // Generate template virtual code
        if let Some(ref template) = descriptor.template {
            let template_content = template.content.as_ref();

            // Parse template with provided allocator
            let (ast, _errors) = vize_armature::parse(allocator, template_content);

            self.template_gen
                .set_block_offset(template.loc.start as u32);
            let mut template_doc = self.template_gen.generate(&ast, template_content);
            template_doc.uri = cstr!("{base_uri}.__template.ts").to_string();

            docs.template = Some(template_doc);
        }

        // Generate script virtual code
        if let Some(ref script) = descriptor.script {
            let mut script_doc = self.script_gen.generate(script, false);
            script_doc.uri = cstr!("{base_uri}.__script.ts").to_string();
            docs.script = Some(script_doc);
        }

        // Generate script setup virtual code
        if let Some(ref script_setup) = descriptor.script_setup {
            let mut script_doc = self.script_gen.generate(script_setup, true);
            script_doc.uri = cstr!("{base_uri}.__script_setup.ts").to_string();
            docs.script_setup = Some(script_doc);
        }

        // Generate style virtual codes
        for (i, style) in descriptor.styles.iter().enumerate() {
            let mut style_doc = self.style_gen.generate(style, i);
            let ext = style.lang.as_ref().map(|l| l.as_ref()).unwrap_or("css");
            style_doc.uri = cstr!("{base_uri}.__style_{i}.{ext}").to_string();
            docs.styles.push(style_doc);
        }

        docs
    }

    /// Quick generation for a single template string.
    ///
    /// Useful for testing and single-file scenarios.
    #[inline]
    pub fn generate_template_only(&mut self, template_content: &str) -> Option<VirtualDocument> {
        let allocator = Bump::new();
        let (ast, _) = vize_armature::parse(&allocator, template_content);

        let mut doc = self.template_gen.generate(&ast, template_content);
        doc.uri = "__inline.__template.ts".to_string();

        Some(doc)
    }
}

impl Default for VirtualCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch generator for processing multiple SFC files efficiently.
///
/// Reuses a single arena allocator across multiple files to minimize
/// allocation overhead.
pub struct BatchVirtualCodeGenerator {
    /// Underlying generator
    generator: VirtualCodeGenerator,
    /// Shared allocator for batch processing
    allocator: Bump,
}

impl BatchVirtualCodeGenerator {
    /// Create a new batch generator.
    #[inline]
    pub fn new() -> Self {
        Self {
            generator: VirtualCodeGenerator::new(),
            allocator: Bump::new(),
        }
    }

    /// Create with pre-allocated capacity.
    ///
    /// Use this when you know approximately how much memory will be needed.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            generator: VirtualCodeGenerator::new(),
            allocator: Bump::with_capacity(capacity),
        }
    }

    /// Generate virtual documents for a single file.
    ///
    /// The allocator is reused but reset between calls.
    pub fn generate<'a>(
        &mut self,
        descriptor: &SfcDescriptor<'a>,
        base_uri: &str,
    ) -> VirtualDocuments {
        // Reset allocator for new file
        self.allocator.reset();

        self.generator
            .generate_with_allocator(descriptor, base_uri, &self.allocator)
    }

    /// Process multiple files in batch.
    ///
    /// More efficient than calling generate() repeatedly as it
    /// minimizes allocator resets.
    pub fn generate_batch<'a>(
        &mut self,
        files: &[(&SfcDescriptor<'a>, &str)],
    ) -> Vec<VirtualDocuments> {
        files
            .iter()
            .map(|(descriptor, uri)| {
                self.allocator.reset();
                self.generator
                    .generate_with_allocator(descriptor, uri, &self.allocator)
            })
            .collect()
    }

    /// Get memory usage statistics.
    #[inline]
    pub fn allocated_bytes(&self) -> usize {
        self.allocator.allocated_bytes()
    }
}

impl Default for BatchVirtualCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

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
    if let Some(ref template) = descriptor.template {
        if offset >= template.loc.start && offset < template.loc.end {
            return Some(BlockType::Template);
        }
    }

    // Check script
    if let Some(ref script) = descriptor.script {
        if offset >= script.loc.start && offset < script.loc.end {
            return Some(BlockType::Script);
        }
    }

    // Check script setup
    if let Some(ref script_setup) = descriptor.script_setup {
        if offset >= script_setup.loc.start && offset < script_setup.loc.end {
            return Some(BlockType::ScriptSetup);
        }
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
        if let Some(ref script) = descriptor.script {
            if offset >= script.loc.start && offset < script.loc.end {
                return Some(BlockType::Script);
            }
        }
        if let Some(ref script_setup) = descriptor.script_setup {
            if offset >= script_setup.loc.start && offset < script_setup.loc.end {
                return Some(BlockType::ScriptSetup);
            }
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
                // Determine template content position using pointer arithmetic
                let template_ptr = variant.template.as_ptr() as usize;
                let source_ptr = source.as_ptr() as usize;
                let template_start = template_ptr - source_ptr;
                let template_end = template_start + variant.template.len();

                if offset >= template_start && offset < template_end {
                    return Some(BlockType::Art(ArtCursorPosition::VariantTemplate(
                        ArtVariantInfo {
                            variant_index: i,
                            template_start,
                            template_end,
                            relative_offset: offset - template_start,
                        },
                    )));
                }

                return Some(BlockType::Art(ArtCursorPosition::VariantTag(i)));
            }
        }
    }

    Some(BlockType::Art(ArtCursorPosition::ArtContent))
}

#[cfg(test)]
mod tests {
    use super::{
        find_art_block_at_offset, find_block_at_offset, ArtCursorPosition,
        BatchVirtualCodeGenerator, BlockType, VirtualCodeGenerator, VirtualLanguage,
    };

    #[test]
    fn test_virtual_code_generator() {
        let source = r#"<template>
  <div>{{ message }}</div>
</template>

<script setup lang="ts">
const message = ref('hello')
</script>

<style scoped>
.container { color: red; }
</style>"#;

        let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();

        let mut gen = VirtualCodeGenerator::new();
        let docs = gen.generate(&descriptor, "test.vue");

        assert!(docs.template.is_some());
        assert!(docs.script_setup.is_some());
        assert_eq!(docs.styles.len(), 1);

        // Check template virtual code
        let template = docs.template.unwrap();
        assert!(template.content.contains("__VIZE_ctx.message"));
        assert!(!template.source_map.is_empty());
    }

    #[test]
    fn test_batch_generator() {
        let source1 = "<template><div>{{ a }}</div></template>";
        let source2 = "<template><div>{{ b }}</div></template>";

        let desc1 = vize_atelier_sfc::parse_sfc(source1, Default::default()).unwrap();
        let desc2 = vize_atelier_sfc::parse_sfc(source2, Default::default()).unwrap();

        let mut batch = BatchVirtualCodeGenerator::new();
        let results = batch.generate_batch(&[(&desc1, "file1.vue"), (&desc2, "file2.vue")]);

        assert_eq!(results.len(), 2);
        assert!(results[0].template.is_some());
        assert!(results[1].template.is_some());
    }

    #[test]
    fn test_find_block_at_offset() {
        let source = r#"<template>
  <div>test</div>
</template>

<script setup>
const x = 1
</script>"#;

        let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();

        // In template
        assert_eq!(
            find_block_at_offset(&descriptor, 15),
            Some(BlockType::Template)
        );

        // In script setup
        assert_eq!(
            find_block_at_offset(&descriptor, 60),
            Some(BlockType::ScriptSetup)
        );
    }

    #[test]
    fn test_find_block_at_offset_inline_art() {
        let source = r#"<template>
  <div>test</div>
</template>

<script setup>
const x = 1
</script>

<art title="Test" component="./Foo.vue">
  <variant name="Default" default>
    <Foo />
  </variant>
</art>"#;

        let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();

        // Verify custom_blocks contains the art block
        assert_eq!(descriptor.custom_blocks.len(), 1);
        assert_eq!(descriptor.custom_blocks[0].block_type, "art");

        // Offset inside <art> content area
        let art_content_start = descriptor.custom_blocks[0].loc.start;
        assert_eq!(
            find_block_at_offset(&descriptor, art_content_start + 5),
            Some(BlockType::Art(ArtCursorPosition::ArtContent))
        );

        // In template - should still be Template
        assert_eq!(
            find_block_at_offset(&descriptor, 15),
            Some(BlockType::Template)
        );

        // Outside any block
        assert_eq!(find_block_at_offset(&descriptor, 0), None);
    }

    #[test]
    fn test_block_type_art_language() {
        assert_eq!(
            BlockType::Art(ArtCursorPosition::ArtContent).language(),
            VirtualLanguage::Template
        );
    }

    #[test]
    fn test_find_art_block_at_offset() {
        let source = r#"<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button>Click me</Button>
  </variant>
</art>

<script setup lang="ts">
import Button from './Button.vue'
</script>"#;

        // In script setup
        let script_offset = source.find("import Button").unwrap();
        assert_eq!(
            find_art_block_at_offset(source, script_offset),
            Some(BlockType::ScriptSetup)
        );

        // In variant template content
        let template_offset = source.find("<Button>Click me</Button>").unwrap();
        let result = find_art_block_at_offset(source, template_offset);
        assert!(matches!(
            result,
            Some(BlockType::Art(ArtCursorPosition::VariantTemplate(_)))
        ));

        // In art content (between variants)
        let art_content_offset = source.find("\n  <variant").unwrap() + 1;
        // This offset is just before <variant, which is inside the <art> but before variant tag starts
        // It should be ArtContent
        assert!(matches!(
            find_art_block_at_offset(source, art_content_offset),
            Some(BlockType::Art(_))
        ));
    }
}
