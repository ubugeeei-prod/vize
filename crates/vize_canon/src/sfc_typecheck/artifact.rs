//! Typed Atlas request and product for one AST-based SFC typecheck.

use vize_atelier_sfc::{
    SfcCompileOptions, SfcCompileRequest, SfcCompileSettingsInput, SfcCroquisMode,
    SfcCroquisRequest, SfcCroquisSettingsInput, SfcDescriptorProduct, SfcResolvedPropsPolicy,
};
use vize_atlas::{
    Compilation, CompilationInputError, PlanningContext, Product, ProductId, Provider,
    ProviderContext, ProviderError, RegisterProviderError, SourceId, SourceInput, SourceInputId,
};
use vize_croquis::CroquisDocumentProduct;
use vize_flow::FlowProduct;
use vize_relief::ReliefProduct;

use super::{SfcTypeCheckOptions, SfcTypeCheckResult, engine::type_check_from_artifacts};

/// Complete per-source request for Canon's AST-based SFC typechecker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfcTypeCheckRequest {
    pub options: SfcTypeCheckOptions,
    pub mode: SfcCroquisMode,
}

impl SfcTypeCheckRequest {
    pub fn new(options: SfcTypeCheckOptions, mode: SfcCroquisMode) -> Self {
        Self { options, mode }
    }
}

/// Source-local input carrying every option that changes typecheck output.
pub struct SfcTypeCheckSettingsInput;

impl SourceInput for SfcTypeCheckSettingsInput {
    type Value = SfcTypeCheckRequest;

    const NAME: &'static str = "canon.sfc-typecheck-settings";
}

/// Complete diagnostics and optional virtual TypeScript for one SFC source.
pub struct SfcTypeCheckProduct;

impl Product for SfcTypeCheckProduct {
    type Value = SfcTypeCheckResult;

    const NAME: &'static str = "canon.sfc-typecheck";
}

/// Parser-free Canon provider over frontend-owned Atlas artifacts.
#[derive(Debug, Clone, Copy, Default)]
pub struct SfcTypeCheckProvider;

impl Provider for SfcTypeCheckProvider {
    type Product = SfcTypeCheckProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<SfcTypeCheckSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        context.source().name().ends_with(".vue")
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![
            ProductId::of::<SfcDescriptorProduct>(),
            ProductId::of::<ReliefProduct>(),
            ProductId::of::<CroquisDocumentProduct>(),
            ProductId::of::<FlowProduct>(),
        ]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<SfcTypeCheckResult, ProviderError> {
        let request = context
            .source_input::<SfcTypeCheckSettingsInput>()
            .cloned()
            .unwrap_or_else(|| default_request(context.source().name()));
        let descriptor = context.get::<SfcDescriptorProduct>()?;
        let relief = context.get::<ReliefProduct>()?;
        let semantics = context.get::<CroquisDocumentProduct>()?;
        let flow = context.get::<FlowProduct>()?;
        Ok(type_check_from_artifacts(
            &request,
            &descriptor,
            relief.as_ref().as_ref(),
            &semantics,
            &flow,
        ))
    }
}

/// Register Canon's production AST-based SFC typecheck root.
pub fn register_sfc_typecheck_provider(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(SfcTypeCheckProvider)
}

/// Install one request and the matching frontend parse/semantic settings.
pub fn install_sfc_typecheck_request(
    compilation: &mut Compilation,
    source: SourceId,
    request: SfcTypeCheckRequest,
) -> Result<(), CompilationInputError> {
    let filename = request.options.filename.clone();
    let mut compile_options = SfcCompileOptions::default();
    compile_options.parse.filename = filename.clone();
    compilation.set_source_input::<SfcCompileSettingsInput>(
        source,
        SfcCompileRequest::new(compile_options, Default::default()),
    )?;
    compilation.set_source_input::<SfcCroquisSettingsInput>(
        source,
        SfcCroquisRequest {
            mode: request.mode,
            resolved_filename: Some(filename),
            resolved_props_policy: SfcResolvedPropsPolicy::BeforeTemplate,
        },
    )?;
    compilation.set_source_input::<SfcTypeCheckSettingsInput>(source, request)?;
    Ok(())
}

fn default_request(filename: &str) -> SfcTypeCheckRequest {
    SfcTypeCheckRequest::new(SfcTypeCheckOptions::new(filename), SfcCroquisMode::Full)
}

#[cfg(test)]
#[path = "artifact/tests.rs"]
mod tests;
