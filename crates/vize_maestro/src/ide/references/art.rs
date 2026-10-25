//! Join independent variant projections through native binding occurrences in
//! their shared physical script. Template-local names never create this edge.

use tower_lsp::lsp_types::Location;
use vize_canon::CorsaBridge;
use vize_l0::String;

use crate::ide::{IdeContext, corsa_support, offset_to_position, position_to_offset};
use crate::virtual_code::{ArtVariantInfo, BlockType};

#[derive(Clone, Copy)]
enum Template {
    Art(usize),
    Ordinary,
}

pub(super) async fn variant_references(
    ctx: &IdeContext<'_>,
    info: &ArtVariantInfo,
    include_declaration: bool,
    bridge: Option<&CorsaBridge>,
) -> Option<Vec<Location>> {
    let bridge = bridge.filter(|bridge| bridge.is_initialized())?;
    let documents = ctx.virtual_docs.as_ref()?;
    let template = documents.art_template(info.variant_index)?;
    let offset = template.source_map.to_generated(ctx.offset).or_else(|| {
        let (start, _) = crate::ide::token_span_at_offset(&ctx.content, ctx.offset, |byte| {
            byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
        })?;
        template.source_map.to_generated(start)
    })?;
    let (line, character) = offset_to_position(&template.content, offset);
    // Refresh every variant before a query can return imported-symbol hits
    // from a previously opened projection at another buffer revision.
    let opened = synchronize(ctx, bridge).await?;
    let (_, uri) = opened.iter().find(
        |(template, _)| matches!(template, Template::Art(index) if *index == info.variant_index),
    )?;
    // Discovery retains the script declaration even when the client excludes
    // it from the final answer. Native identity, not spelling, chooses a join.
    let discovered = bridge.references(uri, line, character, true).await.ok()?;
    let mapped = corsa_support::map_corsa_locations(ctx, discovered);
    let anchor = script_anchor(ctx, &mapped);
    let selected = if include_declaration {
        mapped
    } else {
        corsa_support::map_corsa_locations(
            ctx,
            bridge.references(uri, line, character, false).await.ok()?,
        )
    };
    let extra = match anchor {
        Some(anchor) => {
            shared_at(
                ctx,
                anchor,
                include_declaration,
                bridge,
                Some(info.variant_index),
                &opened,
            )
            .await
        }
        None => None,
    };
    corsa_support::merge_canonical_locations(Some(selected), extra, None)
}

pub(super) async fn shared_script_references(
    ctx: &IdeContext<'_>,
    include_declaration: bool,
    bridge: Option<&CorsaBridge>,
) -> Option<Vec<Location>> {
    let documents = ctx.virtual_docs.as_ref()?;
    if documents.art_templates.iter().all(Option::is_none) {
        return None;
    }
    let bridge = bridge.filter(|bridge| bridge.is_initialized())?;
    let anchor = match ctx.block_type? {
        BlockType::Script | BlockType::ScriptSetup => {
            crate::ide::token_span_at_offset(&ctx.content, ctx.offset, |byte| {
                byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
            })?
            .0
        }
        BlockType::Template | BlockType::Style(_) => {
            let authored = super::ReferencesService::references(ctx, true)?;
            script_anchor(ctx, &authored)?
        }
        BlockType::Art(_) => return None,
    };
    let opened = synchronize(ctx, bridge).await?;
    shared_at(ctx, anchor, include_declaration, bridge, None, &opened).await
}

async fn synchronize(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
) -> Option<Vec<(Template, String)>> {
    let documents = ctx.virtual_docs.as_ref()?;
    let mut opened = Vec::new();
    let mut contents = Vec::new();
    for (index, template) in documents.art_templates.iter().enumerate() {
        let Some(template) = template else { continue };
        let path = corsa_support::art_template_request_path(ctx.uri, index);
        let uri = corsa_support::request_file_uri(&path);
        contents.push((uri.clone(), template.content.as_str().into()));
        opened.push((Template::Art(index), uri));
    }
    // An inline art binding also drives the ordinary SFC template. In a
    // standalone art file `documents.template` aliases its default variant.
    if !ctx.uri.path().ends_with(".art.vue")
        && let Some(template) = documents.template.as_ref()
    {
        let path = corsa_support::template_request_path(ctx.uri);
        let uri = corsa_support::request_file_uri(&path);
        contents.push((uri.clone(), template.content.as_str().into()));
        opened.push((Template::Ordinary, uri));
    }
    if opened.is_empty() {
        return None;
    }
    bridge.open_virtual_documents_batch(&contents).await.ok()?;
    Some(opened)
}

fn script_anchor(ctx: &IdeContext<'_>, locations: &[Location]) -> Option<usize> {
    let descriptor = ctx.descriptor()?;
    locations.iter().find_map(|location| {
        if location.uri != *ctx.uri {
            return None;
        }
        let start = position_to_offset(
            &ctx.content,
            location.range.start.line,
            location.range.start.character,
        )?;
        let end = position_to_offset(
            &ctx.content,
            location.range.end.line,
            location.range.end.character,
        )?;
        (start < end
            && [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
                .into_iter()
                .flatten()
                .any(|script| start >= script.loc.start && end <= script.loc.end))
        .then_some(start)
    })
}

async fn shared_at(
    ctx: &IdeContext<'_>,
    anchor: usize,
    include_declaration: bool,
    bridge: &CorsaBridge,
    selected_variant: Option<usize>,
    opened: &[(Template, String)],
) -> Option<Vec<Location>> {
    let documents = ctx.virtual_docs.as_ref()?;
    let mut positions = Vec::new();
    for (target, uri) in opened {
        if matches!(target, Template::Art(index) if selected_variant == Some(*index)) {
            continue;
        }
        let template = match target {
            Template::Art(index) => documents.art_template(*index)?,
            Template::Ordinary => documents.template.as_ref()?,
        };
        let Some(offset) = template.source_map.to_generated(anchor) else {
            continue;
        };
        // Reject clamped or derived mappings rather than query a different
        // generated token under a physical source occurrence.
        if template.source_map.to_authored(offset) != Some(anchor) {
            continue;
        }
        let (line, character) = offset_to_position(&template.content, offset);
        positions.push((uri.as_str(), line, character));
    }
    if positions.is_empty() {
        return None;
    }
    let locations = bridge
        .references_batch(&positions, include_declaration)
        .await
        .ok()?
        .into_iter()
        .flatten()
        .collect();
    Some(corsa_support::map_corsa_locations(ctx, locations))
}
