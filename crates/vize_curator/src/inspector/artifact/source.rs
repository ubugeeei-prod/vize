//! Per-source inspector analysis produced once from frontend artifacts.

use vize_atelier_jsx::JsxSyntaxProduct;
use vize_atelier_sfc::SfcDescriptorProduct;
use vize_atlas::{
    Compilation, PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError,
};
use vize_croquis::{CroquisDocumentProduct, CroquisSemanticSnapshot};
use vize_module::{ModuleDocument, ModuleSyntaxProduct};
use vize_relief::ReliefProduct;

use super::super::imports::{FileAnalysis, analyze_component_file, analyze_script_file};

/// Cached imports, template usage, and semantics for one inspector source.
#[derive(Debug, Clone, Default)]
pub struct InspectorSourceAnalysis {
    pub(in crate::inspector) graph: FileAnalysis,
    pub(in crate::inspector) semantic: Option<CroquisSemanticSnapshot>,
    pub(in crate::inspector) sfc_parse_error: bool,
    pub(in crate::inspector) jsx_parse_error: bool,
    pub(in crate::inspector) template_parse_error: bool,
}

/// Per-source inspector artifact. It does not aggregate or trigger other files.
pub struct InspectorSourceAnalysisProduct;

impl Product for InspectorSourceAnalysisProduct {
    type Value = InspectorSourceAnalysis;

    const NAME: &'static str = "curator.inspector.source-analysis";
}

/// Selects the SFC, JSX/TSX, or raw Module frontend for one inspector source.
pub struct InspectorSourceAnalysisProvider;

impl Provider for InspectorSourceAnalysisProvider {
    type Product = InspectorSourceAnalysisProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_inspector_source(context.source().name())
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        if is_sfc_source(context.source().name()) {
            let mut dependencies = vec![
                ProductId::of::<SfcDescriptorProduct>(),
                ProductId::of::<ReliefProduct>(),
                ProductId::of::<CroquisDocumentProduct>(),
            ];
            if vize_atelier_sfc::sfc_source_has_script(context.source().text()) {
                dependencies.push(ProductId::of::<ModuleSyntaxProduct>());
            }
            dependencies
        } else if is_jsx_source(context.source().name()) {
            vec![
                ProductId::of::<JsxSyntaxProduct>(),
                ProductId::of::<ModuleSyntaxProduct>(),
                ProductId::of::<CroquisDocumentProduct>(),
            ]
        } else {
            vec![ProductId::of::<ModuleSyntaxProduct>()]
        }
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<InspectorSourceAnalysis, ProviderError> {
        if !is_sfc_source(context.source().name()) {
            let modules = context.get::<ModuleSyntaxProduct>()?;
            let is_jsx = is_jsx_source(context.source().name());
            let semantic = if is_jsx {
                Some(context.get::<CroquisDocumentProduct>()?.semantic_snapshot())
            } else {
                None
            };
            let jsx_parse_error = if is_jsx {
                context.get::<JsxSyntaxProduct>()?.has_errors()
            } else {
                false
            };
            let graph = semantic.as_ref().map_or_else(
                || analyze_script_file(&modules),
                |semantic| analyze_component_file(&modules, semantic),
            );
            return Ok(InspectorSourceAnalysis {
                graph,
                semantic,
                jsx_parse_error,
                ..Default::default()
            });
        }

        let descriptor = context.get::<SfcDescriptorProduct>()?;
        let relief = context.get::<ReliefProduct>()?;
        let document = context.get::<CroquisDocumentProduct>()?;
        if descriptor.descriptor().is_none() {
            return Ok(InspectorSourceAnalysis {
                sfc_parse_error: true,
                ..Default::default()
            });
        }
        let semantic = document.semantic_snapshot();
        let modules = if vize_atelier_sfc::sfc_source_has_script(context.source().text()) {
            Some(context.get::<ModuleSyntaxProduct>()?)
        } else {
            None
        };
        let empty_modules = ModuleDocument::default();
        Ok(InspectorSourceAnalysis {
            graph: analyze_component_file(modules.as_deref().unwrap_or(&empty_modules), &semantic),
            semantic: Some(semantic),
            template_parse_error: relief
                .as_ref()
                .as_ref()
                .is_some_and(|syntax| syntax.has_fatal_diagnostics()),
            sfc_parse_error: false,
            jsx_parse_error: false,
        })
    }
}

fn is_jsx_source(name: &str) -> bool {
    source_name_has_extension(name, ".jsx") || source_name_has_extension(name, ".tsx")
}

fn is_sfc_source(name: &str) -> bool {
    source_name_has_extension(name, ".vue")
}

pub(super) fn is_inspector_source(name: &str) -> bool {
    [
        ".vue", ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs",
    ]
    .into_iter()
    .any(|extension| source_name_has_extension(name, extension))
}

fn source_name_has_extension(name: &str, extension: &str) -> bool {
    name.ends_with(extension)
        || name.char_indices().any(|(index, character)| {
            matches!(character, '?' | '#') && name[..index].ends_with(extension)
        })
}

/// Keep the standalone graph API on the same typed product path.
pub(in crate::inspector) fn analyze_source_compatibility(path: &str, text: &str) -> FileAnalysis {
    let mut compilation = Compilation::new();
    if super::report::register_inspector_atlas_providers(&mut compilation).is_err() {
        return FileAnalysis::default();
    }
    let Ok(source) = compilation.add_source(path, text) else {
        return FileAnalysis::default();
    };
    compilation
        .query::<InspectorSourceAnalysisProduct>(source)
        .map(|outcome| outcome.value().graph.clone())
        .unwrap_or_default()
}
