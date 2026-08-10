use tower_lsp::lsp_types::Location;
use vize_canon::CorsaBridge;
use vize_carton::FxHashSet;

use super::ReferencesService;
use crate::ide::{IdeContext, corsa_support};

/// Query the project-aware canonical Vue document before the block-local
/// virtual documents so references can cross SFC boundaries.
pub(super) async fn references(
    ctx: &IdeContext<'_>,
    include_declaration: bool,
    bridge: Option<&CorsaBridge>,
) -> Option<Vec<Location>> {
    let bridge = bridge?;
    if !bridge.is_initialized() {
        return None;
    }
    let document = corsa_support::open_canonical_virtual_project_document(ctx, bridge).await?;
    let (line, character) =
        corsa_support::canonical_source_offset_to_position(&document, ctx.offset)?;
    let mut locations = bridge
        .references(&document.request_uri, line, character, include_declaration)
        .await
        .ok()?;
    let mut linked = linked_positions(&document, &locations);
    if linked.is_empty() && !include_declaration {
        let discovery = bridge
            .references(&document.request_uri, line, character, true)
            .await
            .ok()?;
        linked.extend(linked_positions(&document, &discovery));
    }
    for position in linked {
        let extra = bridge
            .references(
                &position.request_uri,
                position.line,
                position.character,
                include_declaration,
            )
            .await
            .ok()?;
        locations.extend(extra);
    }

    let mut mapped = corsa_support::map_canonical_corsa_locations(ctx, &document, locations);
    mapped.extend(style_locations(ctx, &mapped));
    mapped.sort_by(|left, right| {
        left.uri
            .as_str()
            .cmp(right.uri.as_str())
            .then(left.range.start.line.cmp(&right.range.start.line))
            .then(left.range.start.character.cmp(&right.range.start.character))
            .then(left.range.end.line.cmp(&right.range.end.line))
            .then(left.range.end.character.cmp(&right.range.end.character))
    });
    mapped.dedup_by(|left, right| left.uri == right.uri && left.range == right.range);
    Some(mapped)
}

fn style_locations(ctx: &IdeContext<'_>, semantic: &[Location]) -> Vec<Location> {
    let mut seeds = FxHashSet::default();
    let mut styles = Vec::new();
    collect_style_locations(ctx, ctx.uri, ctx.offset, &mut seeds, &mut styles);

    for location in semantic {
        if !location.uri.path().ends_with(".vue") {
            continue;
        }
        let Some(source) = ctx.state.documents.text(&location.uri) else {
            continue;
        };
        let Some(offset) = crate::ide::position_to_offset(
            &source,
            location.range.start.line,
            location.range.start.character,
        ) else {
            continue;
        };
        collect_style_locations(ctx, &location.uri, offset, &mut seeds, &mut styles);
    }
    styles
}

fn collect_style_locations(
    query: &IdeContext<'_>,
    uri: &tower_lsp::lsp_types::Url,
    offset: usize,
    seeds: &mut FxHashSet<(tower_lsp::lsp_types::Url, vize_carton::String)>,
    locations: &mut Vec<Location>,
) {
    let Some(source) = query.state.documents.text(uri) else {
        return;
    };
    let Some(word) = crate::ide::token_at_offset(&source, offset, |byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
    }) else {
        return;
    };
    if !seeds.insert((uri.clone(), word.clone().into())) {
        return;
    }
    let Some(ctx) = IdeContext::new(query.state, uri, offset) else {
        return;
    };
    locations.extend(ReferencesService::find_references_in_style(&ctx, &word));
}

fn linked_positions(
    document: &corsa_support::CanonicalVirtualDocument,
    locations: &[vize_canon::LspLocation],
) -> FxHashSet<corsa_support::CanonicalSemanticPosition> {
    locations
        .iter()
        .filter_map(|location| {
            corsa_support::linked_semantic_position(document, &location.uri, &location.range)
        })
        .collect()
}
