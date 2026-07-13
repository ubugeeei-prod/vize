//! Production SFC compilation from shared Atlas artifacts.

#[path = "compile/backend.rs"]
mod backend;
#[path = "compile/settings.rs"]
mod settings;

pub use backend::{
    SfcRenderModuleArtifact, SfcRenderModuleProduct, SfcRenderModuleProvider, SfcRenderTarget,
};
pub use settings::{
    SfcCompileRequest, SfcCompileSettings, SfcCompileSettingsInput, SfcParseSettingsInput,
    SfcRenderRequest, SfcRenderSettingsInput, SfcTemplateFrontendRequest,
    SfcTemplateFrontendSettingsInput, install_sfc_compile_request,
};

use vize_atlas::{
    PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError, SourceInputId,
};
use vize_carton::cstr;
use vize_module::ModuleSyntaxProduct;
use vize_relief::TransformedReliefProduct;

use crate::SfcCompileResult;
use crate::compile::{GraphRenderModule, compile_sfc_with_graph_render};

use super::{
    SfcDescriptorProduct, SfcScriptSyntaxProduct, is_sfc_source, source_structure,
    usable_descriptor,
};

/// Complete compiled JavaScript, CSS, maps, diagnostics, and macro artifacts.
pub struct SfcCompileProduct;

impl Product for SfcCompileProduct {
    type Value = SfcCompileResult;

    const NAME: &'static str = "sfc.compiled-module";
}

/// Assemble an SFC module from its shared container and typed render product.
pub struct SfcCompileProvider;

impl Provider for SfcCompileProvider {
    type Product = SfcCompileProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<SfcCompileSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        let mut dependencies = vec![ProductId::of::<SfcDescriptorProduct>()];
        // Planning cannot execute the descriptor product: classify the source
        // once with the allocation-light container scanner, then declare the
        // complete closure before any provider runs.
        let structure = source_structure(context);
        if structure.has_script {
            dependencies.push(ProductId::of::<SfcScriptSyntaxProduct>());
            dependencies.push(ProductId::of::<ModuleSyntaxProduct>());
        }
        if structure.has_template {
            dependencies.push(ProductId::of::<TransformedReliefProduct>());
            dependencies.push(ProductId::of::<SfcRenderModuleProduct>());
        }
        dependencies
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<SfcCompileResult, ProviderError> {
        let mut request = request_for(context);
        let artifact = context.get::<SfcDescriptorProduct>()?;
        let descriptor = usable_descriptor(&artifact)?;
        apply_descriptor_inference(&mut request, descriptor);
        let (render, warnings) = if descriptor.template.is_some() {
            let transformed = context.get::<TransformedReliefProduct>()?;
            let warnings = transformed
                .as_ref()
                .as_ref()
                .map(recoverable_warnings)
                .unwrap_or_default();
            let render = context.get::<SfcRenderModuleProduct>()?;
            (render.render.clone(), warnings)
        } else {
            (
                GraphRenderModule {
                    code: Default::default(),
                    templates: None,
                    mappings: Vec::new(),
                    render: None,
                    vapor: request.options.vapor,
                },
                Vec::new(),
            )
        };
        let (script_syntax, modules) =
            if descriptor.script.is_some() || descriptor.script_setup.is_some() {
                (
                    Some(context.get::<SfcScriptSyntaxProduct>()?),
                    Some(context.get::<ModuleSyntaxProduct>()?),
                )
            } else {
                (None, None)
            };
        compile_sfc_with_graph_render(
            descriptor,
            request.options,
            &request.render_emit,
            render,
            warnings,
            modules.as_deref(),
            script_syntax.as_deref(),
        )
        .map_err(|error| ProviderError::message(error.message))
    }
}

fn recoverable_warnings(artifact: &vize_relief::TransformedReliefArtifact) -> Vec<crate::SfcError> {
    artifact
        .parse_diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.is_recoverable())
        .map(|diagnostic| crate::SfcError {
            message: diagnostic.message.clone(),
            code: Some(cstr!("{:?}", diagnostic.code)),
            loc: None,
        })
        .collect()
}

fn apply_descriptor_inference(request: &mut SfcCompileRequest, descriptor: &crate::SfcDescriptor) {
    if request.infer_scoped_from_descriptor {
        let scoped = descriptor.styles.iter().any(|style| style.scoped);
        request.options.template.scoped = scoped;
        request.options.style.scoped = scoped;
    }
}

/// Final composed source map projected from the complete SFC module.
pub struct SfcSourceMapProduct;

impl Product for SfcSourceMapProduct {
    type Value = Option<serde_json::Value>;

    const NAME: &'static str = "sfc.source-map";
}

/// Expose source maps as an independently demandable Atlas product.
pub struct SfcSourceMapProvider;

impl Provider for SfcSourceMapProvider {
    type Product = SfcSourceMapProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<SfcCompileProduct>()]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<Option<serde_json::Value>, ProviderError> {
        Ok(context.get::<SfcCompileProduct>()?.map.clone())
    }
}

pub(super) fn request_for(context: &ProviderContext<'_>) -> SfcCompileRequest {
    let mut request = context
        .source_input::<SfcCompileSettingsInput>()
        .cloned()
        .unwrap_or_default();
    if request.options.parse.filename.is_empty() {
        request.options.parse.filename = context.source().name().into();
    }
    request
}

#[cfg(test)]
#[path = "compile/tests.rs"]
mod tests;
