//! Full CrossFileAnalyzer execution as an opt-in Atlas product.

#[path = "analysis/model.rs"]
mod model;

use model::OffsetSegment;
pub use model::{
    CrossFileAnalysisArtifact, CrossFileAnalysisInput, CrossFileAnalysisProduct,
    CrossFileAnalysisRequest, CrossFileOffsetRegion, CrossFileSourceLayout,
};

use std::path::Path;

use vize_atlas::{
    InputId, PlanningContext, ProductRequest, Provider, ProviderContext, ProviderError, SourceId,
};
use vize_carton::{CompactString, String};
use vize_croquis::{
    CroquisDocument, CroquisDocumentProduct, EffectGraphScript, EffectGraphSummary,
    build_effect_graph_from_sfc_scripts,
};

use crate::{CrossFileAnalyzer, FileId};

/// Build the full analysis from frontend-owned Croquis documents and raw modules.
pub struct CrossFileAnalysisProvider;

impl Provider for CrossFileAnalysisProvider {
    type Product = CrossFileAnalysisProduct;

    fn input_dependencies(&self) -> Vec<InputId> {
        vec![InputId::of::<CrossFileAnalysisInput>()]
    }

    fn source_dependencies(&self, context: &PlanningContext<'_>) -> Vec<SourceId> {
        source_ids(context, is_raw_module)
    }

    fn dependency_requests(&self, context: &PlanningContext<'_>) -> Vec<ProductRequest> {
        source_ids(context, is_document_source)
            .into_iter()
            .map(ProductRequest::for_product::<CroquisDocumentProduct>)
            .collect()
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<CrossFileAnalysisArtifact, ProviderError> {
        let request = context
            .input::<CrossFileAnalysisInput>()
            .cloned()
            .unwrap_or_default();
        let mut analyzer = request.project_root.map_or_else(
            || CrossFileAnalyzer::new(request.options.clone()),
            |root| CrossFileAnalyzer::with_project_root(request.options.clone(), root),
        );
        let mut layouts = Vec::new();
        let mut sources: Vec<_> = context
            .sources()
            .iter()
            .filter(|source| is_supported_source(source.name()))
            .cloned()
            .collect();
        sources.sort_by_key(|source| source.id());

        for source in sources {
            if is_document_source(source.name()) {
                let document = context.get_for_source::<CroquisDocumentProduct>(source.id())?;
                let (analysis_source, mut layout) = document_source_layout(
                    source.id(),
                    source.name(),
                    source.text(),
                    document.as_ref(),
                );
                let path = Path::new(source.name());
                let file = if path.extension().and_then(|value| value.to_str()) == Some("vue") {
                    analyzer.add_file_with_analysis_and_effect_summary(
                        path,
                        analysis_source.as_str(),
                        document.shared_analysis(),
                        document_effect_summary(document.as_ref()),
                    )
                } else {
                    analyzer.add_file_with_analysis(
                        path,
                        analysis_source.as_str(),
                        document.shared_analysis(),
                    )
                };
                layout.file = file;
                record_layout(&mut layouts, layout);
            } else {
                let file = analyzer.add_file(Path::new(source.name()), source.text());
                record_layout(
                    &mut layouts,
                    identity_layout(file, source.id(), source.name()),
                );
            }
        }

        analyzer.rebuild_import_edges();
        analyzer.rebuild_component_edges();
        let result = analyzer.analyze();
        let provide_inject_tree = result
            .provide_inject_tree
            .as_ref()
            .map(|tree| tree.to_markdown(analyzer.registry()));
        layouts.sort_by_key(|layout| layout.file.as_u32());
        Ok(CrossFileAnalysisArtifact {
            result,
            layouts,
            provide_inject_tree,
        })
    }
}

fn record_layout(layouts: &mut Vec<CrossFileSourceLayout>, layout: CrossFileSourceLayout) {
    if let Some(previous) = layouts
        .iter_mut()
        .find(|previous| previous.file == layout.file)
    {
        *previous = layout;
    } else {
        layouts.push(layout);
    }
}

fn document_effect_summary(document: &CroquisDocument) -> EffectGraphSummary {
    let script = document
        .source_by_role("script")
        .map(|segment| EffectGraphScript::new(segment.text(), segment.language()));
    let setup = document
        .source_by_role("script-setup")
        .map(|segment| EffectGraphScript::new(segment.text(), segment.language()));
    build_effect_graph_from_sfc_scripts(script, setup).summary()
}

fn source_ids(context: &PlanningContext<'_>, predicate: fn(&str) -> bool) -> Vec<SourceId> {
    let mut sources: Vec<_> = context
        .sources()
        .iter()
        .filter(|source| predicate(source.name()))
        .map(|source| source.id())
        .collect();
    sources.sort_unstable();
    sources
}

fn is_document_source(name: &str) -> bool {
    matches!(extension(name), Some("vue" | "jsx" | "tsx"))
}

fn is_raw_module(name: &str) -> bool {
    matches!(
        extension(name),
        Some("js" | "ts" | "mjs" | "mts" | "cjs" | "cts")
    )
}

fn is_supported_source(name: &str) -> bool {
    is_document_source(name) || is_raw_module(name)
}

fn extension(name: &str) -> Option<&str> {
    Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
}

fn document_source_layout(
    source: SourceId,
    path: &str,
    source_text: &str,
    document: &CroquisDocument,
) -> (String, CrossFileSourceLayout) {
    if !path.ends_with(".vue") {
        return (
            String::from(source_text),
            identity_layout(FileId::INVALID, source, path),
        );
    }
    let mut analysis_source = String::default();
    let mut segments = Vec::new();
    for role in ["script", "script-setup"] {
        let Some(segment) = document.source_by_role(role) else {
            continue;
        };
        if !analysis_source.is_empty() {
            analysis_source.push('\n');
        }
        let generated_start = analysis_source.len() as u32;
        analysis_source.push_str(segment.text());
        let generated_end = analysis_source.len() as u32;
        let source_start = segment
            .anchor()
            .parent_range()
            .map_or(0, |range| range.start);
        segments.push(OffsetSegment {
            generated_start,
            generated_end,
            source_start,
        });
    }
    let template_content_start = document
        .source_by_role("template")
        .and_then(|segment| segment.anchor().parent_range())
        .map_or(0, |range| range.start);
    let template_tag = template_tag_range(source_text, template_content_start);
    (
        analysis_source,
        CrossFileSourceLayout {
            file: FileId::INVALID,
            source,
            path: CompactString::new(path),
            script: segments,
            template_tag,
            template_content_start,
        },
    )
}

fn identity_layout(file: FileId, source: SourceId, path: &str) -> CrossFileSourceLayout {
    CrossFileSourceLayout {
        file,
        source,
        path: CompactString::new(path),
        script: Vec::new(),
        template_tag: (0, 0),
        template_content_start: 0,
    }
}

fn template_tag_range(source: &str, content_start: u32) -> (u32, u32) {
    let content_start = content_start as usize;
    let prefix = source.get(..content_start).unwrap_or(source);
    let start = prefix.rfind("<template").unwrap_or(content_start);
    (start as u32, content_start as u32)
}
