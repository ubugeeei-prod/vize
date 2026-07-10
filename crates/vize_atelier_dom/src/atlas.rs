//! Atlas product and backend provider for DOM/VDOM emission.

use vize_atlas::{
    Compilation, InputId, PlanningContext, Product, ProductId, Provider, ProviderContext,
    ProviderError, RegisterProviderError,
};
use vize_rendu::{RenderCapabilitiesInput, RenduProduct};

use crate::{RenduDomOutput, compile_rendu};

/// Emitted DOM/VDOM JavaScript module and provenance mappings.
pub struct DomOutputProduct;

impl Product for DomOutputProduct {
    type Value = RenduDomOutput;

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

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<RenduDomOutput, ProviderError> {
        let rendu = context.get::<RenduProduct>()?;
        Ok(compile_rendu(rendu.as_ref()))
    }
}

pub fn register_atlas_provider(compilation: &mut Compilation) -> Result<(), RegisterProviderError> {
    compilation.register_provider(DomProvider)
}
