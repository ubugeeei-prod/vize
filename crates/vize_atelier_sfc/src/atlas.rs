//! Independently registered Atlas providers for Vue SFC sources.

#[path = "atlas/croquis.rs"]
mod croquis;
#[path = "atlas/compile.rs"]
mod full_compile;
#[path = "atlas/relief.rs"]
mod relief;

use vize_atlas::{
    Compilation, ObservationKind, PlanningContext, Product, ProductId, Provider, ProviderContext,
    ProviderError, RegisterProviderError, SourceId, SourceInputId, SourceRange, SourceRevision,
};
use vize_carton::{
    String, cstr, source_anchor::SourceAnchor, source_range::SourceRange as StableSourceRange,
};
use vize_croquis::CroquisSemanticProduct;
use vize_flow::{FlowGraph, FlowProduct};
use vize_relief::{ReliefSnapshot, TransformedReliefArtifact, TransformedReliefProduct};
use vize_rendu::{RenduBuilder, RenduModule, RenduProduct};

use crate::graph_frontend::{
    lower_relief_snapshot_to_rendu_with_anchor, project_relief_snapshot_to_flow_with_anchor,
};
use crate::{SfcDescriptor, SfcError, SfcGraphAdapterError, parse_sfc};
pub use croquis::{
    SfcCroquisMode, SfcCroquisProvider, SfcCroquisRequest, SfcCroquisSettings,
    SfcCroquisSettingsInput, SfcResolvedPropsPolicy,
};
pub use full_compile::{
    SfcCompileProduct, SfcCompileProvider, SfcCompileRequest, SfcCompileSettings,
    SfcCompileSettingsInput, SfcRenderModuleArtifact, SfcRenderModuleProduct,
    SfcRenderModuleProvider, SfcRenderTarget, SfcSourceMapProduct, SfcSourceMapProvider,
};
pub use relief::{SfcReliefProvider, SfcTransformedReliefProvider};

/// Cached SFC container parse, including a structured fatal diagnostic.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SfcDescriptorArtifact {
    result: Result<SfcDescriptor<'static>, SfcError>,
}

impl SfcDescriptorArtifact {
    fn new(result: Result<SfcDescriptor<'static>, SfcError>) -> Self {
        Self { result }
    }

    /// Return the parsed descriptor when the SFC container is valid.
    pub fn descriptor(&self) -> Option<&SfcDescriptor<'static>> {
        self.result.as_ref().ok()
    }

    /// Return the cached fatal SFC container diagnostic, if any.
    pub fn diagnostic(&self) -> Option<&SfcError> {
        self.result.as_ref().err()
    }

    /// Borrow the complete parse result without discarding either state.
    pub fn as_result(&self) -> Result<&SfcDescriptor<'static>, &SfcError> {
        self.result.as_ref()
    }

    /// Consume the artifact and recover the complete parse result.
    pub fn into_result(self) -> Result<SfcDescriptor<'static>, SfcError> {
        self.result
    }
}

/// Parsed, owned SFC container descriptor or its fatal parse diagnostic.
pub struct SfcDescriptorProduct;

impl Product for SfcDescriptorProduct {
    type Value = SfcDescriptorArtifact;

    const NAME: &'static str = "sfc.descriptor";
}

/// Owned template block plus exact parent-source provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfcTemplateSource {
    pub parent: SourceId,
    pub parent_revision: SourceRevision,
    pub range: SourceRange,
    pub name: String,
    pub text: String,
}

/// SFC template block selected from a container descriptor.
pub struct SfcTemplateProduct;

impl Product for SfcTemplateProduct {
    type Value = Option<SfcTemplateSource>;

    const NAME: &'static str = "sfc.template-source";
}

/// Parse an applicable `.vue` source without constructing downstream products.
pub struct SfcDescriptorProvider;

impl Provider for SfcDescriptorProvider {
    type Product = SfcDescriptorProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<SfcCompileSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<SfcDescriptorArtifact, ProviderError> {
        let request = full_compile::request_for(context);
        let result = parse_sfc(context.source().text(), request.options.parse)
            .map(SfcDescriptor::into_owned);
        if let Err(error) = &result {
            context.observe(
                ObservationKind::Diagnostic,
                "sfc.parse.error",
                error.message.as_str(),
                error
                    .loc
                    .as_ref()
                    .map(|loc| SourceRange::new(loc.start, loc.end)),
            );
        }
        Ok(SfcDescriptorArtifact::new(result))
    }
}

/// Decompose the template block while retaining its parent identity/range.
pub struct SfcTemplateProvider;

impl Provider for SfcTemplateProvider {
    type Product = SfcTemplateProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<SfcDescriptorProduct>()]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<Option<SfcTemplateSource>, ProviderError> {
        let artifact = context.get::<SfcDescriptorProduct>()?;
        let Some(descriptor) = artifact.descriptor() else {
            return Ok(None);
        };
        let Some(template) = descriptor.template.as_ref() else {
            return Ok(None);
        };
        let source = context.source();
        Ok(Some(SfcTemplateSource {
            parent: source.id(),
            parent_revision: source.revision(),
            range: SourceRange::new(template.loc.start, template.loc.end),
            name: cstr!("{}#template", source.name()),
            text: template.content.as_ref().into(),
        }))
    }
}

/// Relief syntax to frontend-neutral Rendu for SFC sources.
pub struct SfcRenduProvider;

impl Provider for SfcRenduProvider {
    type Product = RenduProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![
            ProductId::of::<SfcTemplateProduct>(),
            ProductId::of::<TransformedReliefProduct>(),
        ]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<RenduProduct as Product>::Value, ProviderError> {
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
        lower_relief_snapshot_to_rendu_with_anchor(relief, anchor)
            .map(RenduModule::from_root)
            .map_err(graph_error)
    }
}

/// Relief syntax to the separate single-file Flow representation.
pub struct SfcFlowProvider;

impl Provider for SfcFlowProvider {
    type Product = FlowProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name())
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![
            ProductId::of::<SfcTemplateProduct>(),
            ProductId::of::<TransformedReliefProduct>(),
        ]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<FlowProduct as Product>::Value, ProviderError> {
        let template = context.get::<SfcTemplateProduct>()?;
        let relief = context.get::<TransformedReliefProduct>()?;
        let (template, relief) = match (template.as_ref(), relief.as_ref()) {
            (Some(template), Some(relief)) => (template, relief),
            (None, None) => return Ok(FlowGraph::new()),
            _ => return Err(inconsistent_template_artifacts()),
        };
        // Flow is a diagnostic/control-analysis product and must remain
        // available for recoverable or malformed sources. Strict render
        // backends validate these diagnostics at the Rendu boundary instead.
        let relief = relief.snapshot();
        let anchor = template_source_anchor(template)?;
        project_relief_snapshot_to_flow_with_anchor(relief, anchor).map_err(graph_error)
    }
}

/// Register the SFC frontend's independently applicable providers.
pub fn register_atlas_providers(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(SfcDescriptorProvider)?;
    compilation.register_provider(SfcTemplateProvider)?;
    compilation.register_provider(SfcReliefProvider)?;
    compilation.register_provider(SfcTransformedReliefProvider)?;
    compilation.register_provider(SfcRenduProvider)?;
    compilation.register_provider(SfcFlowProvider)?;
    compilation.register_provider(SfcCroquisProvider)?;
    vize_atelier_dom::register_atlas_provider(compilation)?;
    vize_atelier_ssr::register_atlas_provider(compilation)?;
    vize_atelier_vapor::register_atlas_provider(compilation)?;
    compilation.register_provider(SfcRenderModuleProvider)?;
    compilation.register_provider(SfcCompileProvider)?;
    compilation.register_provider(SfcSourceMapProvider)?;
    if !compilation.has_provider::<CroquisSemanticProduct>() {
        vize_croquis::register_semantic_projection(compilation)?;
    }
    Ok(())
}

fn is_sfc_source(name: &str) -> bool {
    name.ends_with(".vue")
}

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
