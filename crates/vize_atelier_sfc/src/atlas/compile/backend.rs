//! Typed target selection and backend output projection.

use vize_atlas::{
    PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError, SourceInputId,
};

use crate::compile::{GraphRenderMapping, GraphRenderModule, output_module::RenderFunctionName};

use super::super::{SfcDescriptorProduct, is_sfc_source, source_structure, usable_descriptor};
use super::{SfcRenderRequest, SfcRenderSettingsInput};

/// Selected graph backend for one SFC render module.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SfcRenderTarget {
    Dom,
    Ssr,
    Vapor,
}

/// Host-ready template output emitted by one independently registered backend.
#[derive(Debug, Clone)]
pub struct SfcRenderModuleArtifact {
    pub(super) render: GraphRenderModule,
    target: SfcRenderTarget,
}

impl SfcRenderModuleArtifact {
    pub fn code(&self) -> &str {
        self.render.code.as_str()
    }

    /// Static HTML templates materialized by the Vapor backend.
    pub fn templates(&self) -> Option<&[vize_carton::String]> {
        self.render.templates.as_deref()
    }

    pub const fn target(&self) -> SfcRenderTarget {
        self.target
    }
}

/// Selected target output before SFC script/style assembly.
pub struct SfcRenderModuleProduct;

impl Product for SfcRenderModuleProduct {
    type Value = SfcRenderModuleArtifact;

    const NAME: &'static str = "sfc.render-module";
}

/// Route one SFC Rendu module to its independently registered backend.
pub struct SfcRenderModuleProvider;

impl Provider for SfcRenderModuleProvider {
    type Product = SfcRenderModuleProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<SfcRenderSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_source(context.source().name()) && source_structure(context).has_template
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        let target = render_target(
            context
                .source_input::<SfcRenderSettingsInput>()
                .copied()
                .unwrap_or_default(),
            source_structure(context).vapor_script,
        );
        vec![
            ProductId::of::<SfcDescriptorProduct>(),
            backend_product(target),
        ]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<SfcRenderModuleArtifact, ProviderError> {
        let request = context
            .source_input::<SfcRenderSettingsInput>()
            .copied()
            .unwrap_or_default();
        let descriptor = context.get::<SfcDescriptorProduct>()?;
        let descriptor = usable_descriptor(&descriptor)?;
        let target = render_target_from_descriptor(request, descriptor);
        let render = match target {
            SfcRenderTarget::Dom => dom_render(context)?,
            SfcRenderTarget::Ssr => ssr_render(context)?,
            SfcRenderTarget::Vapor => vapor_render(context)?,
        };
        Ok(SfcRenderModuleArtifact { render, target })
    }
}

fn dom_render(context: &mut ProviderContext<'_>) -> Result<GraphRenderModule, ProviderError> {
    let output = context.get::<vize_atelier_dom::DomOutputProduct>()?;
    Ok(GraphRenderModule {
        templates: None,
        mappings: output
            .mappings
            .iter()
            .map(|mapping| GraphRenderMapping {
                generated_start: mapping.generated_start,
                source_start: mapping.source.start.offset,
            })
            .collect(),
        code: output.code.clone(),
        render: Some(RenderFunctionName::Render),
        vapor: false,
    })
}

fn ssr_render(context: &mut ProviderContext<'_>) -> Result<GraphRenderModule, ProviderError> {
    let output = context.get::<vize_atelier_ssr::SsrOutputProduct>()?;
    Ok(GraphRenderModule {
        templates: None,
        mappings: output
            .mappings
            .iter()
            .map(|mapping| GraphRenderMapping {
                generated_start: mapping.generated_start,
                source_start: mapping.source.start.offset,
            })
            .collect(),
        code: output.code.clone(),
        render: Some(RenderFunctionName::SsrRender),
        vapor: false,
    })
}

fn vapor_render(context: &mut ProviderContext<'_>) -> Result<GraphRenderModule, ProviderError> {
    let output = context.get::<vize_atelier_vapor::VaporOutputProduct>()?;
    Ok(GraphRenderModule {
        templates: Some(output.templates.clone()),
        mappings: output
            .mappings
            .iter()
            .map(|mapping| GraphRenderMapping {
                generated_start: mapping.generated_start,
                source_start: mapping.source.start.offset,
            })
            .collect(),
        code: output.code.clone(),
        render: Some(RenderFunctionName::Render),
        vapor: true,
    })
}

fn render_target(request: SfcRenderRequest, vapor_script: bool) -> SfcRenderTarget {
    if request.ssr {
        SfcRenderTarget::Ssr
    } else if request.vapor || vapor_script {
        SfcRenderTarget::Vapor
    } else {
        SfcRenderTarget::Dom
    }
}

fn render_target_from_descriptor(
    request: SfcRenderRequest,
    descriptor: &crate::SfcDescriptor<'_>,
) -> SfcRenderTarget {
    if request.ssr {
        return SfcRenderTarget::Ssr;
    }
    let vapor = request.vapor
        || descriptor
            .script_setup
            .as_ref()
            .is_some_and(|script| script.attrs.contains_key("vapor"))
        || descriptor
            .script
            .as_ref()
            .is_some_and(|script| script.attrs.contains_key("vapor"));
    if vapor {
        SfcRenderTarget::Vapor
    } else {
        SfcRenderTarget::Dom
    }
}

fn backend_product(target: SfcRenderTarget) -> ProductId {
    match target {
        SfcRenderTarget::Dom => ProductId::of::<vize_atelier_dom::DomOutputProduct>(),
        SfcRenderTarget::Ssr => ProductId::of::<vize_atelier_ssr::SsrOutputProduct>(),
        SfcRenderTarget::Vapor => ProductId::of::<vize_atelier_vapor::VaporOutputProduct>(),
    }
}
