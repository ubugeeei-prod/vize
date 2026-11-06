//! Shared projection view for the latency-sensitive hover path.
#![expect(
    clippy::disallowed_types,
    reason = "warm hover shares one immutable projection across repeated requests"
)]

use std::sync::Arc;

use vize_carton::line_index::LineBreaks;

use super::CorsaVueVirtualDocument;
use crate::corsa_bridge::LspRange;
use crate::virtual_ts::{ProjectionFeatures, ProjectionRow, VizeMapping};

/// One immutable Canon projection shared by repeated hover requests.
///
/// Unlike the general editor document, this view does not copy dependency or
/// materialized-source payloads merely to map one local quick-info response.
#[derive(Clone)]
pub struct CorsaVueHoverDocument {
    document: Arc<CorsaVueVirtualDocument>,
}

impl CorsaVueHoverDocument {
    pub(super) fn new(document: Arc<CorsaVueVirtualDocument>) -> Self {
        Self { document }
    }

    /// The native URI already synchronized for this projection.
    pub fn request_uri(&self) -> &str {
        &self.document.request_uri
    }

    /// Map an authored byte into the generated LSP coordinate used by Corsa.
    pub fn source_offset_to_position(&self, source_offset: usize) -> Option<(u32, u32)> {
        let mapping = hover_rows(&self.document)
            .filter(|row| contains_source(row.span, source_offset))
            .min_by_key(|row| source_rank(row.span, source_offset))?;
        let generated_pre_rewrite = source_to_generated(mapping.span, source_offset);
        let generated = self
            .document
            .import_source_map
            .get_virtual_offset(generated_pre_rewrite as u32) as usize;
        Some(LineBreaks::Lsp.offset_to_position(&self.document.code, generated))
    }

    /// Map a Corsa hover range back to authored byte offsets.
    pub fn range_to_source_offsets(&self, range: &LspRange) -> Option<(usize, usize)> {
        let generated_start_post = LineBreaks::Lsp.position_to_offset(
            &self.document.code,
            range.start.line,
            range.start.character,
        )?;
        let generated_end_post = LineBreaks::Lsp
            .position_to_offset(&self.document.code, range.end.line, range.end.character)
            .unwrap_or(generated_start_post);
        let generated_start =
            self.document
                .import_source_map
                .get_original_offset(generated_start_post as u32) as usize;
        let generated_end = self
            .document
            .import_source_map
            .get_original_offset(generated_end_post as u32) as usize;

        let mut rows = hover_rows(&self.document).peekable();
        if rows.peek().is_none() {
            return Some((generated_start, generated_end.max(generated_start)));
        }
        let start_mapping = rows
            .filter(|row| contains_generated(row.span, generated_start))
            .min_by_key(|row| row.span.gen_range.len())?;
        let source_start = generated_to_source(start_mapping.span, generated_start, false);
        let source_end = hover_rows(&self.document)
            .filter(|row| contains_generated(row.span, generated_end))
            .min_by_key(|row| row.span.gen_range.len())
            .map(|row| generated_to_source(row.span, generated_end, true))
            .unwrap_or_else(|| {
                source_start
                    .saturating_add(generated_end.saturating_sub(generated_start))
                    .min(start_mapping.span.src_range.end)
            })
            .max(source_start);
        Some((source_start, source_end))
    }

    #[cfg(test)]
    fn shares_projection_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.document, &other.document)
    }
}

impl From<CorsaVueVirtualDocument> for CorsaVueHoverDocument {
    fn from(document: CorsaVueVirtualDocument) -> Self {
        Self::new(Arc::new(document))
    }
}

fn hover_rows(document: &CorsaVueVirtualDocument) -> impl Iterator<Item = ProjectionRow<'_>> + '_ {
    document
        .mapping
        .rows()
        .filter(|row| row.meta.features != ProjectionFeatures::DIAGNOSTICS)
}

fn contains_source(mapping: &VizeMapping, offset: usize) -> bool {
    (mapping.src_range.start <= offset && offset <= mapping.src_range.end)
        || mapping
            .sub_spans
            .iter()
            .any(|span| span.src_range.start <= offset && offset <= span.src_range.end)
}

fn source_rank(mapping: &VizeMapping, offset: usize) -> (usize, usize) {
    mapping
        .sub_spans
        .iter()
        .filter(|span| span.src_range.start <= offset && offset <= span.src_range.end)
        .map(|span| (span.src_range.len(), span.gen_range.len()))
        .min()
        .unwrap_or_else(|| (mapping.src_range.len(), mapping.gen_range.len()))
}

fn source_to_generated(mapping: &VizeMapping, source_offset: usize) -> usize {
    if let Some(span) = mapping
        .sub_spans
        .iter()
        .find(|span| span.src_range.start <= source_offset && source_offset <= span.src_range.end)
    {
        let relative = source_offset.saturating_sub(span.src_range.start);
        return span
            .gen_range
            .start
            .saturating_add(relative.min(span.gen_range.len()));
    }
    let relative = source_offset.saturating_sub(mapping.src_range.start);
    mapping
        .gen_range
        .start
        .saturating_add(relative.min(mapping.gen_range.len()))
}

fn contains_generated(mapping: &VizeMapping, offset: usize) -> bool {
    mapping.gen_range.start <= offset && offset <= mapping.gen_range.end
}

fn generated_to_source(mapping: &VizeMapping, generated_offset: usize, prefer_end: bool) -> usize {
    if prefer_end {
        crate::virtual_ts::mapping::map_generated_end_offset_to_source(mapping, generated_offset)
    } else {
        crate::virtual_ts::mapping::map_generated_offset_to_source(mapping, generated_offset)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::batch::ImportSourceMap;
    use crate::corsa_bridge::{CorsaSourceCatalog, LspPosition};
    use crate::virtual_ts::{ProjectionMapping, VizeSubSpan};

    fn document() -> CorsaVueHoverDocument {
        let mapping = VizeMapping {
            src_range: 10..20,
            gen_range: 100..120,
            sub_spans: vec![VizeSubSpan {
                src_range: 12..16,
                gen_range: 104..108,
            }],
        };
        CorsaVueHoverDocument::new(Arc::new(CorsaVueVirtualDocument {
            request_uri: "file:///App.vue.ts".into(),
            code: format!("{}value{}", " ".repeat(104), " ".repeat(12)).into(),
            pre_rewrite_code: vize_carton::String::new(""),
            mapping: ProjectionMapping::from_spans(vec![mapping]),
            import_source_map: ImportSourceMap::empty(),
            source_type: oxc_span::SourceType::from_path(Path::new("App.ts")).unwrap(),
            virtual_suffix: ".ts",
            dependencies: Vec::new(),
            resolved_dependencies: Vec::new(),
            materialized_sources: Vec::new(),
            source_catalog: CorsaSourceCatalog::default(),
            session_project_root: None,
        }))
    }

    #[test]
    fn clones_share_projection_and_map_hover_coordinates() {
        let document = document();
        let second = document.clone();
        assert!(document.shares_projection_with(&second));
        assert_eq!(document.source_offset_to_position(14), Some((0, 106)));
        assert_eq!(
            document.range_to_source_offsets(&LspRange {
                start: LspPosition {
                    line: 0,
                    character: 104,
                },
                end: LspPosition {
                    line: 0,
                    character: 108,
                },
            }),
            Some((12, 16))
        );
    }

    #[test]
    fn real_projection_round_trips_unicode_after_import_rewrite() {
        let workspace = tempfile::tempdir().unwrap();
        let host_path = workspace.path().join("Host.vue");
        let child_path = workspace.path().join("Child.vue");
        let source = r#"<script setup lang="ts">
import Child from "./Child.vue"
const 雪 = "🌸"
</script>
<template><Child :title="`前🌸${雪}`" /></template>
"#;
        std::fs::write(&host_path, source).unwrap();
        std::fs::write(&child_path, "<template><span /></template>").unwrap();

        let project = crate::corsa_bridge::vue_document::build_vue_virtual_project(
            &host_path,
            source,
            super::super::CorsaVueVirtualDocumentOptions::default(),
        )
        .unwrap();
        assert_ne!(
            project.host.import_source_map,
            ImportSourceMap::empty(),
            "the .vue import must exercise post-rewrite coordinates"
        );
        let document = CorsaVueHoverDocument::new(Arc::clone(&project.host));
        let source_start = source.rfind('雪').unwrap();
        let source_end = source_start + '雪'.len_utf8();
        let start = document
            .source_offset_to_position(source_start)
            .expect("mapped Unicode token start");
        let end = document
            .source_offset_to_position(source_end)
            .expect("mapped Unicode token end");
        let generated_start = LineBreaks::Lsp
            .position_to_offset(&project.host.code, start.0, start.1)
            .unwrap();
        let generated_end = LineBreaks::Lsp
            .position_to_offset(&project.host.code, end.0, end.1)
            .unwrap();

        assert_eq!(&project.host.code[generated_start..generated_end], "雪");
        assert_ne!(
            project
                .host
                .import_source_map
                .get_original_offset(generated_start as u32),
            generated_start as u32,
            "the mapped token follows the rewritten import"
        );
        assert_eq!(
            document.range_to_source_offsets(&LspRange {
                start: LspPosition {
                    line: start.0,
                    character: start.1,
                },
                end: LspPosition {
                    line: end.0,
                    character: end.1,
                },
            }),
            Some((source_start, source_end))
        );
    }
}
