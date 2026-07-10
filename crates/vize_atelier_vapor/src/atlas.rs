//! Atlas product and backend provider for Vapor planning.

use vize_atlas::{
    Compilation, InputId, PlanningContext, Product, ProductId, Provider, ProviderContext,
    ProviderError, RegisterProviderError,
};
use vize_rendu::{RenderCapabilitiesInput, RenduProduct};

use crate::{VaporPlan, plan_rendu};

/// Owned frontend-neutral Vapor operation plan.
pub struct VaporPlanProduct;

impl Product for VaporPlanProduct {
    type Value = VaporPlan;

    const NAME: &'static str = "backend.vapor-plan";
}

/// Frontend-independent Rendu to Vapor backend.
pub struct VaporProvider;

impl Provider for VaporProvider {
    type Product = VaporPlanProduct;

    fn input_dependencies(&self) -> Vec<InputId> {
        vec![InputId::of::<RenderCapabilitiesInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        context
            .input::<RenderCapabilitiesInput>()
            .is_none_or(|capabilities| capabilities.vapor)
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<RenduProduct>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<VaporPlan, ProviderError> {
        let rendu = context.get::<RenduProduct>()?;
        Ok(plan_rendu(rendu.as_ref()))
    }
}

pub fn register_atlas_provider(compilation: &mut Compilation) -> Result<(), RegisterProviderError> {
    compilation.register_provider(VaporProvider)
}
