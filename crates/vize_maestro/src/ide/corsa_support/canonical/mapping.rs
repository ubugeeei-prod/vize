//! Bidirectional authored/generated coordinate mapping for Canon documents.

use tower_lsp::lsp_types::Range;

use super::CanonicalMaterializedSource;
use super::CanonicalVirtualDocument;
use crate::ide::IdeContext;
use crate::ide::diagnostics::VirtualTsResult;

pub(crate) fn canonical_source_offset_to_position(
    doc: &CanonicalVirtualDocument,
    source_offset: usize,
) -> Option<(u32, u32)> {
    let generated_offset =
        source_offset_to_virtual_generated_offset(&doc.virtual_result, source_offset)?;
    Some(crate::ide::offset_to_position(
        &doc.virtual_result.code,
        generated_offset,
    ))
}

pub(super) fn source_offset_to_virtual_generated_offset(
    virtual_result: &VirtualTsResult,
    source_offset: usize,
) -> Option<usize> {
    let mapping = mapping_for_source_offset(&virtual_result.source_mappings, source_offset)?;
    let generated_pre_rewrite = map_source_offset_to_generated(mapping, source_offset);
    Some(
        virtual_result
            .import_source_map
            .get_virtual_offset(generated_pre_rewrite as u32) as usize,
    )
}

pub(super) fn materialized_source_offset_to_generated_offset(
    source: &CanonicalMaterializedSource,
    source_offset: usize,
) -> Option<usize> {
    match source.mapping_kind {
        vize_canon::CorsaMaterializedMappingKind::Generated => {
            source_offset_to_virtual_generated_offset(&source.virtual_result, source_offset)
        }
        vize_canon::CorsaMaterializedMappingKind::AuthoredIdentity => (source_offset
            <= source.source.len()
            && source_offset <= source.virtual_result.code.len())
        .then_some(source_offset),
        vize_canon::CorsaMaterializedMappingKind::Synthetic => None,
    }
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

pub(crate) fn map_canonical_lsp_range(
    ctx: &IdeContext<'_>,
    doc: &CanonicalVirtualDocument,
    range: &vize_canon::LspRange,
) -> Option<Range> {
    map_lsp_range_to_source(&ctx.content, doc, range)
}

pub(crate) fn map_lsp_range_to_source(
    source: &str,
    doc: &CanonicalVirtualDocument,
    range: &vize_canon::LspRange,
) -> Option<Range> {
    map_virtual_result_lsp_range_to_source(source, &doc.virtual_result, range)
}

pub(crate) fn map_virtual_result_lsp_range_to_source(
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

    if virtual_result.source_mappings.is_empty() {
        return source_range(source, generated_start_pre, generated_end_pre);
    }
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
    source_range(source, source_start, source_end)
}

fn source_range(source: &str, start: usize, end: usize) -> Option<Range> {
    let start = start.min(source.len());
    let end = end.max(start).min(source.len());
    let (start_line, start_character) = crate::ide::offset_to_position(source, start);
    let (end_line, end_character) = crate::ide::offset_to_position(source, end);
    Some(Range::new(
        tower_lsp::lsp_types::Position::new(start_line, start_character),
        tower_lsp::lsp_types::Position::new(end_line, end_character),
    ))
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
