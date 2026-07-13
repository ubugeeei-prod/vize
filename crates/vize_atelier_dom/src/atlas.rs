//! Atlas product and backend provider for DOM/VDOM emission.

use std::ops::Deref;
use vize_atlas::{
    Compilation, InputId, PlanningContext, Product, ProductId, Provider, ProviderContext,
    ProviderError, RegisterProviderError,
};

use vize_rendu::{RenderCapabilitiesInput, RenduProduct};

use crate::{RenduDomOutput, compile_rendu};

/// DOM outputs preserving the frontend module's root boundaries.
#[derive(Debug, Clone)]
pub struct DomOutputArtifact {
    outputs: Vec<RenduDomOutput>,
}

impl DomOutputArtifact {
    pub fn outputs(&self) -> &[RenduDomOutput] {
        &self.outputs
    }
}

impl Deref for DomOutputArtifact {
    type Target = RenduDomOutput;

    fn deref(&self) -> &Self::Target {
        &self.outputs[0]
    }
}

/// Emitted DOM/VDOM JavaScript module and provenance mappings.
pub struct DomOutputProduct;

impl Product for DomOutputProduct {
    type Value = DomOutputArtifact;

    const NAME: &'static str = "backend.dom-module";
}

/// Frontend-independent Rendu to DOM/VDOM backend.
pub struct DomProvider;

impl Provider for DomProvider {
    type Product = DomOutputProduct;

    fn input_dependencies(&self) -> Vec<InputId> {
        vec![InputId::of::<RenderCapabilitiesInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        context
            .input::<RenderCapabilitiesInput>()
            .is_none_or(|capabilities| capabilities.dom || capabilities.custom_renderer)
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<RenduProduct>()]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<DomOutputArtifact, ProviderError> {
        let rendu = context.get::<RenduProduct>()?;
        Ok(DomOutputArtifact {
            outputs: rendu.roots().iter().map(compile_rendu).collect(),
        })
    }
}

pub fn register_atlas_provider(compilation: &mut Compilation) -> Result<(), RegisterProviderError> {
    if !compilation.has_provider::<DomOutputProduct>() {
        compilation.register_provider(DomProvider)?;
    }
    Ok(())
}
