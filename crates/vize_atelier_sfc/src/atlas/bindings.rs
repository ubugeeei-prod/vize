//! Neutral template-visible bindings projected from parse-once SFC script facts.

use vize_atlas::{
    PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError, SourceInputId,
    SourceKindInput,
};
use vize_carton::BindingMetadata;

use crate::croquis::{SfcCroquisOptions, analyze_sfc_descriptor_with_script_analysis};

use super::croquis::{
    SfcCroquisSettingsInput, SfcInferredCroquisSettingsInput, croquis_request_for_provider,
};
use super::{
    SfcDescriptorProduct, SfcScriptSyntaxProduct, is_sfc_context, source_structure,
    usable_descriptor,
};

/// Script binding identities required by template transforms and render lowering.
///
/// This deliberately is not a complete Croquis document: build and render
/// consumers pay only for the script projection they need, while linting,
/// type checking, and editor features can independently request Croquis.
pub struct SfcTemplateBindingsProduct;

impl Product for SfcTemplateBindingsProduct {
    type Value = BindingMetadata;

    const NAME: &'static str = "sfc.template-bindings";
}

/// Produce template-visible bindings without parsing or traversing a template.
pub struct SfcTemplateBindingsProvider;

impl Provider for SfcTemplateBindingsProvider {
    type Product = SfcTemplateBindingsProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![
            SourceInputId::of::<SfcCroquisSettingsInput>(),
            SourceInputId::of::<SfcInferredCroquisSettingsInput>(),
            SourceInputId::of::<SourceKindInput>(),
        ]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_context(context) && source_structure(context).has_script
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![
            ProductId::of::<SfcDescriptorProduct>(),
            ProductId::of::<SfcScriptSyntaxProduct>(),
        ]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<BindingMetadata, ProviderError> {
        let descriptor = context.get::<SfcDescriptorProduct>()?;
        let descriptor = usable_descriptor(&descriptor)?;
        let syntax = context.get::<SfcScriptSyntaxProduct>()?;
        let request = croquis_request_for_provider(context);
        let analysis = analyze_sfc_descriptor_with_script_analysis(
            descriptor,
            None,
            SfcCroquisOptions::for_declaration(),
            matches!(request.mode, super::SfcCroquisMode::OptionsApi),
            matches!(request.mode, super::SfcCroquisMode::LegacyVue2),
            request.resolved_filename.as_deref(),
            false,
            syntax.croquis(true),
        )
        .croquis;
        Ok(BindingMetadata {
            bindings: analysis.bindings.bindings,
            props_aliases: analysis.bindings.props_aliases,
            is_script_setup: analysis.bindings.is_script_setup,
        })
    }
}
