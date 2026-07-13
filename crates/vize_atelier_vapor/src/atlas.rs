//! Atlas product and backend provider for Vapor planning.

use std::ops::Deref;

use vize_atlas::{
    Compilation, InputId, PlanningContext, Product, ProductId, Provider, ProviderContext,
    ProviderError, RegisterProviderError,
};
use vize_carton::String;
use vize_rendu::{RenderCapabilitiesInput, RenduProduct, RenduSpan};

use crate::{VaporEmitResult, VaporPlan, emit_vapor_plan, plan_rendu};

/// Per-root Vapor plans preserving frontend component boundaries.
#[derive(Debug, Clone)]
pub struct VaporPlanArtifact {
    plans: Vec<VaporPlan>,
}

impl VaporPlanArtifact {
    pub fn plans(&self) -> &[VaporPlan] {
        &self.plans
    }
}

impl Deref for VaporPlanArtifact {
    type Target = VaporPlan;

    fn deref(&self) -> &Self::Target {
        &self.plans[0]
    }
}

/// One expression mapping emitted directly by the Vapor backend.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct VaporOutputMapping {
    pub generated_start: usize,
    pub source: RenduSpan,
}

/// Emitted Vapor output with frontend-neutral provenance mappings.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VaporOutput {
    pub code: String,
    pub templates: Vec<String>,
    pub mappings: Vec<VaporOutputMapping>,
}

/// Emitted Vapor outputs preserving frontend component boundaries.
#[derive(Debug, Clone)]
pub struct VaporOutputArtifact {
    outputs: Vec<VaporOutput>,
}

impl VaporOutputArtifact {
    pub fn outputs(&self) -> &[VaporOutput] {
        &self.outputs
    }
}

impl Deref for VaporOutputArtifact {
    type Target = VaporOutput;

    fn deref(&self) -> &Self::Target {
        &self.outputs[0]
    }
}

/// Owned frontend-neutral Vapor operation plan.
pub struct VaporPlanProduct;

impl Product for VaporPlanProduct {
    type Value = VaporPlanArtifact;

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

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<VaporPlanArtifact, ProviderError> {
        let rendu = context.get::<RenduProduct>()?;
        Ok(VaporPlanArtifact {
            plans: rendu.roots().iter().map(plan_rendu).collect(),
        })
    }
}

/// Fully emitted Vapor JavaScript product.
pub struct VaporOutputProduct;

impl Product for VaporOutputProduct {
    type Value = VaporOutputArtifact;

    const NAME: &'static str = "backend.vapor-module";
}

/// Plan-to-output Vapor provider kept independent from frontend compilers.
pub struct VaporOutputProvider;

impl Provider for VaporOutputProvider {
    type Product = VaporOutputProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        context
            .input::<RenderCapabilitiesInput>()
            .is_none_or(|capabilities| capabilities.vapor)
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<VaporPlanProduct>()]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<VaporOutputArtifact, ProviderError> {
        let plans = context.get::<VaporPlanProduct>()?;
        Ok(VaporOutputArtifact {
            outputs: plans.plans().iter().map(emit_plan).collect(),
        })
    }
}

fn emit_plan(plan: &VaporPlan) -> VaporOutput {
    let VaporEmitResult { code, templates } = emit_vapor_plan(plan);
    let mut mappings = Vec::new();
    let mut cursor = 0;
    for expression in plan.expressions() {
        let Some(source) = expression.provenance.primary else {
            continue;
        };
        let Some(relative) = code[cursor..].find(expression.code.as_ref()) else {
            continue;
        };
        let generated_start = cursor + relative;
        mappings.push(VaporOutputMapping {
            generated_start,
            source,
        });
        cursor = generated_start.saturating_add(expression.code.len());
    }
    VaporOutput {
        code,
        templates,
        mappings,
    }
}

pub fn register_atlas_provider(compilation: &mut Compilation) -> Result<(), RegisterProviderError> {
    if !compilation.has_provider::<VaporPlanProduct>() {
        compilation.register_provider(VaporProvider)?;
    }
    if !compilation.has_provider::<VaporOutputProduct>() {
        compilation.register_provider(VaporOutputProvider)?;
    }
    Ok(())
}
