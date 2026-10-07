//! All exact generated identities for a resolved authored property declaration.

use tower_lsp::lsp_types::Location;

use super::{CanonicalSemanticPosition, CanonicalVirtualDocument};
use crate::ide::diagnostics::VirtualTsResult;

pub(super) fn positions(
    document: &CanonicalVirtualDocument,
    definition: &Location,
    source: &str,
) -> Vec<CanonicalSemanticPosition> {
    let offset = |position: tower_lsp::lsp_types::Position| {
        crate::ide::position_to_offset(source, position.line, position.character)
    };
    let (Some(start), Some(end)) = (offset(definition.range.start), offset(definition.range.end))
    else {
        return Vec::new();
    };
    if start >= end {
        return Vec::new();
    }
    let range = start..end;
    let mut positions = Vec::new();
    let mut collect = |request_uri: &vize_l0::String, result: &VirtualTsResult| {
        positions.extend(
            super::super::mapping::source_range_to_virtual_positions(result, source, &range)
                .into_iter()
                .map(|(line, character)| CanonicalSemanticPosition {
                    request_uri: request_uri.clone(),
                    line,
                    character,
                }),
        );
    };
    if document.source_uri == definition.uri {
        collect(&document.request_uri, &document.virtual_result);
    }
    for dependency in &document.dependencies {
        if dependency.source_uri == definition.uri {
            collect(&dependency.request_uri, &dependency.virtual_result);
        }
    }
    for materialized in &document.materialized_sources {
        if materialized.source_uri == definition.uri
            && materialized.mapping_kind == vize_canon::CorsaMaterializedMappingKind::Generated
        {
            collect(&materialized.request_uri, &materialized.virtual_result);
        }
    }
    positions
}
