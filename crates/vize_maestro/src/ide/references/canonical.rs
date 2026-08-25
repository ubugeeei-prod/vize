use tower_lsp::lsp_types::Location;
use vize_canon::CorsaBridge;
use vize_carton::FxHashSet;
use vize_croquis::{Drawer, DrawerOptions};
use vize_relief::BindingType;

use super::ReferencesService;
use crate::ide::{IdeContext, corsa_support};

/// Whether the queried word is a top-level `<script setup>` binding declared
/// in this SFC. Script-setup declarations cannot be imported by other
/// modules, so their canonical reference surface is the current document plus
/// its open importers — never the rest of the workspace. Imported names keep
/// the workspace surface: croquis classifies them with the same `Setup*`
/// binding types, so an import lookup separates the two.
fn is_script_setup_local_binding(ctx: &IdeContext<'_>) -> bool {
    let Some(word) = ReferencesService::get_word_at_offset(&ctx.content, ctx.offset) else {
        return false;
    };
    if crate::ide::definition::helpers::find_import_path(ctx, &word).is_some() {
        return false;
    }
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
        return false;
    };
    let Some(script_setup) = descriptor.script_setup else {
        return false;
    };
    let mut analyzer = Drawer::with_options(DrawerOptions::full());
    analyzer.analyze_script_setup(&script_setup.content);
    matches!(
        analyzer.finish().get_binding_type(&word),
        Some(
            BindingType::SetupLet
                | BindingType::SetupMaybeRef
                | BindingType::SetupRef
                | BindingType::SetupReactiveConst
                | BindingType::SetupConst
        )
    )
}

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
    // A top-level `<script setup>` binding is invisible to other modules, so
    // its references live in this SFC and the already-open project surface;
    // materializing every workspace SFC for it takes minutes on a
    // component-library-sized workspace and cannot add hits.
    let document = if is_script_setup_local_binding(ctx) {
        corsa_support::open_canonical_virtual_project_document_strict(ctx, bridge)
            .await
            .ok()
            .flatten()?
    } else {
        corsa_support::open_canonical_virtual_workspace_document(ctx, bridge).await?
    };
    let (line, character) =
        corsa_support::canonical_source_offset_to_position(&document, ctx.offset)?;
    let mut locations = bridge
        .references(&document.request_uri, line, character, include_declaration)
        .await
        .ok()?;
    locations.extend(
        component_prop_references(ctx, bridge, &document, line, character, include_declaration)
            .await,
    );
    let mut linked = linked_positions(&document, &locations);
    linked.extend(corsa_support::materialized_semantic_positions(
        &document, ctx.uri, ctx.offset,
    ));
    linked.remove(&corsa_support::CanonicalSemanticPosition {
        request_uri: document.request_uri.clone(),
        line,
        character,
    });
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
    mapped.extend(style_locations(ctx, &document, &mapped));
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

async fn component_prop_references(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
    document: &corsa_support::CanonicalVirtualDocument,
    line: u32,
    character: u32,
    include_declaration: bool,
) -> Vec<vize_canon::LspLocation> {
    let mut matches = corsa_support::matching_component_prop_navigation_positions(
        ctx,
        bridge,
        document,
        &document.request_uri,
        line,
        character,
    )
    .await;
    if matches.names.is_empty() {
        return Vec::new();
    }

    let queries = matches
        .positions
        .iter()
        .map(|position| {
            (
                position.request_uri.as_str(),
                position.line,
                position.character,
            )
        })
        .collect::<Vec<_>>();
    let Ok(batches) = bridge.references_batch(&queries, include_declaration).await else {
        return Vec::new();
    };
    let mut references = Vec::new();
    let names = &matches.names;
    let source_cache = &mut matches.source_cache;
    for extra in batches {
        references.extend(extra.into_iter().filter(|location| {
            let Some(authored) =
                corsa_support::map_canonical_corsa_location(ctx, document, location)
            else {
                return false;
            };
            corsa_support::component_prop_location_matches(
                ctx,
                document,
                &authored,
                names,
                source_cache,
            )
        }));
    }
    references
}

fn style_locations(
    ctx: &IdeContext<'_>,
    document: &corsa_support::CanonicalVirtualDocument,
    semantic: &[Location],
) -> Vec<Location> {
    let mut seeds = FxHashSet::default();
    let mut styles = Vec::new();
    collect_style_locations(
        ctx,
        ctx.uri,
        &ctx.content,
        ctx.offset,
        &mut seeds,
        &mut styles,
    );

    for location in semantic {
        if !location.uri.path().ends_with(".vue") {
            continue;
        }
        let Some(source) = ctx
            .state
            .documents
            .text(&location.uri)
            .or_else(|| document.authored_source(&location.uri).map(str::to_owned))
        else {
            continue;
        };
        let Some(offset) = crate::ide::position_to_offset(
            &source,
            location.range.start.line,
            location.range.start.character,
        ) else {
            continue;
        };
        collect_style_locations(ctx, &location.uri, &source, offset, &mut seeds, &mut styles);
    }
    styles
}

fn collect_style_locations(
    query: &IdeContext<'_>,
    uri: &tower_lsp::lsp_types::Url,
    source: &str,
    offset: usize,
    seeds: &mut FxHashSet<(tower_lsp::lsp_types::Url, vize_carton::String)>,
    locations: &mut Vec<Location>,
) {
    let Some(word) = crate::ide::token_at_offset(source, offset, |byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
    }) else {
        return;
    };
    if !seeds.insert((uri.clone(), word.clone().into())) {
        return;
    }
    let ctx = IdeContext::with_content(query.state, uri, offset, source.to_owned());
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
