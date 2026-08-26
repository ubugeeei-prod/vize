//! Shared Corsa helpers for mapping virtual document responses back to Vue SFCs.
#![cfg(feature = "native")]
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{Location, PrepareRenameResponse, Range, Url};
#[cfg(test)]
mod art_variant_fallback_tests;
mod canonical;
#[cfg(test)]
mod canonical_dependency_tests;
#[cfg(test)]
mod canonical_rename_tests;
#[cfg(test)]
mod canonical_tests;
mod external_mirror;
mod html_attribute;
#[cfg(test)]
mod html_attribute_tests;
mod html_tag;
mod location_merge;
mod rename_merge;
mod svg_attribute;
mod virtual_document;
mod workspace_edit;

pub(crate) use canonical::{
    CanonicalProjectOpenError, CanonicalSemanticPosition, CanonicalVirtualDocument,
    ComponentPropNavigationIdentities, ComponentPropSourceCache,
    canonical_source_offset_to_position, component_prop_location_matches,
    component_prop_navigation_identity_matches, linked_semantic_position,
    map_canonical_corsa_location, map_canonical_corsa_locations,
    map_canonical_corsa_workspace_edit, map_canonical_lsp_range,
    map_canonical_materialized_module_location, map_canonical_prepare_rename,
    matching_component_prop_navigation_positions, materialized_semantic_positions,
    merge_canonical_workspace_edits, open_canonical_virtual_document,
    open_canonical_virtual_document_strict, open_canonical_virtual_project_document,
    open_canonical_virtual_project_document_strict, open_canonical_virtual_workspace_document,
    tower_range,
};
pub(crate) use html_attribute::{
    html_attribute_request_path, html_attribute_virtual_document, native_dom_attribute_info,
};
pub(crate) use html_tag::{html_tag_request_path, html_tag_virtual_document, native_dom_tag_info};
pub(crate) use location_merge::merge_canonical_locations;
pub(crate) use rename_merge::merge_authored_rename;
use virtual_document::{
    MatchedVirtualDocument, is_virtual_document_uri, match_virtual_document, virtual_document_path,
};
pub(crate) use workspace_edit::map_corsa_workspace_edit;

use vize_canon::LspLocation;
use vize_s0::{String, cstr};

use super::IdeContext;
use crate::virtual_code::{SourceRange, VirtualDocument};

pub(crate) fn template_request_path(uri: &Url) -> String {
    cstr!("{}.template.ts", uri.path())
}

pub(crate) fn art_template_request_path(uri: &Url, variant_index: usize) -> String {
    cstr!("{}.art_variant_{variant_index}.template.ts", uri.path())
}

pub(crate) fn script_request_path(uri: &Url, is_setup: bool) -> String {
    if is_setup {
        cstr!("{}.setup.ts", uri.path())
    } else {
        cstr!("{}.script.ts", uri.path())
    }
}

/// Map a completion cursor, including the valid position immediately after a
/// source expression such as `props.`. Other language features keep half-open
/// source ranges so a cursor between tokens cannot attach to the previous one.
pub(crate) fn completion_source_offset_to_generated(
    document: &VirtualDocument,
    source_offset: u32,
) -> Option<usize> {
    document
        .source_map
        .to_generated_for(source_offset, |features| features.completion)
        .or_else(|| {
            document
                .source_map
                .mappings()
                .iter()
                .filter(|mapping| {
                    mapping.features.completion && mapping.source.end == source_offset
                })
                .min_by_key(|mapping| mapping.source.end.saturating_sub(mapping.source.start))
                .map(|mapping| mapping.generated.end)
        })
        .map(|offset| offset as usize)
}

pub(crate) fn request_file_uri(path: &str) -> String {
    if path.starts_with("file://") {
        String::from(path)
    } else {
        cstr!("file://{path}")
    }
}

/// Map a batch of Corsa locations back onto the current Vue document.
pub(crate) fn map_corsa_locations(
    ctx: &IdeContext<'_>,
    locations: Vec<LspLocation>,
) -> Vec<Location> {
    locations
        .iter()
        .filter_map(|location| map_corsa_location(ctx, location))
        .collect()
}

/// Map a single Corsa location back to either the Vue SFC or a real file URI.
pub(crate) fn map_corsa_location(ctx: &IdeContext<'_>, location: &LspLocation) -> Option<Location> {
    if let Some(target) = match_virtual_document(ctx, &location.uri) {
        let range = map_virtual_range_for_content(
            target.content(),
            target.document()?,
            &Range {
                start: tower_lsp::lsp_types::Position {
                    line: location.range.start.line,
                    character: location.range.start.character,
                },
                end: tower_lsp::lsp_types::Position {
                    line: location.range.end.line,
                    character: location.range.end.character,
                },
            },
        )?;

        return Some(Location {
            uri: target.uri().clone(),
            range,
        });
    }
    if is_virtual_document_uri(&location.uri) {
        return None;
    }

    if let Some(location) = external_mirror::map_location(ctx, location) {
        return Some(location);
    }

    let uri = accessible_external_uri(ctx, &location.uri)?;
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

/// Reject stale file locations returned from an older checker snapshot.
///
/// Open editor buffers are valid even before their first save. Closed file
/// targets, however, must still exist on disk before they are exposed to the
/// client; otherwise delete/rename races leave a convincing but dead jump.
fn accessible_external_uri(ctx: &IdeContext<'_>, raw_uri: &str) -> Option<Url> {
    let uri = Url::parse(raw_uri).ok()?;
    if uri.scheme() != "file" || ctx.state.documents.contains(&uri) {
        return Some(uri);
    }

    uri.to_file_path().ok()?.is_file().then_some(uri)
}

/// Translate a Corsa prepare-rename payload into SFC coordinates.
pub(crate) fn map_corsa_prepare_rename(
    ctx: &IdeContext<'_>,
    request_uri: &str,
    response: PrepareRenameResponse,
) -> Option<PrepareRenameResponse> {
    let target = match_virtual_document(ctx, request_uri)?;

    match response {
        PrepareRenameResponse::Range(range) => {
            map_virtual_range_for_content(target.content(), target.document()?, &range)
                .map(PrepareRenameResponse::Range)
        }
        // The placeholder Corsa reports is sliced out of the *generated*
        // document, so it describes neither the authored span nor even a valid
        // identifier once the range is mapped back. Answer with the authored
        // range alone and let the client seed the rename from the SFC text.
        PrepareRenameResponse::RangeWithPlaceholder { range, .. } => {
            map_virtual_range_for_content(target.content(), target.document()?, &range)
                .map(PrepareRenameResponse::Range)
        }
        PrepareRenameResponse::DefaultBehavior { default_behavior } => {
            Some(PrepareRenameResponse::DefaultBehavior { default_behavior })
        }
    }
}

pub(crate) fn map_virtual_range(
    ctx: &IdeContext<'_>,
    document: &VirtualDocument,
    range: &Range,
) -> Option<Range> {
    map_virtual_range_for_content(&ctx.content, document, range)
}

fn map_virtual_range_for_content(
    content: &str,
    document: &VirtualDocument,
    range: &Range,
) -> Option<Range> {
    let generated_start =
        super::position_to_offset(&document.content, range.start.line, range.start.character)?;
    let generated_end =
        super::position_to_offset(&document.content, range.end.line, range.end.character)?;

    let source_range = if generated_end > generated_start {
        document
            .source_map
            .generated_range_to_source(SourceRange::new(
                generated_start as u32,
                generated_end as u32,
            ))?
    } else {
        let source_offset = document.source_map.to_source(generated_start as u32)?;
        SourceRange::new(source_offset, source_offset)
    };

    let (start_line, start_character) =
        super::offset_to_position(content, source_range.start as usize);
    let (end_line, end_character) = super::offset_to_position(content, source_range.end as usize);

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
