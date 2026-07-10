//! Atlas product and backend provider for SSR emission.

use vize_atlas::{
    Compilation, InputId, PlanningContext, Product, ProductId, Provider, ProviderContext,
    ProviderError, RegisterProviderError,
};
use vize_rendu::{RenderCapabilitiesInput, RenduProduct};

use crate::{RenduSsrOutput, compile_rendu};

/// Emitted SSR JavaScript module and provenance mappings.
pub struct SsrOutputProduct;

impl Product for SsrOutputProduct {
    type Value = RenduSsrOutput;

    const NAME: &'static str = "backend.ssr-module";
}

/// Frontend-independent Rendu to SSR backend.
pub struct SsrProvider;

impl Provider for SsrProvider {
    type Product = SsrOutputProduct;

    fn input_dependencies(&self) -> Vec<InputId> {
        vec![InputId::of::<RenderCapabilitiesInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        context
            .input::<RenderCapabilitiesInput>()
            .is_none_or(|capabilities| capabilities.ssr)
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<RenduProduct>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<RenduSsrOutput, ProviderError> {
        let rendu = context.get::<RenduProduct>()?;
        Ok(compile_rendu(rendu.as_ref()))
    }
}

pub fn register_atlas_provider(compilation: &mut Compilation) -> Result<(), RegisterProviderError> {
    compilation.register_provider(SsrProvider)
}
