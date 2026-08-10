use tower_lsp::lsp_types::{Location, Range, Url};
use vize_canon::LspLocation;
use vize_carton::{String, cstr};

use crate::ide::IdeContext;
use crate::ide::diagnostics::VirtualTsResult;

mod open;
mod project;
pub(super) mod rename;
mod semantic_links;

pub(crate) use open::open_canonical_virtual_document;
pub(crate) use project::open_canonical_virtual_project_document;
pub(crate) use rename::{
    map_canonical_corsa_workspace_edit, map_canonical_prepare_rename,
    merge_canonical_workspace_edits,
};
pub(crate) use semantic_links::{CanonicalSemanticPosition, linked_semantic_position, tower_range};

pub(crate) struct CanonicalVirtualDocument {
    pub(crate) request_uri: String,
    pub(crate) virtual_result: VirtualTsResult,
    pub(crate) dependencies: Vec<CanonicalDependencyDocument>,
}

pub(crate) struct CanonicalDependencyDocument {
    pub(crate) source_uri: Url,
    pub(crate) source: String,
    pub(crate) request_uri: String,
    pub(crate) virtual_result: VirtualTsResult,
}

pub(crate) fn canonical_request_path(uri: &Url) -> String {
    cstr!("{}.ts", uri.path())
}

pub(crate) fn canonical_source_offset_to_position(
    doc: &CanonicalVirtualDocument,
    source_offset: usize,
) -> Option<(u32, u32)> {
    let generated_offset = source_offset_to_canonical_generated_offset(doc, source_offset)?;
    Some(crate::ide::offset_to_position(
        &doc.virtual_result.code,
        generated_offset,
    ))
}

fn source_offset_to_canonical_generated_offset(
    doc: &CanonicalVirtualDocument,
    source_offset: usize,
) -> Option<usize> {
    let mapping = mapping_for_source_offset(&doc.virtual_result.source_mappings, source_offset)?;
    let generated_pre_rewrite = map_source_offset_to_generated(mapping, source_offset);
    let generated_post_rewrite = doc
        .virtual_result
        .import_source_map
        .get_virtual_offset(generated_pre_rewrite as u32);
    Some(generated_post_rewrite as usize)
}

fn mapping_for_source_offset(
    mappings: &[vize_canon::virtual_ts::VizeMapping],
    offset: usize,
) -> Option<&vize_canon::virtual_ts::VizeMapping> {
    mappings
        .iter()
        .filter(|mapping| {
            (offset >= mapping.src_range.start && offset <= mapping.src_range.end)
                || mapping
                    .sub_spans
                    .iter()
                    .any(|span| offset >= span.src_range.start && offset <= span.src_range.end)
        })
        .min_by_key(|mapping| {
            mapping
                .sub_spans
                .iter()
                .filter(|span| offset >= span.src_range.start && offset <= span.src_range.end)
                .map(|span| span.src_range.end.saturating_sub(span.src_range.start))
                .min()
                .unwrap_or_else(|| {
                    mapping
                        .src_range
                        .end
                        .saturating_sub(mapping.src_range.start)
                })
        })
}

fn map_source_offset_to_generated(
    mapping: &vize_canon::virtual_ts::VizeMapping,
    source_offset: usize,
) -> usize {
    if let Some(span) = mapping
        .sub_spans
        .iter()
        .find(|span| source_offset >= span.src_range.start && source_offset <= span.src_range.end)
    {
        let relative = source_offset.saturating_sub(span.src_range.start);
        return span
            .gen_range
            .start
            .saturating_add(relative.min(span.gen_range.end.saturating_sub(span.gen_range.start)));
    }

    let relative = source_offset.saturating_sub(mapping.src_range.start);
    mapping.gen_range.start.saturating_add(
        relative.min(
            mapping
                .gen_range
                .end
                .saturating_sub(mapping.gen_range.start),
        ),
    )
}

pub(crate) fn map_canonical_corsa_locations(
    ctx: &IdeContext<'_>,
    doc: &CanonicalVirtualDocument,
    locations: Vec<LspLocation>,
) -> Vec<Location> {
    locations
        .iter()
        .filter_map(|location| map_canonical_corsa_location(ctx, doc, location))
        .collect()
}

pub(crate) fn map_canonical_corsa_location(
    ctx: &IdeContext<'_>,
    doc: &CanonicalVirtualDocument,
    location: &LspLocation,
) -> Option<Location> {
    if location_matches_uri(&location.uri, doc.request_uri.as_str())
        || super::virtual_document_path(&location.uri).as_deref()
            == Some(canonical_request_path(ctx.uri).as_str())
    {
        let range = map_canonical_lsp_range(ctx, doc, &location.range)?;
        return Some(Location {
            uri: ctx.uri.clone(),
            range,
        });
    }

    if let Some(dependency) = doc
        .dependencies
        .iter()
        .find(|dependency| location_matches_uri(&location.uri, &dependency.request_uri))
    {
        let range = map_virtual_result_lsp_range_to_source(
            &dependency.source,
            &dependency.virtual_result,
            &location.range,
        )?;
        return Some(Location {
            uri: dependency.source_uri.clone(),
            range,
        });
    }

    if let Some(location) = super::external_mirror::map_location(ctx, location) {
        return Some(location);
    }

    let uri = super::accessible_external_uri(ctx, &location.uri)?;
    Some(Location {
        uri,
        range: Range {
            start: tower_lsp::lsp_types::Position {
                line: location.range.start.line,
                character: location.range.start.character,
            },
            end: tower_lsp::lsp_types::Position {
                line: location.range.end.line,
                character: location.range.end.character,
            },
        },
    })
}

fn is_canonical_vue_virtual_uri(uri: &Url) -> bool {
    uri.path().ends_with(".vue.ts") || uri.path().ends_with(".vue.tsx")
}

pub(crate) fn map_canonical_lsp_range(
    ctx: &IdeContext<'_>,
    doc: &CanonicalVirtualDocument,
    range: &vize_canon::LspRange,
) -> Option<Range> {
    map_lsp_range_to_source(&ctx.content, doc, range)
}

pub(super) fn map_lsp_range_to_source(
    source: &str,
    doc: &CanonicalVirtualDocument,
    range: &vize_canon::LspRange,
) -> Option<Range> {
    map_virtual_result_lsp_range_to_source(source, &doc.virtual_result, range)
}

fn map_virtual_result_lsp_range_to_source(
    source: &str,
    virtual_result: &VirtualTsResult,
    range: &vize_canon::LspRange,
) -> Option<Range> {
    let generated_start_post = crate::ide::position_to_offset(
        &virtual_result.code,
        range.start.line,
        range.start.character,
    )?;
    let generated_end_post =
        crate::ide::position_to_offset(&virtual_result.code, range.end.line, range.end.character)
            .unwrap_or(generated_start_post);

    let generated_start_pre = virtual_result
        .import_source_map
        .get_original_offset(generated_start_post as u32) as usize;
    let generated_end_pre = virtual_result
        .import_source_map
        .get_original_offset(generated_end_post as u32) as usize;

    let start_mapping =
        mapping_for_generated_offset(&virtual_result.source_mappings, generated_start_pre)?;
    let source_start = map_generated_offset_to_source(start_mapping, generated_start_pre, false);
    let source_end =
        mapping_for_generated_offset(&virtual_result.source_mappings, generated_end_pre)
            .map(|mapping| map_generated_offset_to_source(mapping, generated_end_pre, true))
            .unwrap_or_else(|| {
                source_start
                    .saturating_add(generated_end_pre.saturating_sub(generated_start_pre))
                    .min(start_mapping.src_range.end)
            })
            .max(source_start);

    let (start_line, start_character) = crate::ide::offset_to_position(source, source_start);
    let (end_line, end_character) = crate::ide::offset_to_position(source, source_end);

    Some(Range {
        start: tower_lsp::lsp_types::Position {
            line: start_line,
            character: start_character,
        },
        end: tower_lsp::lsp_types::Position {
            line: end_line,
            character: end_character,
        },
    })
}

fn mapping_for_generated_offset(
    mappings: &[vize_canon::virtual_ts::VizeMapping],
    offset: usize,
) -> Option<&vize_canon::virtual_ts::VizeMapping> {
    mappings
        .iter()
        .filter(|mapping| offset >= mapping.gen_range.start && offset <= mapping.gen_range.end)
        .min_by_key(|mapping| {
            mapping
                .gen_range
                .end
                .saturating_sub(mapping.gen_range.start)
        })
}

fn map_generated_offset_to_source(
    mapping: &vize_canon::virtual_ts::VizeMapping,
    generated_offset: usize,
    prefer_end: bool,
) -> usize {
    if let Some(span) = mapping.sub_spans.iter().find(|span| {
        generated_offset >= span.gen_range.start && generated_offset <= span.gen_range.end
    }) {
        let relative = generated_offset.saturating_sub(span.gen_range.start);
        let source_len = span.src_range.end.saturating_sub(span.src_range.start);
        return span
            .src_range
            .start
            .saturating_add(relative.min(source_len));
    }

    if prefer_end && generated_offset >= mapping.gen_range.end {
        return mapping.src_range.end;
    }

    let relative = generated_offset.saturating_sub(mapping.gen_range.start);
    let source_len = mapping
        .src_range
        .end
        .saturating_sub(mapping.src_range.start);
    mapping
        .src_range
        .start
        .saturating_add(relative.min(source_len))
}

fn location_matches_uri(actual: &str, expected: &str) -> bool {
    actual == expected
        || super::virtual_document_path(actual).as_deref()
            == super::virtual_document_path(expected).as_deref()
}

#[cfg(test)]
mod tests {
    use super::{map_source_offset_to_generated, mapping_for_source_offset};
    use vize_canon::virtual_ts::{VizeMapping, VizeSubSpan};

    #[test]
    fn source_mapping_selects_a_value_sub_span_outside_the_parent_source_range() {
        let mappings = [VizeMapping {
            gen_range: 100..140,
            src_range: 10..20,
            sub_spans: vec![VizeSubSpan {
                gen_range: 124..129,
                src_range: 30..35,
            }],
        }];

        let mapping = mapping_for_source_offset(&mappings, 32).expect("value sub-span mapping");
        assert_eq!(map_source_offset_to_generated(mapping, 32), 126);
    }
}
