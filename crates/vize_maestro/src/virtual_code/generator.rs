//! Virtual code generator that transforms SFC into virtual documents.
//!
//! Uses arena allocation from vize_carton for optimal performance.
#![allow(clippy::disallowed_methods)]

mod art_script;
mod binding;
mod block;
mod inline_art;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_semantic_bindings;

use vize_atelier_sfc::SfcDescriptor;
use vize_carton::Bump;
use vize_carton::cstr;

use binding::{template_used_croquis_bindings, template_used_script_bindings};

use super::{
    ScriptCodeGenerator, StyleCodeGenerator, TemplateCodeGenerator, VirtualDocument,
    VirtualDocuments, VirtualLanguage,
    script_code::extract_simple_bindings,
    template_code::{TemplateExpression, extract_expressions},
};

pub use art_script::{
    ArtScriptChunk, ArtScriptSetupParts, ArtTargetComponent, analyze_art_script_setup,
    art_target_component_from_source, find_define_art_component_name,
    find_define_art_target_component,
};
pub(crate) use block::find_art_block_at_offset_with_descriptor;
pub use block::{
    ArtCursorPosition, ArtVariantInfo, BlockType, find_art_block_at_offset, find_block_at_offset,
};
pub(crate) use inline_art::inline_art_variants;

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
        let allocator = Bump::new();
        let root = descriptor
            .template
            .as_ref()
            .map(|template| vize_armature::parse(&allocator, template.content.as_ref()).0);
        self.generate_from_root(descriptor, base_uri, root.as_ref(), None)
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
        let root = descriptor
            .template
            .as_ref()
            .map(|template| vize_armature::parse(allocator, template.content.as_ref()).0);
        self.generate_from_root(descriptor, base_uri, root.as_ref(), None)
    }

    /// Generate from Atlas-owned Relief syntax without parsing template text.
    pub(crate) fn generate_from_snapshot<'a>(
        &mut self,
        descriptor: &SfcDescriptor<'a>,
        base_uri: &str,
        syntax: Option<&vize_relief::ReliefSnapshot>,
        croquis: &vize_croquis::Croquis,
    ) -> VirtualDocuments {
        let allocator = Bump::new();
        let root = syntax.map(|syntax| syntax.materialize(&allocator));
        self.generate_from_root(descriptor, base_uri, root.as_ref(), Some(croquis))
    }

    fn generate_from_root<'a>(
        &mut self,
        descriptor: &SfcDescriptor<'a>,
        base_uri: &str,
        root: Option<&vize_relief::RootNode<'_>>,
        croquis: Option<&vize_croquis::Croquis>,
    ) -> VirtualDocuments {
        let mut docs = VirtualDocuments::new();
        let mut template_expressions = Vec::new();
        if let (Some(template), Some(root)) = (descriptor.template.as_ref(), root) {
            template_expressions = extract_expressions(root);
            self.template_gen
                .set_block_offset(template.loc.start as u32);
            let mut template_doc = self.template_gen.generate(root, template.content.as_ref());
            template_doc.uri = cstr!("{base_uri}.__template.ts").to_string();
            docs.template = Some(template_doc);
        }

        if let Some(ref script) = descriptor.script {
            let mut script_doc = self.script_gen.generate(script, false);
            script_doc.uri = cstr!("{base_uri}.__script.ts").to_string();
            docs.script = Some(script_doc);
        }

        if let Some(ref script_setup) = descriptor.script_setup {
            let template_bindings = croquis.map_or_else(
                || {
                    template_used_script_bindings(
                        script_setup.content.as_ref(),
                        &template_expressions,
                    )
                },
                template_used_croquis_bindings,
            );
            let mut script_doc =
                self.script_gen
                    .generate_with_exports(script_setup, true, &template_bindings);
            script_doc.uri = cstr!("{base_uri}.__script_setup.ts").to_string();
            docs.script_setup = Some(script_doc);
        }

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
