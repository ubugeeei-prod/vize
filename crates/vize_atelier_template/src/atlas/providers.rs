use vize_armature::{parse_document_with_options, parse_with_options_and_template_syntax};
use vize_atelier_core::{transform, transform_with_template_syntax_quirks};
use vize_atlas::{
    InputId, PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError,
    SourceInputId,
};
use vize_carton::{Bump, cstr, source_anchor::SourceAnchor};
use vize_croquis::{
    CroquisDocument, CroquisDocumentProduct, CroquisSourceSegment, Drawer, DrawerOptions,
};
use vize_flow::{FlowGraph, FlowProduct};
use vize_relief::{
    ReliefArtifact, ReliefProduct, ReliefSnapshot, TransformedReliefArtifact,
    TransformedReliefProduct, VueDialectInput,
};
use vize_rendu::{RenduModule, RenduProduct};

use crate::graph_frontend::{
    TemplateGraphAdapterError, lower_relief_snapshot_to_rendu_with_anchor,
    project_relief_snapshot_to_flow_with_anchor,
};

use super::settings::{
    TemplateCompileRequest, TemplateCompileSettingsInput, TemplateParseMode, TemplateParseModeInput,
};

pub struct RawTemplateReliefProvider;

impl Provider for RawTemplateReliefProvider {
    type Product = ReliefProduct;

    fn input_dependencies(&self) -> Vec<InputId> {
        vec![InputId::of::<VueDialectInput>()]
    }

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![
            SourceInputId::of::<TemplateCompileSettingsInput>(),
            SourceInputId::of::<TemplateParseModeInput>(),
        ]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        applicable(context)
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<ReliefProduct as Product>::Value, ProviderError> {
        let request = request_for(context);
        let allocator = Bump::new();
        let (root, diagnostics) = match context
            .source_input::<TemplateParseModeInput>()
            .copied()
            .unwrap_or_default()
        {
            TemplateParseMode::Fragment => parse_with_options_and_template_syntax(
                &allocator,
                context.source().text(),
                request.parser,
                request.template_syntax,
            ),
            TemplateParseMode::Document => {
                parse_document_with_options(&allocator, context.source().text(), request.parser)
            }
        };
        Ok(Some(ReliefArtifact::new(
            ReliefSnapshot::from_root(&root),
            diagnostics.to_vec(),
        )))
    }
}

/// Complete semantic projection for raw template and standalone HTML sources.
pub struct RawTemplateCroquisProvider;

impl Provider for RawTemplateCroquisProvider {
    type Product = CroquisDocumentProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<TemplateCompileSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        applicable(context)
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<ReliefProduct>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<CroquisDocument, ProviderError> {
        let syntax = context.get::<ReliefProduct>()?;
        let syntax = syntax
            .as_ref()
            .as_ref()
            .ok_or_else(|| ProviderError::message("raw template Relief is absent"))?;
        let allocator = Bump::new();
        let root = syntax.snapshot().materialize(&allocator);
        let mut drawer = Drawer::with_options(DrawerOptions::full());
        drawer.draw_template(&root);
        let source = context.source();
        let anchor = SourceAnchor::new(source.id().get(), source.revision().get());
        Ok(CroquisDocument::new(drawer.finish())
            .with_source_anchor(anchor)
            .with_source(
                CroquisSourceSegment::new("template", source.text(), anchor).with_language("html"),
            ))
    }
}

pub struct RawTemplateTransformedReliefProvider;

impl Provider for RawTemplateTransformedReliefProvider {
    type Product = TransformedReliefProduct;

    fn input_dependencies(&self) -> Vec<InputId> {
        vec![InputId::of::<VueDialectInput>()]
    }

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<TemplateCompileSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        applicable(context)
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<ReliefProduct>()]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<TransformedReliefProduct as Product>::Value, ProviderError> {
        let request = request_for(context);
        let syntax = context.get::<ReliefProduct>()?;
        let syntax = syntax
            .as_ref()
            .as_ref()
            .ok_or_else(|| ProviderError::message("raw template Relief is absent"))?;
        let allocator = Bump::new();
        let mut root = syntax.snapshot().materialize(&allocator);
        let transformed = if request.template_syntax.is_quirks() {
            transform_with_template_syntax_quirks(&allocator, &mut root, request.transform, None)
        } else {
            transform(&allocator, &mut root, request.transform, None)
        };
        Ok(Some(TransformedReliefArtifact::new(
            ReliefSnapshot::from_root(&root),
            syntax.parse_diagnostics().to_vec(),
            transformed.errors,
        )))
    }
}

pub struct RawTemplateRenduProvider;

impl Provider for RawTemplateRenduProvider {
    type Product = RenduProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<TemplateCompileSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        applicable(context)
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<TransformedReliefProduct>()]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<<RenduProduct as Product>::Value, ProviderError> {
        let artifact = context.get::<TransformedReliefProduct>()?;
        let relief = usable_relief(artifact.as_ref().as_ref())?;
        let source = context.source();
        let anchor = SourceAnchor::new(source.id().get(), source.revision().get());
        lower_relief_snapshot_to_rendu_with_anchor(relief, anchor)
            .map(RenduModule::from_root)
            .map_err(graph_error)
    }
}

pub struct RawTemplateFlowProvider;

impl Provider for RawTemplateFlowProvider {
    type Product = FlowProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<TemplateCompileSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        applicable(context)
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<TransformedReliefProduct>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<FlowGraph, ProviderError> {
        let artifact = context.get::<TransformedReliefProduct>()?;
        let artifact = artifact
            .as_ref()
            .as_ref()
            .ok_or_else(|| ProviderError::message("raw template Relief is absent"))?;
        let relief = artifact.snapshot();
        let source = context.source();
        let anchor = SourceAnchor::new(source.id().get(), source.revision().get());
        project_relief_snapshot_to_flow_with_anchor(relief, anchor).map_err(graph_error)
    }
}

pub(super) fn request_for(context: &ProviderContext<'_>) -> TemplateCompileRequest {
    let mut request = context
        .source_input::<TemplateCompileSettingsInput>()
        .cloned()
        .unwrap_or_default();
    if let Some(dialect) = context.input::<VueDialectInput>().copied() {
        request.parser.dialect = dialect;
        request.transform.dialect = dialect;
    }
    request
}

fn applicable(context: &PlanningContext<'_>) -> bool {
    context
        .source_input::<TemplateCompileSettingsInput>()
        .is_some()
}

fn usable_relief(
    artifact: Option<&TransformedReliefArtifact>,
) -> Result<&ReliefSnapshot, ProviderError> {
    let artifact =
        artifact.ok_or_else(|| ProviderError::message("raw template Relief is absent"))?;
    if let Some(error) = artifact
        .parse_diagnostics()
        .iter()
        .find(|diagnostic| !diagnostic.is_recoverable())
    {
        return Err(ProviderError::message(cstr!("{error:?}")));
    }
    if let Some(error) = artifact.transform_diagnostics().first() {
        return Err(ProviderError::message(cstr!("{error:?}")));
    }
    Ok(artifact.snapshot())
}

fn graph_error(error: TemplateGraphAdapterError) -> ProviderError {
    ProviderError::message(cstr!("{error}"))
}
