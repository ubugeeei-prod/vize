//! Virtual code generator that transforms SFC into virtual documents.
//!
//! Uses arena allocation from vize_l0 for optimal performance.
#![expect(
    clippy::disallowed_methods,
    reason = "VirtualDocument fields store std String"
)]

mod art_script;
mod binding;
mod block;
mod checker_document;
mod inline_art;
pub(crate) mod occurrences;

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests;
#[cfg(test)]
#[expect(
    clippy::disallowed_macros,
    reason = "`insta` snapshot macros expand through `std::format!`; see CONTRIBUTING.md, \"Snapshot assertions in test targets\""
)]
mod tests_semantic_bindings;

use vize_atelier_sfc::SfcDescriptor;
use vize_l0::Allocator;
use vize_l0::cstr;

use binding::{template_used_script_bindings, template_used_script_bindings_demand};

use super::{
    ScriptCodeGenerator, StyleCodeGenerator, VirtualDocument, VirtualDocuments, VirtualLanguage,
    template_code::{TemplateExpression, extract_expressions},
};

pub use art_script::{
    ArtScriptChunk, ArtScriptSetupParts, ArtTargetComponent, analyze_art_script_setup,
    art_target_component_from_source, find_define_art_component_name,
    find_define_art_target_component,
};
pub use block::{
    ArtCursorPosition, ArtVariantInfo, BlockType, find_art_block_at_offset, find_block_at_offset,
};
pub(crate) use block::{find_art_block_at_completion_offset, find_block_at_completion_offset};
pub(crate) use inline_art::inline_art_variants;

/// Virtual code generator for SFC files.
///
/// This generator transforms Vue SFC files into virtual documents for each
/// embedded language (template, script, style). It uses arena allocation
/// for temporary parsing data to minimize allocations.
pub struct VirtualCodeGenerator {
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
        let allocator = Allocator::new();
        self.generate_with_allocator(descriptor, base_uri, &allocator)
    }

    /// Carry authored editor facts out of the existing checker analysis.
    pub(crate) fn generate_with_occurrences(
        &mut self,
        descriptor: &SfcDescriptor<'_>,
        base_uri: &str,
        source: &str,
    ) -> (VirtualDocuments, Option<occurrences::PhysicalOccurrences>) {
        let allocator = Allocator::new();
        self.generate_demand(descriptor, base_uri, &allocator, Some(source))
    }

    /// Generate virtual documents with explicit allocator.
    ///
    /// Use this when you want to control the allocator lifetime,
    /// for example when processing multiple files in a batch.
    pub fn generate_with_allocator<'a, 'alloc>(
        &mut self,
        descriptor: &SfcDescriptor<'a>,
        base_uri: &str,
        allocator: &'alloc Allocator,
    ) -> VirtualDocuments {
        self.generate_demand(descriptor, base_uri, allocator, None)
            .0
    }

    fn generate_demand(
        &mut self,
        descriptor: &SfcDescriptor<'_>,
        base_uri: &str,
        allocator: &Allocator,
        source: Option<&str>,
    ) -> (VirtualDocuments, Option<occurrences::PhysicalOccurrences>) {
        let source = source.filter(|source| {
            descriptor.source.as_ref() == *source
                && occurrences::PhysicalOccurrences::supports_sfc_descriptor(descriptor)
        });
        let mut packet = None;
        let mut docs = VirtualDocuments::new();

        // Generate template virtual code
        let mut template_expressions = Vec::new();
        if let Some(ref template) = descriptor.template {
            let template_content = template.content.as_ref();

            // Parse template with provided allocator
            let (ast, errors) = vize_armature::parse(allocator, template_content);
            template_expressions = extract_expressions(&ast);
            let (document, facts) = checker_document::template_document(
                descriptor,
                &ast,
                template.loc.start as u32,
                base_uri,
                source.filter(|_| errors.is_empty()),
            );
            docs.template = Some(document);
            packet = facts;
        }

        // Generate script virtual code
        if let Some(ref script) = descriptor.script {
            let mut script_doc = self.script_gen.generate(script, false);
            script_doc.uri = cstr!("{base_uri}.__script.ts").to_string();
            docs.script = Some(script_doc);
        }

        // Generate script setup virtual code
        if let Some(ref script_setup) = descriptor.script_setup {
            let capture_script = source.is_some() && descriptor.template.is_none();
            let (template_bindings, script_packet) = if capture_script {
                template_used_script_bindings_demand(
                    script_setup.content.as_ref(),
                    &template_expressions,
                    true,
                )
            } else {
                (
                    template_used_script_bindings(
                        script_setup.content.as_ref(),
                        &template_expressions,
                    ),
                    None,
                )
            };
            if capture_script {
                packet = source.zip(script_packet).and_then(|(source, packet)| {
                    occurrences::PhysicalOccurrences::from_fragment(
                        packet,
                        script_setup.loc.start as u32,
                        0,
                        source,
                    )
                });
            }
            let mut script_doc =
                self.script_gen
                    .generate_with_exports(script_setup, true, &template_bindings);
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

        (docs, packet)
    }

    /// Quick generation for a single template string.
    ///
    /// Useful for testing and single-file scenarios.
    #[inline]
    pub fn generate_template_only(&mut self, template_content: &str) -> Option<VirtualDocument> {
        let allocator = Allocator::new();
        let (ast, _) = vize_armature::parse(&allocator, template_content);
        Some(checker_document::fragment_document(
            None,
            false,
            0,
            &ast,
            0,
            "__inline.__template.ts".to_string(),
        ))
    }
}

pub(crate) fn project_template_fragment(
    script: Option<&str>,
    script_setup: bool,
    script_offset: u32,
    root: &vize_relief::RootNode<'_>,
    template_offset: u32,
    uri: String,
) -> VirtualDocument {
    checker_document::fragment_document(
        script,
        script_setup,
        script_offset,
        root,
        template_offset,
        uri,
    )
}

pub(crate) fn project_template_fragment_with_occurrences(
    script: Option<&str>,
    script_setup: bool,
    script_offset: u32,
    root: &vize_relief::RootNode<'_>,
    template_offset: u32,
    uri: String,
    source: Option<&str>,
) -> (VirtualDocument, Option<occurrences::PhysicalOccurrences>) {
    checker_document::fragment_document_demand(
        script,
        script_setup,
        script_offset,
        root,
        template_offset,
        uri,
        source,
    )
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
    allocator: Allocator,
}

impl BatchVirtualCodeGenerator {
    /// Create a new batch generator.
    #[inline]
    pub fn new() -> Self {
        Self {
            generator: VirtualCodeGenerator::new(),
            allocator: Allocator::new(),
        }
    }

    /// Create with pre-allocated capacity.
    ///
    /// Use this when you know approximately how much memory will be needed.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            generator: VirtualCodeGenerator::new(),
            allocator: Allocator::with_capacity(capacity),
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
