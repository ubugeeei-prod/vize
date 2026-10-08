use tower_lsp::lsp_types::{Location, Url};
use vize_canon::{LspPosition, LspRange};
use vize_l0::{FxHashMap, FxHashSet, String};

use super::{CanonicalVirtualDocument, location_matches_uri};
use crate::ide::diagnostics::VirtualTsResult;

mod binding_links;
mod binding_roles;

pub(crate) use binding_links::linked_semantic_position;
mod component_arguments;
mod component_props;

pub(crate) use component_props::{
    component_prop_location_matches, component_prop_navigation_identity_matches,
    matching_component_prop_navigation_positions,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CanonicalSemanticPosition {
    pub(crate) request_uri: String,
    pub(crate) line: u32,
    pub(crate) character: u32,
}

#[derive(Default)]
pub(crate) struct ComponentPropNavigationMatches {
    pub(crate) positions: Vec<CanonicalSemanticPosition>,
    pub(crate) names: FxHashSet<String>,
    pub(crate) authored_definitions: Vec<Location>,
    pub(crate) navigation_identities: ComponentPropNavigationIdentities,
    pub(crate) source_cache: ComponentPropSourceCache,
}

pub(crate) type ComponentPropNavigationIdentities =
    FxHashMap<CanonicalSemanticPosition, Vec<Location>>;
pub(crate) type ComponentPropSourceCache = FxHashMap<Url, Option<std::string::String>>;

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

pub(super) fn virtual_result<'a>(
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
            source_catalogs: Vec::new(),
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

