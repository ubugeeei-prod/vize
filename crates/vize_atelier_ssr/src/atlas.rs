//! Atlas product and backend provider for SSR emission.

use std::ops::Deref;
use vize_atlas::{
    Compilation, InputId, PlanningContext, Product, ProductId, Provider, ProviderContext,
    ProviderError, RegisterProviderError,
};

use vize_rendu::{RenderCapabilitiesInput, RenduProduct};

use crate::{RenduSsrOutput, compile_rendu};

/// SSR outputs preserving the frontend module's root boundaries.
#[derive(Debug, Clone)]
pub struct SsrOutputArtifact {
    outputs: Vec<RenduSsrOutput>,
}

impl SsrOutputArtifact {
    pub fn outputs(&self) -> &[RenduSsrOutput] {
        &self.outputs
    }
}

impl Deref for SsrOutputArtifact {
    type Target = RenduSsrOutput;

    fn deref(&self) -> &Self::Target {
        &self.outputs[0]
    }
}

/// Emitted SSR JavaScript module and provenance mappings.
pub struct SsrOutputProduct;

impl Product for SsrOutputProduct {
    type Value = SsrOutputArtifact;

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

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<SsrOutputArtifact, ProviderError> {
        let rendu = context.get::<RenduProduct>()?;
        Ok(SsrOutputArtifact {
            outputs: rendu.roots().iter().map(compile_rendu).collect(),
        })
    }
}

pub fn register_atlas_provider(compilation: &mut Compilation) -> Result<(), RegisterProviderError> {
    if !compilation.has_provider::<SsrOutputProduct>() {
        compilation.register_provider(SsrProvider)?;
    }
    Ok(())
}
