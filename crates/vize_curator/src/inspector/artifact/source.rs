//! Per-source inspector analysis produced once from frontend artifacts.

use vize_atelier_sfc::SfcDescriptorProduct;
use vize_atlas::{
    Compilation, PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError,
};
use vize_croquis::{CroquisDocumentProduct, CroquisSemanticSnapshot};
use vize_relief::ReliefProduct;

use super::super::imports::{FileAnalysis, analyze_script_file, analyze_sfc_file};

/// Cached imports, template usage, and semantics for one inspector source.
#[derive(Debug, Clone, Default)]
pub struct InspectorSourceAnalysis {
    pub(in crate::inspector) graph: FileAnalysis,
    pub(in crate::inspector) semantic: Option<CroquisSemanticSnapshot>,
    pub(in crate::inspector) sfc_parse_error: bool,
    pub(in crate::inspector) template_parse_error: bool,
}

/// Per-source inspector artifact. It does not aggregate or trigger other files.
pub struct InspectorSourceAnalysisProduct;

impl Product for InspectorSourceAnalysisProduct {
    type Value = InspectorSourceAnalysis;

    const NAME: &'static str = "curator.inspector.source-analysis";
}

/// Selects the Vue frontend graph for SFCs and raw OXC syntax for script files.
pub struct InspectorSourceAnalysisProvider;

impl Provider for InspectorSourceAnalysisProvider {
    type Product = InspectorSourceAnalysisProduct;

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_inspector_source(context.source().name())
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        if context.source().name().ends_with(".vue") {
            vec![
                ProductId::of::<SfcDescriptorProduct>(),
                ProductId::of::<ReliefProduct>(),
                ProductId::of::<CroquisDocumentProduct>(),
            ]
        } else {
            Vec::new()
        }
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<InspectorSourceAnalysis, ProviderError> {
        if !context.source().name().ends_with(".vue") {
            return Ok(InspectorSourceAnalysis {
                graph: analyze_script_file(context.source().name(), context.source().text()),
                ..Default::default()
            });
        }

        let descriptor = context.get::<SfcDescriptorProduct>()?;
        let relief = context.get::<ReliefProduct>()?;
        let document = context.get::<CroquisDocumentProduct>()?;
        let Some(descriptor) = descriptor.descriptor() else {
            return Ok(InspectorSourceAnalysis {
                sfc_parse_error: true,
                ..Default::default()
            });
        };
        let semantic = document.semantic_snapshot();
        Ok(InspectorSourceAnalysis {
            graph: analyze_sfc_file(descriptor, &semantic),
            semantic: Some(semantic),
            template_parse_error: relief
                .as_ref()
                .as_ref()
                .is_some_and(|syntax| syntax.has_fatal_diagnostics()),
            sfc_parse_error: false,
        })
    }
}

pub(super) fn is_inspector_source(name: &str) -> bool {
    let name = name.split(['?', '#']).next().unwrap_or(name);
    matches!(
        name.rsplit_once('.').map(|(_, extension)| extension),
        Some("vue" | "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs")
    )
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
