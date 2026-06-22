use std::sync::Arc;

use tower_lsp::lsp_types::{Location, Range, Url};
use vize_canon::{CorsaBridge, LspLocation};
use vize_carton::{String, cstr};

use crate::ide::IdeContext;
use crate::ide::diagnostics::VirtualTsResult;

pub(crate) struct CanonicalVirtualDocument {
    pub(crate) request_uri: String,
    pub(crate) virtual_result: VirtualTsResult,
}

pub(crate) fn canonical_request_path(uri: &Url) -> String {
    cstr!("{}.ts", uri.path())
}

pub(crate) async fn open_canonical_virtual_document(
    ctx: &IdeContext<'_>,
    bridge: &Arc<CorsaBridge>,
) -> Option<CanonicalVirtualDocument> {
    if !ctx.uri.path().ends_with(".vue") || ctx.uri.path().ends_with(".art.vue") {
        return None;
    }

    let options_api = ctx.state.options_api_enabled();
    let legacy_vue2 = ctx.state.legacy_vue2_enabled();
    let virtual_result = crate::ide::DiagnosticService::generate_virtual_ts(
        ctx.uri,
        &ctx.content,
        options_api,
        legacy_vue2,
    )?;

    crate::ide::diagnostics::corsa::collect::overlay_sibling_vue_mirrors(
        bridge,
        ctx.uri,
        &virtual_result.relative_vue_imports,
        options_api,
        legacy_vue2,
    )
    .await;
    crate::ide::diagnostics::corsa::collect::overlay_relative_ts_imports(
        bridge,
        ctx.uri,
        &virtual_result.relative_ts_imports,
    )
    .await;

    let request_uri = bridge
        .open_or_update_virtual_document(&canonical_request_path(ctx.uri), &virtual_result.code)
        .await
        .ok()?;

    Some(CanonicalVirtualDocument {
        request_uri,
        virtual_result,
    })
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
        .filter(|mapping| offset >= mapping.src_range.start && offset <= mapping.src_range.end)
        .min_by_key(|mapping| {
            mapping
                .src_range
                .end
                .saturating_sub(mapping.src_range.start)
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

    if let Some(location) = map_vue_virtual_mirror_location(location) {
        return Some(location);
    }

    let uri = Url::parse(&location.uri).ok()?;
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

pub(crate) fn map_canonical_lsp_range(
    ctx: &IdeContext<'_>,
    doc: &CanonicalVirtualDocument,
    range: &vize_canon::LspRange,
) -> Option<Range> {
    let generated_start_post = crate::ide::position_to_offset(
        &doc.virtual_result.code,
        range.start.line,
        range.start.character,
    )?;
    let generated_end_post = crate::ide::position_to_offset(
        &doc.virtual_result.code,
        range.end.line,
        range.end.character,
    )
    .unwrap_or(generated_start_post);

    let generated_start_pre = doc
        .virtual_result
        .import_source_map
        .get_original_offset(generated_start_post as u32) as usize;
    let generated_end_pre = doc
        .virtual_result
        .import_source_map
        .get_original_offset(generated_end_post as u32) as usize;

    let start_mapping =
        mapping_for_generated_offset(&doc.virtual_result.source_mappings, generated_start_pre)?;
    let source_start = map_generated_offset_to_source(start_mapping, generated_start_pre, false);
    let source_end =
        mapping_for_generated_offset(&doc.virtual_result.source_mappings, generated_end_pre)
            .map(|mapping| map_generated_offset_to_source(mapping, generated_end_pre, true))
            .unwrap_or_else(|| {
                source_start
                    .saturating_add(generated_end_pre.saturating_sub(generated_start_pre))
                    .min(start_mapping.src_range.end)
            })
            .max(source_start);

    let (start_line, start_character) = crate::ide::offset_to_position(&ctx.content, source_start);
    let (end_line, end_character) = crate::ide::offset_to_position(&ctx.content, source_end);

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

fn map_vue_virtual_mirror_location(location: &LspLocation) -> Option<Location> {
    let parsed = Url::parse(&location.uri).ok()?;
    let path = parsed.to_file_path().ok()?;
    let file_name = path.file_name()?.to_str()?;
    let vue_file_name = file_name.strip_suffix(".ts")?;
    if !vue_file_name.ends_with(".vue") {
        return None;
    }

    let source_path = path.with_file_name(vue_file_name);
    if !source_path.is_file() {
        return None;
    }

    let uri = Url::from_file_path(source_path).ok()?;
    Some(Location {
        uri,
        range: Range {
            start: tower_lsp::lsp_types::Position {
                line: 0,
                character: 0,
            },
            end: tower_lsp::lsp_types::Position {
                line: 0,
                character: 0,
            },
        },
    })
}
