//! Full CrossFileAnalyzer execution as an opt-in Atlas product.

#[path = "analysis/model.rs"]
mod model;

use model::OffsetSegment;
pub use model::{
    CrossFileAnalysisArtifact, CrossFileAnalysisInput, CrossFileAnalysisProduct,
    CrossFileAnalysisRequest, CrossFileOffsetRegion, CrossFileSourceLayout,
};

use std::path::Path;

use vize_atelier_sfc::sfc_source_structure;
use vize_atlas::{
    InputId, PlanningContext, ProductRequest, Provider, ProviderContext, ProviderError, SourceId,
};
use vize_carton::{CompactString, String};
use vize_croquis::{CroquisDocument, CroquisDocumentProduct};
use vize_module::{ModuleDocument, ModuleSyntaxProduct};

use super::module_facts::{module_effect_summary, module_store_factories, project_raw_croquis};
use crate::analyzer::AtlasModuleFacts;
use crate::{CrossFileAnalyzer, FileId};

/// Build the full analysis from frontend-owned Croquis documents and raw modules.
pub struct CrossFileAnalysisProvider;

impl Provider for CrossFileAnalysisProvider {
    type Product = CrossFileAnalysisProduct;

    fn input_dependencies(&self) -> Vec<InputId> {
        vec![InputId::of::<CrossFileAnalysisInput>()]
    }

    fn source_dependencies(&self, context: &PlanningContext<'_>) -> Vec<SourceId> {
        source_ids(context, is_supported_source)
    }

    fn dependency_requests(&self, context: &PlanningContext<'_>) -> Vec<ProductRequest> {
        let mut requests = Vec::new();
        for source in source_ids(context, is_supported_source) {
            let Some(snapshot) = context.source_by_id(source) else {
                continue;
            };
            let source_name = snapshot.name();
            // Every value read by `provide` is a direct declared request even
            // when another frontend also reaches it transitively. Template-only
            // SFCs intentionally have no Module product, so retain that split at
            // planning time without parsing JavaScript or TypeScript here.
            if source_needs_module(source_name, snapshot.text()) {
                requests.push(ProductRequest::for_product::<ModuleSyntaxProduct>(source));
            }
            if is_document_source(source_name) {
                requests.push(ProductRequest::for_product::<CroquisDocumentProduct>(
                    source,
                ));
            }
        }
        requests
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
                let is_vue = source.name().ends_with(".vue");
                let has_script = document.source_by_role("script").is_some()
                    || document.source_by_role("script-setup").is_some();
                let modules = if !is_vue || has_script {
                    Some(context.get_for_source::<ModuleSyntaxProduct>(source.id())?)
                } else {
                    None
                };
                let (analysis_source, mut layout) = document_source_layout(
                    source.id(),
                    source.name(),
                    source.text(),
                    document.as_ref(),
                );
                let path = Path::new(source.name());
                let template_source = document
                    .source_by_role("template")
                    .map_or("", |template| template.text());
                let empty_modules = ModuleDocument::default();
                let modules = modules.as_deref().unwrap_or(&empty_modules);
                let effect_summary = module_effect_summary(modules);
                let file = analyzer.add_atlas_file_with_analysis_and_effect_summary(
                    path,
                    analysis_source.as_str(),
                    template_source,
                    document.shared_analysis(),
                    AtlasModuleFacts {
                        document: modules,
                        effect_summary,
                        store_factories: module_store_factories(modules),
                    },
                );
                layout.file = file;
                record_layout(&mut layouts, layout);
            } else {
                let modules = context.get_for_source::<ModuleSyntaxProduct>(source.id())?;
                let analysis = project_raw_croquis(modules.as_ref());
                let file = analyzer.add_atlas_file_with_analysis_and_effect_summary(
                    Path::new(source.name()),
                    source.text(),
                    "",
                    analysis,
                    AtlasModuleFacts {
                        document: modules.as_ref(),
                        effect_summary: module_effect_summary(modules.as_ref()),
                        store_factories: module_store_factories(modules.as_ref()),
                    },
                );
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

fn source_needs_module(name: &str, source: &str) -> bool {
    !name.ends_with(".vue") || sfc_source_structure(source).has_script
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

#[cfg(test)]
mod planning_tests {
    use super::source_needs_module;

    #[test]
    fn comments_and_template_literals_do_not_create_sfc_module_work() {
        for source in [
            "<!-- <script setup>const fake = true</script> --><template><main /></template>",
            r#"<template>{{ '<script setup>' }}<div data-code="<script>" /></template>"#,
            r#"<custom-block>const tag = "<script>"</custom-block><template />"#,
        ] {
            assert!(!source_needs_module("OnlyTemplate.vue", source), "{source}");
        }
    }

    #[test]
    fn similar_or_malformed_spellings_do_not_replace_sfc_structure_rules() {
        for source in [
            "<scripture>not a script block</scripture><template />",
            "<script-setup>not a script block</script-setup><template />",
            "<scriptx>not a script block</scriptx><template />",
        ] {
            assert!(!source_needs_module("Adversarial.vue", source), "{source}");
        }
        assert!(source_needs_module(
            "Actual.vue",
            "<script setup lang=\"ts\">const ready = true</script><template>{{ ready }}</template>",
        ));
        assert!(source_needs_module("raw.ts", "export const ready = true"));
    }
}
