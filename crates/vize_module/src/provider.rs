use vize_atlas::{
    Compilation, ObservationKind, PlanningContext, Product, ProductId, Provider, ProviderContext,
    ProviderError, RegisterProviderError, SourceInputId, SourceKindInput, SourceRange,
};
use vize_carton::{cstr, source_anchor::SourceAnchor};
use vize_flow::{FlowGraph, FlowProduct};

use crate::{ModuleDocument, ModuleLanguage, frontend, project_module_flow};

/// Open Atlas source-kind value owned by the raw module frontend.
pub const MODULE_SOURCE_KIND: &str = "js-module";

pub struct ModuleSyntaxProduct;

impl Product for ModuleSyntaxProduct {
    type Value = ModuleDocument;

    const NAME: &'static str = "module.syntax";
}

pub struct RawModuleSyntaxProvider;

impl Provider for RawModuleSyntaxProvider {
    type Product = ModuleSyntaxProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<SourceKindInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        raw_module_supports(context)
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<ModuleDocument, ProviderError> {
        let source = context.source();
        let language = language_for_path(source.name())
            .ok_or_else(|| ProviderError::message("unsupported module source"))?;
        let module = frontend::snapshot_module(
            source.name(),
            source.text(),
            language,
            0,
            Some(SourceAnchor::new(
                source.id().get(),
                source.revision().get(),
            )),
        );
        for diagnostic in &module.diagnostics {
            context.observe(
                ObservationKind::Diagnostic,
                "module.parse.error",
                diagnostic.message.as_ref(),
                Some(SourceRange::new(
                    diagnostic.span.start as usize,
                    diagnostic.span.end as usize,
                )),
            );
        }
        Ok(frontend::one(module))
    }
}

pub struct ModuleFlowProvider;

impl Provider for ModuleFlowProvider {
    type Product = FlowProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<SourceKindInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        raw_module_supports(context)
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<ModuleSyntaxProduct>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<FlowGraph, ProviderError> {
        let syntax = context.get::<ModuleSyntaxProduct>()?;
        project_module_flow(&syntax).map_err(|error| ProviderError::message(cstr!("{error}")))
    }
}

fn raw_module_supports(context: &PlanningContext<'_>) -> bool {
    context.source_input::<SourceKindInput>().map_or_else(
        || language_for_path(context.source().name()).is_some(),
        |kind| kind.is(MODULE_SOURCE_KIND),
    )
}

pub fn register_raw_providers(compilation: &mut Compilation) -> Result<(), RegisterProviderError> {
    compilation.register_provider(RawModuleSyntaxProvider)?;
    compilation.register_provider(ModuleFlowProvider)?;
    Ok(())
}

pub fn language_for_path(path: &str) -> Option<ModuleLanguage> {
    let clean = path.split(['?', '#']).next().unwrap_or(path);
    match clean.rsplit_once('.').map(|(_, extension)| extension) {
        Some("js" | "mjs" | "cjs") => Some(ModuleLanguage::JavaScript),
        Some("ts" | "mts" | "cts") => Some(ModuleLanguage::TypeScript),
        _ => None,
    }
}
