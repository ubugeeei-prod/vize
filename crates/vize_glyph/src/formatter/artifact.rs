//! Demandable Atlas product for a complete formatted SFC.

use vize_atelier_sfc::{SfcDescriptorProduct, register_atlas_providers};
use vize_atlas::{
    Compilation, PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError,
    QueryOutcome, RegisterProviderError, SourceId, SourceInput, SourceInputId,
};
use vize_carton::{Allocator, cstr};

use super::{FormatResult, GlyphFormatter};
use crate::{FormatError, FormatOptions};

/// Per-source formatter request. Changing one file's options does not evict
/// formatting products for unrelated sources.
pub struct GlyphFormatSettingsInput;

impl SourceInput for GlyphFormatSettingsInput {
    type Value = FormatOptions;

    const NAME: &'static str = "glyph.format.settings";
}

/// Complete formatted output owned by Glyph rather than the SFC frontend.
pub struct GlyphFormatProduct;

impl Product for GlyphFormatProduct {
    type Value = FormatResult;

    const NAME: &'static str = "glyph.format.output";
}

/// Formats the cached container descriptor without reparsing the SFC.
pub struct GlyphFormatProvider;

impl Provider for GlyphFormatProvider {
    type Product = GlyphFormatProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<GlyphFormatSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        context.source().name().ends_with(".vue")
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<SfcDescriptorProduct>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<FormatResult, ProviderError> {
        let artifact = context.get::<SfcDescriptorProduct>()?;
        let descriptor = artifact.descriptor().ok_or_else(|| {
            ProviderError::message(artifact.diagnostic().map_or_else(
                || cstr!("missing SFC descriptor"),
                |error| error.message.clone(),
            ))
        })?;
        let options = context
            .source_input::<GlyphFormatSettingsInput>()
            .cloned()
            .unwrap_or_default();
        let allocator = Allocator::with_capacity(context.source().text().len() * 2);
        GlyphFormatter::new(&options, &allocator)
            .format_descriptor_core(context.source().text(), descriptor)
            .map_err(|error| ProviderError::message(cstr!("{error}")))
    }
}

/// Register Glyph's output root and the frontend products it consumes.
pub fn register_glyph_atlas_provider(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    register_atlas_providers(compilation)?;
    register_glyph_format_provider(compilation)
}

/// Register only Glyph's formatter root in a compilation that already owns
/// the SFC frontend providers.
pub fn register_glyph_format_provider(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    if !compilation.has_provider::<GlyphFormatProduct>() {
        compilation.register_provider(GlyphFormatProvider)?;
    }
    Ok(())
}

pub(super) struct FormatterArtifactGraph {
    compilation: std::sync::Mutex<Compilation>,
    source: SourceId,
}

impl FormatterArtifactGraph {
    pub(super) fn new(source: &str, options: &FormatOptions) -> Result<Self, FormatError> {
        let mut compilation = Compilation::new();
        register_glyph_atlas_provider(&mut compilation)
            .map_err(|error| graph_error("register formatter providers", error))?;
        let source = compilation
            .add_source("format.vue", source)
            .map_err(|error| graph_error("add formatter source", error))?;
        compilation
            .set_source_input::<GlyphFormatSettingsInput>(source, options.clone())
            .map_err(|error| graph_error("set formatter options", error))?;
        Ok(Self {
            compilation: std::sync::Mutex::new(compilation),
            source,
        })
    }

    pub(super) fn query(&self) -> Result<QueryOutcome<GlyphFormatProduct>, FormatError> {
        self.compilation
            .lock()
            .map_err(|_| FormatError::ParseError(cstr!("Atlas formatter lock was poisoned")))?
            .query::<GlyphFormatProduct>(self.source)
            .map_err(|error| graph_error("query formatted SFC", error))
    }

    pub(super) fn format(&self) -> Result<FormatResult, FormatError> {
        Ok(self.query()?.value().clone())
    }
}

fn graph_error(action: &str, error: impl std::fmt::Display) -> FormatError {
    FormatError::ParseError(cstr!("Atlas failed to {action}: {error}"))
}

#[cfg(test)]
mod tests {
    use vize_atelier_sfc::SfcDescriptorProduct;
    use vize_atlas::{ProductStatus, SourceInputId};

    use super::*;

    #[test]
    fn format_output_is_the_root_and_reuses_the_descriptor() {
        let graph = FormatterArtifactGraph::new(
            "<script setup>const x=1</script><template><p>{{x}}</p></template>",
            &FormatOptions::default(),
        )
        .unwrap();
        let first = graph.query().unwrap();
        let second = graph.query().unwrap();

        assert_eq!(first.status(), ProductStatus::Executed);
        assert!(first.plan().contains::<GlyphFormatProduct>());
        assert!(first.plan().contains::<SfcDescriptorProduct>());
        assert!(first.trace().executed::<GlyphFormatProduct>());
        assert!(first.trace().executed::<SfcDescriptorProduct>());
        assert_eq!(second.status(), ProductStatus::CacheHit);
        assert!(second.trace().cache_hit::<GlyphFormatProduct>());
        assert!(
            first
                .plan()
                .source_input_revisions()
                .iter()
                .any(|(source, input, _)| {
                    *source == graph.source
                        && *input == SourceInputId::of::<GlyphFormatSettingsInput>()
                })
        );
    }

    #[test]
    fn descriptor_alone_does_not_execute_formatter_output() {
        let graph =
            FormatterArtifactGraph::new("<template><main /></template>", &Default::default())
                .unwrap();
        let outcome = graph
            .compilation
            .lock()
            .unwrap()
            .query::<SfcDescriptorProduct>(graph.source)
            .unwrap();

        assert!(!outcome.plan().contains::<GlyphFormatProduct>());
        assert!(!outcome.trace().executed::<GlyphFormatProduct>());
    }

    #[test]
    fn atlas_output_matches_the_descriptor_core() {
        let source = "<script setup>const x=1</script>\n<template><p>{{x}}</p></template>\n";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
        let direct = GlyphFormatter::new(&options, &allocator)
            .format_descriptor_core(source, &descriptor)
            .unwrap();
        let atlas = FormatterArtifactGraph::new(source, &options)
            .unwrap()
            .format()
            .unwrap();

        assert_eq!(atlas.code, direct.code);
        assert_eq!(atlas.changed, direct.changed);
    }
}
