//! Independently registered Atlas providers for Vue SFC sources.

#[path = "atlas/bindings.rs"]
mod bindings;
#[path = "atlas/container.rs"]
mod container;
#[path = "atlas/croquis.rs"]
mod croquis;
#[path = "atlas/compile.rs"]
mod full_compile;
#[path = "atlas/relief.rs"]
mod relief;
#[path = "atlas/script_generator.rs"]
mod script_generator;
#[path = "atlas/module.rs"]
mod script_module;
#[path = "atlas/structure.rs"]
mod structure;

use vize_atlas::{
    Compilation, PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError,
    RegisterProviderError, SourceInputId, SourceKindInput,
};
use vize_carton::{
    cstr, source_anchor::SourceAnchor, source_range::SourceRange as StableSourceRange,
};
use vize_flow::{FlowGraph, FlowProduct};
use vize_module::{ModuleSyntaxProduct, append_module_flow};
use vize_relief::{ReliefSnapshot, TransformedReliefArtifact, TransformedReliefProduct};
use vize_rendu::{RenduBuilder, RenduModule, RenduProduct};

use crate::graph_frontend::{
    lower_relief_snapshot_to_rendu_with_anchor,
    lower_relief_snapshot_to_rendu_with_anchor_and_bindings,
    project_relief_snapshot_to_flow_with_anchor,
};
use crate::{SfcDescriptor, SfcGraphAdapterError};
pub use bindings::{SfcTemplateBindingsProduct, SfcTemplateBindingsProvider};
pub use container::{
    SfcDescriptorArtifact, SfcDescriptorProduct, SfcDescriptorProvider, SfcTemplateProduct,
    SfcTemplateProvider, SfcTemplateSource,
};
pub use croquis::{
    SfcCroquisMode, SfcCroquisProvider, SfcCroquisRequest, SfcCroquisSettings,
    SfcCroquisSettingsInput, SfcResolvedPropsPolicy,
};
pub use full_compile::{
    SfcCompileProduct, SfcCompileProvider, SfcCompileRequest, SfcCompileSettings,
    SfcCompileSettingsInput, SfcParseSettingsInput, SfcRenderModuleArtifact,
    SfcRenderModuleProduct, SfcRenderModuleProvider, SfcRenderRequest, SfcRenderSettingsInput,
    SfcRenderTarget, SfcSourceMapProduct, SfcSourceMapProvider, SfcTemplateFrontendRequest,
    SfcTemplateFrontendSettingsInput, install_sfc_compile_request,
};
pub use relief::{SfcReliefProvider, SfcTransformedReliefProvider};
pub use script_generator::{
    ScriptDefaultExportTargets, ScriptOptionsApiBridge, ScriptOptionsApiPropsSource,
    ScriptOptionsFunction, ScriptOptionsFunctionKind, SfcScriptGeneratorFacts,
};
pub use script_module::{
    SfcModuleSyntaxProvider, SfcScriptSyntaxProduct, SfcScriptSyntaxProvider,
    SfcScriptSyntaxSnapshot, authored_script_parse_invocations,
    reset_authored_script_parse_invocations,
};
use structure::source_structure;

/// Open Atlas source-kind value owned by the SFC frontend.
pub const SFC_SOURCE_KIND: &str = "vue-sfc";

/// Relief syntax to frontend-neutral Rendu for SFC sources.
pub struct SfcRenduProvider;

impl Provider for SfcRenduProvider {
    type Product = RenduProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![
            SourceInputId::of::<full_compile::SfcRenderScopeSettingsInput>(),
            SourceInputId::of::<SourceKindInput>(),
        ]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_context(context)
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        let mut dependencies = vec![
            ProductId::of::<SfcDescriptorProduct>(),
            ProductId::of::<SfcTemplateProduct>(),
            ProductId::of::<TransformedReliefProduct>(),
        ];
        if source_structure(context).has_script {
            dependencies.push(ProductId::of::<SfcTemplateBindingsProduct>());
        }
        dependencies
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<RenduProduct as Product>::Value, ProviderError> {
        let descriptor = context.get::<SfcDescriptorProduct>()?;
        let template = context.get::<SfcTemplateProduct>()?;
        let relief = context.get::<TransformedReliefProduct>()?;
        let (template, relief) = match (template.as_ref(), relief.as_ref()) {
            (Some(template), Some(relief)) => (template, relief),
            (None, None) => {
                return RenduBuilder::new()
                    .finish()
                    .map(RenduModule::from_root)
                    .map_err(|error| ProviderError::message(cstr!("{error}")));
            }
            _ => return Err(inconsistent_template_artifacts()),
        };
        let relief = usable_relief_snapshot(relief)?;
        let anchor = template_source_anchor(template)?;
        let has_script = sfc_source_structure(context.source().text()).has_script;
        let bindings = if has_script {
            Some(context.get::<SfcTemplateBindingsProduct>()?)
        } else {
            None
        };
        let rendu = if let Some(bindings) = bindings.as_ref() {
            lower_relief_snapshot_to_rendu_with_anchor_and_bindings(relief, anchor, bindings)
        } else {
            lower_relief_snapshot_to_rendu_with_anchor(relief, anchor)
        };
        let mut rendu = rendu.map_err(graph_error)?;
        if usable_descriptor(&descriptor)?
            .styles
            .iter()
            .any(|style| style.scoped)
        {
            let scope_id = context
                .source_input::<full_compile::SfcRenderScopeSettingsInput>()
                .map(|request| request.scope_id.clone())
                .unwrap_or_else(|| crate::compile::generate_scope_id(context.source().name()));
            rendu = rendu.with_component_scope_id(cstr!("data-v-{scope_id}"));
        }
        Ok(RenduModule::from_root(rendu))
    }
}

/// Relief syntax to the separate single-file Flow representation.
pub struct SfcFlowProvider;

impl Provider for SfcFlowProvider {
    type Product = FlowProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<SourceKindInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_context(context)
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        let mut dependencies = vec![
            ProductId::of::<SfcTemplateProduct>(),
            ProductId::of::<TransformedReliefProduct>(),
        ];
        if source_structure(context).has_script {
            dependencies.push(ProductId::of::<ModuleSyntaxProduct>());
        }
        dependencies
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<FlowProduct as Product>::Value, ProviderError> {
        let template = context.get::<SfcTemplateProduct>()?;
        let relief = context.get::<TransformedReliefProduct>()?;
        let mut graph = match (template.as_ref(), relief.as_ref()) {
            (Some(template), Some(relief)) => {
                let anchor = template_source_anchor(template)?;
                project_relief_snapshot_to_flow_with_anchor(relief.snapshot(), anchor)
                    .map_err(graph_error)?
            }
            (None, None) => FlowGraph::new(),
            _ => return Err(inconsistent_template_artifacts()),
        };
        // Flow is a diagnostic/control-analysis product and must remain
        // available for recoverable or malformed sources. Strict render
        // backends validate these diagnostics at the Rendu boundary instead.
        if sfc_source_has_script(context.source().text()) {
            let modules = context.get::<ModuleSyntaxProduct>()?;
            append_module_flow(&modules, &mut graph)
                .map_err(|error| ProviderError::message(cstr!("{error}")))?;
        }
        Ok(graph)
    }
}

/// Register only the SFC frontend's independently applicable providers.
pub fn register_atlas_providers(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(SfcDescriptorProvider)?;
    compilation.register_provider(SfcScriptSyntaxProvider)?;
    compilation.register_provider(SfcTemplateBindingsProvider)?;
    compilation.register_provider(SfcModuleSyntaxProvider)?;
    compilation.register_provider(SfcTemplateProvider)?;
    compilation.register_provider(SfcReliefProvider)?;
    compilation.register_provider(SfcTransformedReliefProvider)?;
    compilation.register_provider(SfcRenduProvider)?;
    compilation.register_provider(SfcFlowProvider)?;
    compilation.register_provider(SfcCroquisProvider)?;
    compilation.register_provider(SfcRenderModuleProvider)?;
    compilation.register_provider(SfcCompileProvider)?;
    compilation.register_provider(SfcSourceMapProvider)?;
    Ok(())
}

fn is_sfc_source(name: &str) -> bool {
    source_name_has_extension(name, ".vue")
}

/// Select the SFC frontend by physical suffix or an explicit parse request.
///
/// Vite and other hosts can feed virtual SFC bytes under their original module
/// identity instead of fabricating a `.vue` filename and corrupting source-map,
/// scope-id, and cache provenance.
fn is_sfc_context(context: &PlanningContext<'_>) -> bool {
    context.source_input::<SourceKindInput>().map_or_else(
        || {
            is_sfc_source(context.source().name())
                || context.source_input::<SfcParseSettingsInput>().is_some()
        },
        |kind| kind.is(SFC_SOURCE_KIND),
    )
}

fn source_name_has_extension(name: &str, extension: &str) -> bool {
    name.ends_with(extension)
        || name.char_indices().any(|(index, character)| {
            matches!(character, '?' | '#') && name[..index].ends_with(extension)
        })
}

/// Return whether a Vue SFC contains an authored script block without running
/// the descriptor parser. Product providers use this to keep template-only
/// plans free of JavaScript/TypeScript module work.
pub fn sfc_source_has_script(source: &str) -> bool {
    sfc_source_structure(source).has_script
}

/// Return whether a Vue SFC contains an authored template block without
/// running the descriptor parser.
pub fn sfc_source_has_template(source: &str) -> bool {
    sfc_source_structure(source).has_template
}

/// Classify authored SFC blocks with the same boundary-aware scanner used by
/// Atlas dependency planning. This does not construct an SFC descriptor.
pub fn sfc_source_structure(source: &str) -> crate::parse::SfcSourceStructure {
    crate::parse::scan_sfc_structure(source).unwrap_or_default()
}

pub use crate::parse::SfcSourceStructure;

fn usable_descriptor(
    artifact: &SfcDescriptorArtifact,
) -> Result<&SfcDescriptor<'static>, ProviderError> {
    artifact
        .as_result()
        .map_err(|error| ProviderError::message(error.message.clone()))
}

fn graph_error(error: SfcGraphAdapterError) -> ProviderError {
    ProviderError::message(cstr!("{error}"))
}

fn inconsistent_template_artifacts() -> ProviderError {
    ProviderError::message("SFC template products disagree about template presence")
}

fn usable_relief_snapshot(
    relief: &TransformedReliefArtifact,
) -> Result<&ReliefSnapshot, ProviderError> {
    if let Some(error) = relief
        .parse_diagnostics()
        .iter()
        .find(|error| !error.is_recoverable())
    {
        return Err(ProviderError::message(cstr!("{error:?}")));
    }
    if let Some(error) = relief.transform_diagnostics().first() {
        return Err(ProviderError::message(cstr!("{error:?}")));
    }
    Ok(relief.snapshot())
}

fn template_source_anchor(template: &SfcTemplateSource) -> Result<SourceAnchor, ProviderError> {
    let start = u32::try_from(template.range.start)
        .map_err(|_| ProviderError::message("SFC template start exceeds u32 source space"))?;
    let end = u32::try_from(template.range.end)
        .map_err(|_| ProviderError::message("SFC template end exceeds u32 source space"))?;
    Ok(
        SourceAnchor::new(template.parent.get(), template.parent_revision.get())
            .with_parent_range(StableSourceRange::new(start, end)),
    )
}
