use tower_lsp::lsp_types::Location;
use vize_canon::CorsaBridge;
use vize_carton::{FxHashSet, String, camelize};

use super::{CanonicalSemanticPosition, ComponentPropNavigationMatches};
use crate::ide::IdeContext;
use crate::ide::corsa_support::canonical::{
    CanonicalVirtualDocument, map_canonical_corsa_locations,
};
use crate::ide::diagnostics::VirtualTsResult;

/// Resolve only component-prop endpoints whose TypeScript definition maps to
/// the same authored declaration as the primary query.
pub(crate) async fn matching_component_prop_navigation_positions(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
    document: &CanonicalVirtualDocument,
    request_uri: &str,
    line: u32,
    character: u32,
) -> ComponentPropNavigationMatches {
    let Ok(definitions) = bridge.definition(request_uri, line, character).await else {
        return ComponentPropNavigationMatches {
            positions: Vec::new(),
            names: FxHashSet::default(),
        };
    };
    let authored_definitions = map_canonical_corsa_locations(ctx, document, definitions);
    let names = authored_definitions
        .iter()
        .filter_map(|location| authored_location_text(ctx, document, location))
        .filter_map(normalized_component_prop_name)
        .collect::<FxHashSet<_>>();
    let mut positions = Vec::new();
    for position in component_prop_navigation_positions(document, &names) {
        let Ok(candidate_definitions) = bridge
            .definition(&position.request_uri, position.line, position.character)
            .await
        else {
            continue;
        };
        let candidate_definitions =
            map_canonical_corsa_locations(ctx, document, candidate_definitions);
        if locations_intersect(&authored_definitions, &candidate_definitions) {
            positions.push(position);
        }
    }
    ComponentPropNavigationMatches { positions, names }
}

pub(crate) fn component_prop_location_matches(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    location: &Location,
    names: &FxHashSet<String>,
) -> bool {
    authored_location_text(ctx, document, location)
        .and_then(normalized_component_prop_name)
        .is_some_and(|name| names.contains(name.as_str()))
}

/// Collect the generated component-prop navigation endpoints whose canonical
/// property name matches one of the queried declarations.
///
/// These endpoints are candidates only. References and rename must still ask
/// TypeScript for each endpoint's definition and compare that authored
/// location with the queried declaration before following it.
fn component_prop_navigation_positions(
    document: &CanonicalVirtualDocument,
    names: &FxHashSet<String>,
) -> Vec<CanonicalSemanticPosition> {
    let mut positions = Vec::new();
    collect_positions(
        &document.request_uri,
        &document.virtual_result,
        names,
        &mut positions,
    );
    for dependency in &document.dependencies {
        collect_positions(
            &dependency.request_uri,
            &dependency.virtual_result,
            names,
            &mut positions,
        );
    }
    for source in &document.materialized_sources {
        if source.mapping_kind.is_mappable() {
            collect_positions(
                &source.request_uri,
                &source.virtual_result,
                names,
                &mut positions,
            );
        }
    }
    positions.sort_by(|left, right| {
        (&left.request_uri, left.line, left.character).cmp(&(
            &right.request_uri,
            right.line,
            right.character,
        ))
    });
    positions.dedup();
    positions
}

fn authored_location_text<'a>(
    ctx: &'a IdeContext<'_>,
    document: &'a CanonicalVirtualDocument,
    location: &Location,
) -> Option<&'a str> {
    let source = if location.uri == *ctx.uri {
        ctx.content.as_str()
    } else {
        document.authored_source(&location.uri)?
    };
    let start = crate::ide::position_to_offset(
        source,
        location.range.start.line,
        location.range.start.character,
    )?;
    let end = crate::ide::position_to_offset(
        source,
        location.range.end.line,
        location.range.end.character,
    )?;
    source.get(start..end)
}

fn normalized_component_prop_name(raw: &str) -> Option<String> {
    let name = raw.trim_matches(['\'', '"', '`']);
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'-'))
    {
        return None;
    }
    Some(camelize(name))
}

fn locations_intersect(left: &[Location], right: &[Location]) -> bool {
    left.iter().any(|left| {
        right
            .iter()
            .any(|right| left.uri == right.uri && left.range == right.range)
    })
}

fn collect_positions(
    request_uri: &String,
    result: &VirtualTsResult,
    names: &FxHashSet<String>,
    positions: &mut Vec<CanonicalSemanticPosition>,
) {
    for link in &result.semantic_links {
        if link.kind != vize_canon::virtual_ts::VizeSemanticLinkKind::VueComponentPropNavigation {
            continue;
        }
        let Some(name) = result.code.get(link.target_range.clone()) else {
            continue;
        };
        if !names.contains(name) {
            continue;
        }
        let (line, character) =
            crate::ide::offset_to_position(&result.code, link.target_range.start);
        positions.push(CanonicalSemanticPosition {
            request_uri: request_uri.clone(),
            line,
            character,
        });
    }
}
