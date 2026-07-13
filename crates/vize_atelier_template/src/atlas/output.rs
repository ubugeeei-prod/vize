use vize_atlas::{
    PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError, SourceInputId,
};
use vize_carton::String;
use vize_relief::{ReliefSnapshot, TransformedReliefProduct};
use vize_rendu::RenderEmitSettingsInput;

use super::{
    TemplateCompileSettingsInput, TemplateRenderTarget, providers::request_for,
    settings::is_raw_template_source, source_map::source_map,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TemplateOutputMapping {
    pub generated_start: usize,
    pub generated_end: usize,
    pub source_start: u32,
}

#[derive(Debug, Clone)]
pub struct TemplateCompileArtifact {
    pub code: String,
    pub preamble: String,
    pub map: Option<serde_json::Value>,
    pub templates: Option<Vec<String>>,
    pub syntax: ReliefSnapshot,
    pub mappings: Vec<TemplateOutputMapping>,
    pub target: TemplateRenderTarget,
}

pub struct TemplateCompileProduct;

impl Product for TemplateCompileProduct {
    type Value = TemplateCompileArtifact;

    const NAME: &'static str = "template.compiled-module";
}

pub struct TemplateCompileProvider;

impl Provider for TemplateCompileProvider {
    type Product = TemplateCompileProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![
            SourceInputId::of::<TemplateCompileSettingsInput>(),
            SourceInputId::of::<RenderEmitSettingsInput>(),
        ]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_raw_template_source(context.source().name())
            && context
                .source_input::<TemplateCompileSettingsInput>()
                .is_some()
            && context.source_input::<RenderEmitSettingsInput>().is_some()
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        let target = context
            .source_input::<TemplateCompileSettingsInput>()
            .map_or(TemplateRenderTarget::Dom, |request| request.target);
        vec![
            ProductId::of::<TransformedReliefProduct>(),
            backend_product(target),
        ]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<TemplateCompileArtifact, ProviderError> {
        let request = request_for(context);
        let (code, preamble, templates, mappings) = match request.target {
            TemplateRenderTarget::Dom => dom_output(context)?,
            TemplateRenderTarget::Ssr => ssr_output(context)?,
            TemplateRenderTarget::Vapor => vapor_output(context)?,
        };
        let transformed = context.get::<TransformedReliefProduct>()?;
        let syntax = transformed
            .as_ref()
            .as_ref()
            .ok_or_else(|| ProviderError::message("raw template Relief is absent"))?
            .snapshot()
            .clone();
        let filename = if request.transform.filename.is_empty() {
            context.source().name()
        } else {
            request.transform.filename.as_str()
        };
        let map = request
            .source_map
            .then(|| source_map(filename, context.source().text(), code.as_str(), &mappings));
        Ok(TemplateCompileArtifact {
            code,
            preamble,
            map,
            templates,
            syntax,
            mappings,
            target: request.target,
        })
    }
}

type BackendOutput = (
    String,
    String,
    Option<Vec<String>>,
    Vec<TemplateOutputMapping>,
);

fn dom_output(context: &mut ProviderContext<'_>) -> Result<BackendOutput, ProviderError> {
    let output = context.get::<vize_atelier_dom::DomOutputProduct>()?;
    let offset = output.preamble.len();
    Ok((
        output.body.clone(),
        output.preamble.clone(),
        None,
        output
            .mappings
            .iter()
            .map(|mapping| TemplateOutputMapping {
                generated_start: mapping.generated_start.saturating_sub(offset),
                generated_end: mapping.generated_end.saturating_sub(offset),
                source_start: mapping.source.start.offset,
            })
            .collect(),
    ))
}

fn ssr_output(context: &mut ProviderContext<'_>) -> Result<BackendOutput, ProviderError> {
    let output = context.get::<vize_atelier_ssr::SsrOutputProduct>()?;
    let offset = output.preamble.len();
    Ok((
        output.body.clone(),
        output.preamble.clone(),
        None,
        output
            .mappings
            .iter()
            .map(|mapping| TemplateOutputMapping {
                generated_start: mapping.generated_start.saturating_sub(offset),
                generated_end: mapping.generated_end.saturating_sub(offset),
                source_start: mapping.source.start.offset,
            })
            .collect(),
    ))
}

fn vapor_output(context: &mut ProviderContext<'_>) -> Result<BackendOutput, ProviderError> {
    let output = context.get::<vize_atelier_vapor::VaporOutputProduct>()?;
    let offset = output.preamble.len();
    Ok((
        output.body.clone(),
        output.preamble.clone(),
        Some(output.templates.clone()),
        output
            .mappings
            .iter()
            .map(|mapping| TemplateOutputMapping {
                generated_start: mapping.generated_start.saturating_sub(offset),
                generated_end: mapping.generated_end.saturating_sub(offset),
                source_start: mapping.source.start.offset,
            })
            .collect(),
    ))
}

fn backend_product(target: TemplateRenderTarget) -> ProductId {
    match target {
        TemplateRenderTarget::Dom => ProductId::of::<vize_atelier_dom::DomOutputProduct>(),
        TemplateRenderTarget::Ssr => ProductId::of::<vize_atelier_ssr::SsrOutputProduct>(),
        TemplateRenderTarget::Vapor => ProductId::of::<vize_atelier_vapor::VaporOutputProduct>(),
    }
}
