//! Bidirectional authored/generated coordinate mapping for Canon documents.

use tower_lsp::lsp_types::Range;
use vize_l0::line_index::LineBreaks;

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
    Some(LineBreaks::Lsp.offset_to_position(&doc.virtual_result.code, generated_offset))
}

/// [`canonical_source_offset_to_position`] for symbol operations (rename,
/// references). A static attribute value (`ref="box"`, `id="note"`) names no
/// script symbol; a coarse mapping that merely covers it would land on
/// whatever its generated range holds, and rename would edit that symbol.
pub(crate) fn canonical_symbol_position(
    doc: &CanonicalVirtualDocument,
    source: &str,
    source_offset: usize,
) -> Option<(u32, u32)> {
    if in_static_attribute_value(source, source_offset) {
        return None;
    }
    canonical_source_offset_to_position(doc, source_offset)
}

/// Whether `offset` is inside the quoted value of a plain (non-directive)
/// attribute of the start tag that contains it.
/// The start of the start tag that contains `offset`, read forward from the
/// template so a `<` inside a quoted attribute value (an expression such as
/// `` :title="`<b>${x}</b>`" ``) never opens a tag.
fn containing_tag_start(source: &str, offset: usize) -> Option<usize> {
    let before = source.get(..offset)?;
    let from = before.rfind("<template").unwrap_or(0);
    let bytes = before.as_bytes();
    let (mut open, mut quote) = (None, None::<u8>);
    let mut index = from;
    while let Some(&byte) = bytes.get(index) {
        match (open, quote) {
            (Some(_), Some(active)) if byte == active => quote = None,
            (Some(_), Some(_)) => {}
            (Some(_), None) if matches!(byte, b'"' | b'\'') => quote = Some(byte),
            (Some(_), None) if byte == b'>' => open = None,
            (None, _)
                if byte == b'<'
                    && bytes
                        .get(index + 1)
                        .is_some_and(|next| next.is_ascii_alphabetic()) =>
            {
                open = Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    open
}

pub(crate) fn in_static_attribute_value(source: &str, offset: usize) -> bool {
    let Some(tag_start) = containing_tag_start(source, offset) else {
        return false;
    };
    let Some(tag) = source.get(tag_start..) else {
        return false;
    };
    let bytes = tag.as_bytes();
    let byte = |index: usize| bytes.get(index).copied();
    let space = |value: u8| matches!(value, b' ' | b'\t' | b'\n' | b'\r');
    let target = offset - tag_start;
    // Skip the tag name, then walk `name[=value]` pairs.
    let mut index = 1;
    while byte(index).is_some_and(|value| !space(value) && value != b'>' && value != b'/') {
        index += 1;
    }
    let tag_name = tag.get(1..index).unwrap_or_default();
    loop {
        while byte(index).is_some_and(|value| space(value) || value == b'/') {
            index += 1;
        }
        if byte(index).is_none_or(|value| value == b'>') || target <= index {
            return false;
        }
        let name_start = index;
        while byte(index).is_some_and(|value| !space(value) && !matches!(value, b'=' | b'>' | b'/'))
        {
            index += 1;
        }
        let name = tag.get(name_start..index).unwrap_or_default();
        if byte(index) != Some(b'=') {
            continue;
        }
        index += 1;
        let Some(quote) = byte(index).filter(|quote| matches!(quote, b'"' | b'\'')) else {
            continue;
        };
        let value_start = index + 1;
        let Some(length) = tag
            .get(value_start..)
            .and_then(|rest| rest.find(char::from(quote)))
        else {
            return false;
        };
        let value_end = value_start + length;
        if (value_start..=value_end).contains(&target) {
            // `<slot name="heading">` declares a slot: renaming it is a
            // symbol rename.
            return !name.is_empty()
                && !name.starts_with([':', '@', '#', '.'])
                && !name.starts_with("v-")
                && !(tag_name == "slot" && name == "name");
        }
        index = value_end + 1;
    }
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

/// Public declarations can be emitted both in the exported component type and
/// in the original setup statement. Keep every exact reverse-mapped identity;
/// selecting only the narrowest row loses native destructure property edits.
pub(super) fn source_range_to_virtual_positions(
    result: &VirtualTsResult,
    source: &str,
    authored_range: &std::ops::Range<usize>,
) -> Vec<(u32, u32)> {
    if authored_range.start >= authored_range.end || authored_range.end > source.len() {
        return Vec::new();
    }
    let Some(expected) = source_range(source, authored_range.start, authored_range.end) else {
        return Vec::new();
    };
    let mut positions = Vec::new();
    for mapping in &result.source_mappings {
        let contains = |range: &std::ops::Range<usize>| {
            range.start <= authored_range.start && authored_range.end <= range.end
        };
        if !contains(&mapping.src_range)
            && !mapping
                .sub_spans
                .iter()
                .any(|span| contains(&span.src_range))
        {
            continue;
        }
        let start = result
            .import_source_map
            .get_virtual_offset(
                map_source_offset_to_generated(mapping, authored_range.start) as u32,
            );
        let end = result
            .import_source_map
            .get_virtual_offset(map_source_offset_to_generated(mapping, authored_range.end) as u32);
        let (line, character) = LineBreaks::Lsp.offset_to_position(&result.code, start as usize);
        let (end_line, end_character) =
            LineBreaks::Lsp.offset_to_position(&result.code, end as usize);
        let range = vize_canon::LspRange {
            start: vize_canon::LspPosition { line, character },
            end: vize_canon::LspPosition {
                line: end_line,
                character: end_character,
            },
        };
        if map_virtual_result_lsp_range_to_source(source, result, &range)
            .is_some_and(|mapped| mapped == expected)
        {
            positions.push((line, character));
        }
    }
    positions.sort_unstable();
    positions.dedup();
    positions
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

#[cfg(test)]
mod component_tag_tests;

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
                .map(|span| (span.src_range.len(), span.gen_range.len()))
                .min()
                .unwrap_or_else(|| (mapping.src_range.len(), mapping.gen_range.len()))
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
    let generated_start_post = LineBreaks::Lsp.position_to_offset(
        &virtual_result.code,
        range.start.line,
        range.start.character,
    )?;
    let generated_end_post = LineBreaks::Lsp
        .position_to_offset(&virtual_result.code, range.end.line, range.end.character)
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
    if prefer_end {
        vize_canon::virtual_ts::mapping::map_generated_end_offset_to_source(
            mapping,
            generated_offset,
        )
    } else {
        vize_canon::virtual_ts::mapping::map_generated_offset_to_source(mapping, generated_offset)
    }
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

    #[test]
    fn exact_property_key_beats_a_receiver_rewrite_of_the_same_source() {
        // Both map the authored `class`, but only the second points at the
        // property key. The broad initializer begins with a synthetic receiver.
        let broad = VizeMapping {
            gen_range: 100..180,
            src_range: 10..15,
            sub_spans: vec![VizeSubSpan {
                gen_range: 124..156,
                src_range: 30..35,
            }],
        };
        let key = VizeMapping {
            gen_range: 149..154,
            src_range: 30..35,
            sub_spans: vec![],
        };
        for mappings in [[broad.clone(), key.clone()], [key.clone(), broad.clone()]] {
            let mapping = mapping_for_source_offset(&mappings, 32).expect("property key");
            assert_eq!(map_source_offset_to_generated(mapping, 32), 151);
        }
    }
    #[test]
    fn public_declaration_keeps_exported_and_setup_native_identities() {
        use crate::ide::DiagnosticService;
        use tower_lsp::lsp_types::Url;

        let source = "<script setup lang=\"ts\">\nconst { label } = defineProps<{ label: string }>();\n</script>";
        let uri = Url::parse("file:///workspace/Child.vue").expect("URI");
        let result = DiagnosticService::generate_virtual_ts(&uri, source, false, false)
            .expect("virtual document");
        let start = source.rfind("label").expect("public declaration");
        let positions =
            super::source_range_to_virtual_positions(&result, source, &(start..start + 5));
        assert!(
            positions.len() >= 2,
            "all public identities: {positions:?}; {}",
            result.code
        );
        for (line, character) in positions {
            let offset = crate::ide::position_to_offset(&result.code, line, character)
                .expect("generated position");
            assert_eq!(result.code.get(offset..offset + 5), Some("label"));
        }
        assert!(
            super::source_range_to_virtual_positions(
                &result,
                source,
                &(source.len()..source.len() + 1)
            )
            .is_empty()
        );
    }
}

#[cfg(test)]
mod static_attribute_tests {
    use super::in_static_attribute_value;

    #[test]
    fn only_plain_attribute_values_name_no_symbol() {
        let source =
            r#"<textarea v-else ref="box" :title="msg" @focus="go" id='note'>{{ msg }}</textarea>"#;
        let at = |needle: &str| source.find(needle).unwrap() + 1;
        assert_eq!(
            [
                at("box"),
                at("note"),
                at("msg\""),
                at("go\""),
                at("msg }}"),
                at("textarea v")
            ]
            .map(|offset| in_static_attribute_value(source, offset)),
            [true, true, false, false, false, false]
        );
        let expression = r#"<template><b :title="`<span id='${box}'>`" id="note" /></template>"#;
        let at_expression = |needle: &str| expression.find(needle).unwrap() + 1;
        assert_eq!(
            [at_expression("box"), at_expression("note")]
                .map(|offset| in_static_attribute_value(expression, offset)),
            [false, true]
        );
        let slot = r#"<slot name="heading" title="Hello" />"#;
        let at = |needle: &str| slot.find(needle).unwrap() + 1;
        assert_eq!(
            [at("heading"), at("Hello")].map(|offset| in_static_attribute_value(slot, offset)),
            [false, true]
        );
    }
}
