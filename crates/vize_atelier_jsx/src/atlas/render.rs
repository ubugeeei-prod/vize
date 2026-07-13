//! Component-boundary-preserving Rendu module product for JSX backends.

use vize_atlas::{
    PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError, SourceInputId,
    SourceKindInput,
};
use vize_rendu::RenduRoot;

use super::{JsxSyntaxProduct, is_jsx_context};
use crate::{JsxDiagnostic, JsxSyntaxRootMetadata};
use vize_rendu::RenduProduct;

/// One independently compilable component root and its authored context.
#[derive(Debug, Clone)]
pub struct JsxRenderRoot {
    pub rendu: RenduRoot,
    pub metadata: JsxSyntaxRootMetadata,
}

/// Owned render roots and diagnostics for one JSX/TSX module.
#[derive(Debug, Clone)]
pub struct JsxRenderModule {
    pub roots: Vec<JsxRenderRoot>,
    pub diagnostics: Vec<JsxDiagnostic>,
}

/// JSX-owned render module preserving component boundaries over Rendu HIR.
pub struct JsxRenderModuleProduct;

impl Product for JsxRenderModuleProduct {
    type Value = JsxRenderModule;
    const NAME: &'static str = "jsx.render-module";
}

/// Build every per-component Rendu root from the cached owned syntax.
pub struct JsxRenderModuleProvider;

impl Provider for JsxRenderModuleProvider {
    type Product = JsxRenderModuleProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<SourceKindInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_jsx_context(context)
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![
            ProductId::of::<JsxSyntaxProduct>(),
            ProductId::of::<RenduProduct>(),
        ]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<JsxRenderModule, ProviderError> {
        let syntax = context.get::<JsxSyntaxProduct>()?;
        let rendu = context.get::<RenduProduct>()?;
        if rendu.roots().len() != syntax.root_metadata().len() {
            return Err(ProviderError::message(
                "JSX Rendu roots and metadata are misaligned",
            ));
        }
        let mut roots = Vec::with_capacity(rendu.roots().len());
        for (rendu, metadata) in rendu.roots().iter().zip(syntax.root_metadata()) {
            roots.push(JsxRenderRoot {
                rendu: rendu.clone(),
                metadata: metadata.clone(),
            });
        }
        Ok(JsxRenderModule {
            roots,
            diagnostics: syntax.diagnostics.clone(),
        })
    }
}
