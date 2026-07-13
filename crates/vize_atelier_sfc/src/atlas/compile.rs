//! Production SFC compilation from shared Atlas artifacts.

#[path = "compile/backend.rs"]
mod backend;

pub use backend::{
    SfcRenderModuleArtifact, SfcRenderModuleProduct, SfcRenderModuleProvider, SfcRenderTarget,
};

use vize_atlas::{
    Compilation, CompilationInputError, PlanningContext, Product, ProductId, Provider,
    ProviderContext, ProviderError, SourceId, SourceInput, SourceInputId,
};
use vize_carton::FxHashMap;
use vize_carton::cstr;
use vize_relief::TemplateSyntaxMode;
use vize_relief::TransformedReliefProduct;

use crate::compile::{GraphRenderModule, compile_sfc_with_graph_render};
use crate::{SfcCompileOptions, SfcCompileResult};

use super::{SfcDescriptorProduct, is_sfc_source, usable_descriptor};

/// Complete output-affecting request for one SFC source.
#[derive(Debug, Clone, Default)]
pub struct SfcCompileRequest {
    /// Complete public compiler options for this source.
    pub options: SfcCompileOptions,
    /// Parser compatibility mode for this source's template.
    pub template_syntax: TemplateSyntaxMode,
    /// Derive template/style scoped flags from the parsed descriptor before compilation.
    pub infer_scoped_from_descriptor: bool,
}

impl SfcCompileRequest {
    /// Create a request without descriptor-derived option normalization.
    pub fn new(options: SfcCompileOptions, template_syntax: TemplateSyntaxMode) -> Self {
        Self {
            options,
            template_syntax,
            infer_scoped_from_descriptor: false,
        }
    }

    /// Derive both template and style scoped flags from the cached descriptor.
    pub fn with_inferred_scoped_from_descriptor(mut self) -> Self {
        self.infer_scoped_from_descriptor = true;
        self
    }
}

/// Source-aware settings for a multi-file compilation.
///
/// Each request is installed as a source input. Updating one file therefore
/// invalidates only that source's dependent products in a persistent batch.
#[derive(Debug, Clone, Default)]
pub struct SfcCompileSettings {
    default: SfcCompileRequest,
    sources: FxHashMap<SourceId, SfcCompileRequest>,
}

impl SfcCompileSettings {
    /// Create settings with one fallback request for sources without overrides.
    pub fn new(default: SfcCompileRequest) -> Self {
        Self {
            default,
            sources: FxHashMap::default(),
        }
    }

    /// Replace the fallback request.
    pub fn set_default(&mut self, request: SfcCompileRequest) {
        self.default = request;
    }

    /// Install or replace one source-specific request.
    pub fn insert(&mut self, source: SourceId, request: SfcCompileRequest) {
        self.sources.insert(source, request);
    }

    /// Resolve a source-specific request or the fallback request.
    pub fn get(&self, source: SourceId) -> &SfcCompileRequest {
        self.sources.get(&source).unwrap_or(&self.default)
    }

    /// Install all source overrides without globally invalidating other files.
    pub fn install(&self, compilation: &mut Compilation) -> Result<(), CompilationInputError> {
        for (source, request) in &self.sources {
            compilation.set_source_input::<SfcCompileSettingsInput>(*source, request.clone())?;
        }
        Ok(())
    }
}

/// Typed Atlas input carrying every public SFC compile option and syntax mode.
pub struct SfcCompileSettingsInput;

impl SourceInput for SfcCompileSettingsInput {
    type Value = SfcCompileRequest;

    const NAME: &'static str = "sfc.compile-settings";
}

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
        if context.source().text().contains("<template") {
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
        compile_sfc_with_graph_render(descriptor, request.options, render, warnings)
            .map_err(|error| ProviderError::message(error.message))
    }
}

pub(super) fn planning_request(context: &PlanningContext<'_>) -> SfcCompileRequest {
    context
        .source_input::<SfcCompileSettingsInput>()
        .cloned()
        .unwrap_or_default()
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
