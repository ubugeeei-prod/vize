use tower_lsp::lsp_types::{Location, Range, Url};
use vize_canon::LspLocation;
use vize_carton::{String, cstr};

use crate::ide::IdeContext;
use crate::ide::diagnostics::VirtualTsResult;

mod mapping;
mod open;
mod project;
pub(super) mod rename;
mod semantic_links;

use mapping::source_offset_to_virtual_generated_offset;
pub(crate) use mapping::{canonical_source_offset_to_position, map_canonical_lsp_range};
pub(super) use mapping::{map_lsp_range_to_source, map_virtual_result_lsp_range_to_source};
pub(crate) use open::{open_canonical_virtual_document, open_canonical_virtual_document_strict};
pub(crate) use project::{
    CanonicalProjectOpenError, open_canonical_virtual_project_document,
    open_canonical_virtual_project_document_strict, open_canonical_virtual_workspace_document,
};
pub(crate) use rename::{
    map_canonical_corsa_workspace_edit, map_canonical_prepare_rename,
    merge_canonical_workspace_edits,
};
pub(crate) use semantic_links::materialized_semantic_positions;
pub(crate) use semantic_links::{
    CanonicalSemanticPosition, ComponentPropSourceCache, component_prop_location_matches,
    linked_semantic_position, matching_component_prop_navigation_positions, tower_range,
};

pub(crate) struct CanonicalVirtualDocument {
    pub(crate) source_uri: Url,
    pub(crate) request_uri: String,
    pub(crate) virtual_result: VirtualTsResult,
    pub(crate) dependencies: Vec<CanonicalDependencyDocument>,
    pub(crate) materialized_sources: Vec<CanonicalMaterializedSource>,
    pub(crate) session_project_roots: Vec<std::path::PathBuf>,
}

pub(crate) struct CanonicalDependencyDocument {
    pub(crate) source_uri: Url,
    pub(crate) source: String,
    pub(crate) request_uri: String,
    pub(crate) virtual_result: VirtualTsResult,
}

pub(crate) struct CanonicalMaterializedSource {
    pub(crate) source_uri: Url,
    pub(crate) source: String,
    pub(crate) request_uri: String,
    pub(crate) virtual_result: VirtualTsResult,
    pub(crate) mapping_kind: vize_canon::CorsaMaterializedMappingKind,
}

impl CanonicalVirtualDocument {
    /// Authored text retained while the canonical workspace surface is open.
    /// Closed SFCs are dependencies or materialized sources rather than editor
    /// documents, so consumers such as style-reference expansion must resolve
    /// them here instead of consulting only the open-buffer store.
    pub(crate) fn authored_source(&self, uri: &Url) -> Option<&str> {
        self.dependencies
            .iter()
            .find(|dependency| dependency.source_uri == *uri)
            .map(|dependency| dependency.source.as_str())
            .or_else(|| {
                self.materialized_sources
                    .iter()
                    .find(|source| source.source_uri == *uri)
                    .map(|source| source.source.as_str())
            })
    }
}

pub(crate) fn canonical_request_path(uri: &Url) -> String {
    cstr!("{}.ts", uri.path())
}

fn is_canonical_vue_virtual_uri(uri: &Url) -> bool {
    uri.path().ends_with(".vue.ts") || uri.path().ends_with(".vue.tsx")
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

    if let Some(materialized) = doc
        .materialized_sources
        .iter()
        .find(|source| location_matches_uri(&location.uri, &source.request_uri))
    {
        if !materialized.mapping_kind.is_mappable() {
            return None;
        }
        let range = map_virtual_result_lsp_range_to_source(
            &materialized.source,
            &materialized.virtual_result,
            &location.range,
        )?;
        return Some(Location {
            uri: materialized.source_uri.clone(),
            range,
        });
    }

    if let Some(location) = super::external_mirror::map_location(ctx, location) {
        return Some(location);
    }

    if is_private_materialized_uri(doc, &location.uri) {
        return None;
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

/// Map an exact Canon-owned materialized module identity to its authored file
/// without interpreting coordinates from a synthetic forwarder. This is only
/// valid for module-specifier definitions, where the destination is the module
/// itself rather than a symbol range inside the generated file.
///
/// Every materialized identity, including a synthetic declaration shadow such
/// as the `.d.vue.ts` forwarder TypeScript selects for a conditional package
/// entry, still stands for exactly one authored file. Because this mapping
/// keeps only that file identity and pins the range to the file origin, it
/// stays correct for synthetic sources whose interiors are not position
/// mappable.
pub(crate) fn map_canonical_materialized_module_location(
    doc: &CanonicalVirtualDocument,
    location: &LspLocation,
) -> Option<Location> {
    let materialized = doc
        .materialized_sources
        .iter()
        .find(|source| location_matches_uri(&location.uri, &source.request_uri))?;
    let origin = tower_lsp::lsp_types::Position::new(0, 0);
    Some(Location {
        uri: materialized.source_uri.clone(),
        range: Range::new(origin, origin),
    })
}

fn location_matches_uri(actual: &str, expected: &str) -> bool {
    actual == expected
        || super::virtual_document_path(actual).as_deref()
            == super::virtual_document_path(expected).as_deref()
        || file_uri_paths_match(actual, expected)
}

fn file_uri_paths_match(left: &str, right: &str) -> bool {
    let Some(left) = Url::parse(left)
        .ok()
        .and_then(|uri| uri.to_file_path().ok())
    else {
        return false;
    };
    let Some(right) = Url::parse(right)
        .ok()
        .and_then(|uri| uri.to_file_path().ok())
    else {
        return false;
    };
    vize_carton::path::canonicalize_non_verbatim(&left)
        == vize_carton::path::canonicalize_non_verbatim(&right)
}

pub(super) fn is_private_materialized_uri(doc: &CanonicalVirtualDocument, raw_uri: &str) -> bool {
    let Some(path) = Url::parse(raw_uri)
        .ok()
        .and_then(|uri| uri.to_file_path().ok())
    else {
        return false;
    };
    doc.session_project_roots.iter().any(|root| {
        path.starts_with(root)
            || vize_carton::path::canonicalize_non_verbatim(&path)
                .starts_with(vize_carton::path::canonicalize_non_verbatim(root))
    })
}
