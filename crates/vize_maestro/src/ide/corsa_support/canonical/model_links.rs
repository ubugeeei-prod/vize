//! Follow native identities owned by one retained authored model declaration.

use std::ops::Range;

use vize_croquis::{Drawer, DrawerOptions};

use super::{CanonicalSemanticPosition, CanonicalVirtualDocument};
use crate::ide::IdeContext;
use crate::ide::diagnostics::VirtualTsResult;

pub(crate) fn model_linked_semantic_positions(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    uri: &str,
    range: &vize_canon::LspRange,
) -> Vec<CanonicalSemanticPosition> {
    let Some((request_uri, source_uri, source, result)) = virtual_source(ctx, document, uri) else {
        return Vec::new();
    };
    let Some(authored) =
        super::mapping::map_virtual_result_lsp_range_to_source(source, result, range)
    else {
        return Vec::new();
    };
    let Some(start) =
        crate::ide::position_to_offset(source, authored.start.line, authored.start.character)
    else {
        return Vec::new();
    };
    let Some(end) =
        crate::ide::position_to_offset(source, authored.end.line, authored.end.character)
    else {
        return Vec::new();
    };
    let authored = start..end;
    // Ordinary references keep their existing scan-only fast path. A retained
    // model event row must own this native location before consulting the
    // existing authored macro declaration scanner.
    if !has_model_event(result, &authored) {
        return Vec::new();
    }
    let Some(declaration) = model_declaration_ranges(source, source_uri.path())
        .into_iter()
        .find(|declaration| owns_declaration(source, declaration, &authored))
    else {
        return Vec::new();
    };
    super::mapping::source_range_to_virtual_positions(result, source, &declaration)
        .into_iter()
        .filter(|&(line, character)| {
            direct_model_position(result, source, &declaration, line, character)
        })
        .map(|(line, character)| CanonicalSemanticPosition {
            request_uri: request_uri.clone(),
            line,
            character,
        })
        .collect()
}

fn direct_model_position(
    result: &VirtualTsResult,
    source: &str,
    declaration: &Range<usize>,
    line: u32,
    character: u32,
) -> bool {
    let Some(post) = crate::ide::position_to_offset(&result.code, line, character) else {
        return false;
    };
    let pre = result.import_source_map.get_original_offset(post as u32) as usize;
    // A broad setup mapping can round-trip the defineModel string argument,
    // but that coordinate is not the projected prop or update identity. The
    // synthesized update prefix likewise has no authored name counterpart.
    result.source_mappings.iter().any(|mapping| {
        mapping.sub_spans.is_empty()
            && mapping.gen_range.len() == mapping.src_range.len()
            && owns_declaration(source, declaration, &mapping.src_range)
            && mapping.gen_range.contains(&pre)
    })
}

fn owns_declaration(source: &str, declaration: &Range<usize>, authored: &Range<usize>) -> bool {
    if declaration == authored {
        return true;
    }
    let Some(start) = declaration.start.checked_sub(1) else {
        return false;
    };
    let Some(end) = declaration.end.checked_add(1) else {
        return false;
    };
    let outer = start..end;
    outer == *authored
        && source.get(outer).is_some_and(
            |text| matches!(text.as_bytes(), [open @ (b'\'' | b'"'), .., close] if open == close),
        )
}

fn has_model_event(result: &VirtualTsResult, authored: &Range<usize>) -> bool {
    result.source_mappings.iter().any(|mapping| {
        let owns_range = mapping.src_range == *authored
            || (mapping.src_range.start.checked_add(1) == Some(authored.start)
                && mapping.src_range.end.checked_sub(1) == Some(authored.end))
            || (mapping.src_range.start.checked_sub(1) == Some(authored.start)
                && mapping.src_range.end.checked_add(1) == Some(authored.end));
        if !owns_range {
            return false;
        }
        let generated = result
            .import_source_map
            .get_virtual_offset(mapping.gen_range.start as u32) as usize;
        result
            .code
            .get(..generated)
            .is_some_and(|prefix| prefix.ends_with("  /* __vize_model_event */ "))
    })
}

fn virtual_source<'a>(
    ctx: &'a IdeContext<'_>,
    document: &'a CanonicalVirtualDocument,
    uri: &str,
) -> Option<(
    &'a vize_l0::String,
    &'a tower_lsp::lsp_types::Url,
    &'a str,
    &'a VirtualTsResult,
)> {
    if uri == document.request_uri {
        return Some((
            &document.request_uri,
            ctx.uri,
            &ctx.content,
            &document.virtual_result,
        ));
    }
    document
        .dependencies
        .iter()
        .find(|dependency| uri == dependency.request_uri)
        .map(|dependency| {
            (
                &dependency.request_uri,
                &dependency.source_uri,
                dependency.source.as_str(),
                &dependency.virtual_result,
            )
        })
        .or_else(|| {
            document
                .materialized_sources
                .iter()
                .find(|source| {
                    uri == source.request_uri
                        && source.mapping_kind
                            == vize_canon::CorsaMaterializedMappingKind::Generated
                })
                .map(|source| {
                    (
                        &source.request_uri,
                        &source.source_uri,
                        source.source.as_str(),
                        &source.virtual_result,
                    )
                })
        })
}

pub(crate) fn model_declaration_ranges(source: &str, filename: &str) -> Vec<Range<usize>> {
    let Some(descriptor) = vize_incremental::parse_descriptor(filename, source) else {
        return Vec::new();
    };
    let Some(script) = descriptor.script_setup.as_ref() else {
        return Vec::new();
    };
    let mut drawer = Drawer::with_options(DrawerOptions {
        analyze_script: true,
        ..Default::default()
    });
    drawer.analyze_script_setup(&script.content);
    let croquis = drawer.finish();
    croquis
        .macros
        .models()
        .iter()
        .filter_map(|model| {
            let (start, end) = croquis.macros.model_declaration(model.name.as_str())?;
            let mut range = script.loc.start + start as usize..script.loc.start + end as usize;
            let authored = source.get(range.clone())?;
            let unquoted = authored
                .strip_prefix(['\'', '"'])
                .and_then(|name| name.strip_suffix(['\'', '"']))
                .unwrap_or(authored);
            (unquoted == model.name).then(|| {
                if unquoted.len() != authored.len() {
                    range.start += 1;
                    range.end -= 1;
                }
                range
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
