use tower_lsp::lsp_types::Location;
use vize_canon::{CorsaBridge, LspPosition, LspRange};
use vize_carton::{FxHashSet, String, camelize};

use super::{CanonicalVirtualDocument, location_matches_uri};
use crate::ide::diagnostics::VirtualTsResult;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CanonicalSemanticPosition {
    pub(crate) request_uri: String,
    pub(crate) line: u32,
    pub(crate) character: u32,
}

pub(crate) struct ComponentPropNavigationMatches {
    pub(crate) positions: Vec<CanonicalSemanticPosition>,
    pub(crate) names: FxHashSet<String>,
}

/// Return every live TypeScript identity Canon materialized for one authored
/// source position. Importer-scoped package shadows intentionally duplicate a
/// physical source under distinct native module identities; project-wide
/// references and rename query each identity once, then map and deduplicate the
/// results back to authored locations.
pub(crate) fn materialized_semantic_positions(
    document: &CanonicalVirtualDocument,
    source_uri: &tower_lsp::lsp_types::Url,
    source_offset: usize,
) -> Vec<CanonicalSemanticPosition> {
    let mut positions = Vec::new();
    if same_authored_uri(&document.source_uri, source_uri)
        && let Some(offset) = super::source_offset_to_virtual_generated_offset(
            &document.virtual_result,
            source_offset,
        )
    {
        let (line, character) =
            crate::ide::offset_to_position(&document.virtual_result.code, offset);
        positions.push(CanonicalSemanticPosition {
            request_uri: document.request_uri.clone(),
            line,
            character,
        });
    }
    positions.extend(document.dependencies.iter().filter_map(|source| {
        if !same_authored_uri(&source.source_uri, source_uri) {
            return None;
        }
        let offset = super::source_offset_to_virtual_generated_offset(
            &source.virtual_result,
            source_offset,
        )?;
        let (line, character) = crate::ide::offset_to_position(&source.virtual_result.code, offset);
        Some(CanonicalSemanticPosition {
            request_uri: source.request_uri.clone(),
            line,
            character,
        })
    }));
    positions.extend(
        document
            .materialized_sources
            .iter()
            .filter(|source| same_authored_uri(&source.source_uri, source_uri))
            .filter(|source| {
                !location_matches_uri(&source.request_uri, &document.request_uri)
                    && !document.dependencies.iter().any(|dependency| {
                        location_matches_uri(&source.request_uri, &dependency.request_uri)
                    })
            })
            .filter_map(|source| {
                let offset = super::mapping::materialized_source_offset_to_generated_offset(
                    source,
                    source_offset,
                )?;
                let (line, character) =
                    crate::ide::offset_to_position(&source.virtual_result.code, offset);
                Some(CanonicalSemanticPosition {
                    request_uri: source.request_uri.clone(),
                    line,
                    character,
                })
            }),
    );
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

fn same_authored_uri(left: &tower_lsp::lsp_types::Url, right: &tower_lsp::lsp_types::Url) -> bool {
    if left == right {
        return true;
    }
    match (left.to_file_path(), right.to_file_path()) {
        (Ok(left), Ok(right)) => {
            vize_carton::path::canonicalize_non_verbatim(&left)
                == vize_carton::path::canonicalize_non_verbatim(&right)
        }
        _ => false,
    }
}

/// Resolve the synthetic link that joins an authored setup binding to the
/// template-scope shadow used for Vue ref unwrapping.
///
/// TypeScript correctly keeps a template `v-for` local separate from the
/// setup binding, but the two generated declarations representing the setup
/// binding are intentionally connected through a type alias rather than the
/// same TS symbol. Following that generated edge lets a second semantic query
/// recover template references without a same-spelling source sweep.
pub(crate) fn linked_semantic_position(
    document: &CanonicalVirtualDocument,
    uri: &str,
    range: &LspRange,
) -> Option<CanonicalSemanticPosition> {
    let (request_uri, result) = virtual_result(document, uri)?;
    let start =
        crate::ide::position_to_offset(&result.code, range.start.line, range.start.character)?;
    let end = crate::ide::position_to_offset(&result.code, range.end.line, range.end.character)?;
    let linked_offset = linked_offset(&result.semantic_links, start, end)?;
    let (line, character) = crate::ide::offset_to_position(&result.code, linked_offset);
    Some(CanonicalSemanticPosition {
        request_uri: request_uri.clone(),
        line,
        character,
    })
}

/// Collect the generated component-prop navigation endpoints whose canonical
/// property name matches one of the queried declarations.
///
/// These endpoints are candidates only. References and rename must still ask
/// TypeScript for each endpoint's definition and compare that authored
/// location with the queried declaration before following it.
pub(crate) fn component_prop_navigation_positions(
    document: &CanonicalVirtualDocument,
    names: &FxHashSet<String>,
) -> Vec<CanonicalSemanticPosition> {
    let mut positions = Vec::new();
    collect_component_prop_navigation_positions(
        &document.request_uri,
        &document.virtual_result,
        names,
        &mut positions,
    );
    for dependency in &document.dependencies {
        collect_component_prop_navigation_positions(
            &dependency.request_uri,
            &dependency.virtual_result,
            names,
            &mut positions,
        );
    }
    for source in &document.materialized_sources {
        if source.mapping_kind.is_mappable() {
            collect_component_prop_navigation_positions(
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

/// Resolve only component-prop endpoints whose TypeScript definition maps to
/// the same authored declaration as the primary query.
pub(crate) async fn matching_component_prop_navigation_positions(
    ctx: &crate::ide::IdeContext<'_>,
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
    let authored_definitions = super::map_canonical_corsa_locations(ctx, document, definitions);
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
            super::map_canonical_corsa_locations(ctx, document, candidate_definitions);
        if locations_intersect(&authored_definitions, &candidate_definitions) {
            positions.push(position);
        }
    }
    ComponentPropNavigationMatches { positions, names }
}

pub(crate) fn component_prop_location_matches(
    ctx: &crate::ide::IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    location: &Location,
    names: &FxHashSet<String>,
) -> bool {
    authored_location_text(ctx, document, location)
        .and_then(normalized_component_prop_name)
        .is_some_and(|name| names.contains(name.as_str()))
}

fn authored_location_text<'a>(
    ctx: &'a crate::ide::IdeContext<'_>,
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

fn collect_component_prop_navigation_positions(
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

fn linked_offset(
    links: &[vize_canon::virtual_ts::VizeSemanticLink],
    start: usize,
    end: usize,
) -> Option<usize> {
    links.iter().find_map(|link| {
        if link.kind != vize_canon::virtual_ts::VizeSemanticLinkKind::VueSetupTemplateRefUnwrap {
            return None;
        }
        if link.source_range.start == start && link.source_range.end == end {
            Some(link.target_range.start)
        } else if link.target_range.start == start && link.target_range.end == end {
            Some(link.source_range.start)
        } else {
            None
        }
    })
}

fn virtual_result<'a>(
    document: &'a CanonicalVirtualDocument,
    uri: &str,
) -> Option<(&'a String, &'a VirtualTsResult)> {
    if location_matches_uri(uri, document.request_uri.as_str()) {
        return Some((&document.request_uri, &document.virtual_result));
    }
    document
        .dependencies
        .iter()
        .find(|dependency| location_matches_uri(uri, dependency.request_uri.as_str()))
        .map(|dependency| (&dependency.request_uri, &dependency.virtual_result))
        .or_else(|| {
            document
                .materialized_sources
                .iter()
                .find(|source| location_matches_uri(uri, source.request_uri.as_str()))
                .map(|source| (&source.request_uri, &source.virtual_result))
        })
}

pub(crate) fn tower_range(range: tower_lsp::lsp_types::Range) -> LspRange {
    LspRange {
        start: LspPosition {
            line: range.start.line,
            character: range.start.character,
        },
        end: LspPosition {
            line: range.end.line,
            character: range.end.character,
        },
    }
}

#[cfg(test)]
mod semantic_position_tests {
    use tower_lsp::lsp_types::Url;
    use vize_canon::{CorsaMaterializedMappingKind, ImportSourceMap};

    use super::materialized_semantic_positions;
    use crate::ide::corsa_support::canonical::{
        CanonicalMaterializedSource, CanonicalVirtualDocument,
    };
    use crate::ide::diagnostics::VirtualTsResult;

    fn identity_result(code: &str) -> VirtualTsResult {
        VirtualTsResult {
            code: code.to_owned(),
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
    fn authored_identity_uses_direct_offsets_without_querying_an_unrelated_host() {
        let host_uri = Url::parse("file:///workspace/Host.vue").unwrap();
        let package_uri = Url::parse("file:///workspace/node_modules/pkg/index.ts").unwrap();
        let source = "export const shared = 1\n";
        let offset = source.find("shared").unwrap() + 3;
        let document = CanonicalVirtualDocument {
            source_uri: host_uri,
            request_uri: "file:///mirror/Host.vue.ts".into(),
            virtual_result: identity_result("export {};\n"),
            dependencies: Vec::new(),
            materialized_sources: vec![CanonicalMaterializedSource {
                source_uri: package_uri.clone(),
                source: source.into(),
                request_uri: "file:///mirror/node_modules/pkg/index.ts".into(),
                virtual_result: identity_result(source),
                mapping_kind: CorsaMaterializedMappingKind::AuthoredIdentity,
            }],
            session_project_roots: vec!["/mirror".into()],
        };

        let positions = materialized_semantic_positions(&document, &package_uri, offset);

        assert_eq!(positions.len(), 1);
        assert_eq!(
            positions[0].request_uri,
            "file:///mirror/node_modules/pkg/index.ts"
        );
        assert_eq!((positions[0].line, positions[0].character), (0, 16));
    }
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;
    use vize_canon::virtual_ts::{VizeSemanticLink, VizeSemanticLinkKind};
    use vize_canon::{ImportSourceMap, LspPosition, LspRange};

    use super::{CanonicalVirtualDocument, linked_offset, linked_semantic_position};
    use crate::ide::diagnostics::VirtualTsResult;

    #[test]
    fn links_the_matching_metadata_pair_when_generated_text_collides() {
        let first = VizeSemanticLink {
            source_range: 15..21,
            target_range: 30..36,
            kind: VizeSemanticLinkKind::VueSetupTemplateRefUnwrap,
        };
        let second = VizeSemanticLink {
            source_range: 115..121,
            target_range: 130..136,
            kind: VizeSemanticLinkKind::VueSetupTemplateRefUnwrap,
        };
        let links = vec![first, second];

        assert_eq!(linked_offset(&links, 115, 121), Some(130));
        assert_eq!(linked_offset(&links, 130, 136), Some(115));
    }

    #[test]
    fn linked_position_uses_metadata_without_generated_helper_spelling() {
        let code =
            "type Capture = typeof shared;\nvar shared: Unwrap<Capture> = undefined as any;\n";
        let source_start = code.find("typeof shared").unwrap() + "typeof ".len();
        let target_start = code.find("var shared").unwrap() + "var ".len();
        let link = VizeSemanticLink {
            source_range: source_start..source_start + "shared".len(),
            target_range: target_start..target_start + "shared".len(),
            kind: VizeSemanticLinkKind::VueSetupTemplateRefUnwrap,
        };
        let document = CanonicalVirtualDocument {
            source_uri: Url::parse("file:///workspace/App.vue").unwrap(),
            request_uri: "file:///workspace/App.vue.ts".into(),
            virtual_result: VirtualTsResult {
                code: code.into(),
                source_mappings: Vec::new(),
                semantic_links: vec![link],
                import_source_map: ImportSourceMap::empty(),
                user_code_start_line: 0,
                sfc_script_start_line: 0,
                template_scope_start_line: 0,
                line_mappings: Vec::new(),
                skipped_import_lines: 0,
            },
            dependencies: Vec::new(),
            materialized_sources: Vec::new(),
            session_project_roots: Vec::new(),
        };
        let (line, character) = crate::ide::offset_to_position(code, source_start);
        let (_, end_character) =
            crate::ide::offset_to_position(code, source_start + "shared".len());

        let linked = linked_semantic_position(
            &document,
            "file:///workspace/App.vue.ts",
            &LspRange {
                start: LspPosition { line, character },
                end: LspPosition {
                    line,
                    character: end_character,
                },
            },
        )
        .expect("linked position");
        let expected = crate::ide::offset_to_position(code, target_start);

        assert_eq!((linked.line, linked.character), expected);
    }
}
