use std::borrow::Cow;

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
        .filter_map(|name| normalized_component_prop_name(name.as_str()))
        .collect::<FxHashSet<_>>();
    let candidates = component_prop_navigation_positions(document, &names);
    let queries = candidates
        .iter()
        .map(|position| {
            (
                position.request_uri.as_str(),
                position.line,
                position.character,
            )
        })
        .collect::<Vec<_>>();
    let Ok(definition_batches) = bridge.definition_batch(&queries).await else {
        return ComponentPropNavigationMatches {
            positions: Vec::new(),
            names,
        };
    };
    let mut positions = Vec::new();
    for (position, candidate_definitions) in candidates.into_iter().zip(definition_batches) {
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
        .and_then(|name| normalized_component_prop_name(name.as_str()))
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
) -> Option<String> {
    let source = if location.uri == *ctx.uri {
        Cow::Borrowed(ctx.content.as_str())
    } else if let Some(source) = document.authored_source(&location.uri) {
        Cow::Borrowed(source)
    } else if let Some(source) = ctx.state.documents.text(&location.uri) {
        Cow::Owned(source)
    } else {
        let path = location.uri.to_file_path().ok()?;
        Cow::Owned(std::fs::read_to_string(path).ok()?)
    };
    let start = crate::ide::position_to_offset(
        source.as_ref(),
        location.range.start.line,
        location.range.start.character,
    )?;
    let end = crate::ide::position_to_offset(
        source.as_ref(),
        location.range.end.line,
        location.range.end.character,
    )?;
    source.get(start..end).map(String::from)
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

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::{Location, Position, Range, Url};
    use vize_canon::ImportSourceMap;

    use super::authored_location_text;
    use crate::ide::IdeContext;
    use crate::ide::corsa_support::canonical::CanonicalVirtualDocument;
    use crate::ide::diagnostics::VirtualTsResult;
    use crate::server::ServerState;

    fn empty_result() -> VirtualTsResult {
        VirtualTsResult {
            code: "".to_owned(),
            source_mappings: Vec::new(),
            semantic_links: Vec::new(),
            import_source_map: ImportSourceMap::empty(),
            user_code_start_line: 0,
            sfc_script_start_line: 0,
            template_scope_start_line: 0,
            line_mappings: Vec::new(),
            skipped_import_lines: 0,
        }
    }

    #[test]
    fn reads_external_prop_declarations_from_disk_with_utf16_ranges() {
        let project = tempfile::TempDir::new().expect("project");
        let props_path = project.path().join("props.ts");
        let source = "export interface Props { /* café 🌱 */ title: string }\n";
        std::fs::write(&props_path, source).expect("external props");
        let props_uri = Url::from_file_path(props_path).expect("props URI");
        let app_uri = Url::from_file_path(project.path().join("App.vue")).expect("app URI");
        let state = ServerState::new();
        let ctx = IdeContext::with_content(&state, &app_uri, 0, "<template />".to_owned());
        let document = CanonicalVirtualDocument {
            source_uri: app_uri.clone(),
            request_uri: "file:///project/App.vue.ts".into(),
            virtual_result: empty_result(),
            dependencies: Vec::new(),
            materialized_sources: Vec::new(),
            session_project_roots: vec![project.path().to_path_buf()],
        };
        let start = source.find("title").expect("prop name");
        let end = start + "title".len();
        let (start_line, start_character) = crate::ide::offset_to_position(source, start);
        let (end_line, end_character) = crate::ide::offset_to_position(source, end);
        let location = Location::new(
            props_uri,
            Range::new(
                Position::new(start_line, start_character),
                Position::new(end_line, end_character),
            ),
        );

        assert_eq!(
            authored_location_text(&ctx, &document, &location).as_deref(),
            Some("title"),
        );
        assert_ne!(start_character as usize, start);
    }
}
