use tower_lsp::lsp_types::Url;
use vize_canon::{LspPosition, LspRange};
use vize_carton::{FxHashMap, FxHashSet, String};

use super::{CanonicalVirtualDocument, location_matches_uri};
use crate::ide::diagnostics::VirtualTsResult;

mod component_props;

pub(crate) use component_props::{
    component_prop_location_matches, matching_component_prop_navigation_positions,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CanonicalSemanticPosition {
    pub(crate) request_uri: String,
    pub(crate) line: u32,
    pub(crate) character: u32,
}

pub(crate) struct ComponentPropNavigationMatches {
    pub(crate) positions: Vec<CanonicalSemanticPosition>,
    pub(crate) names: FxHashSet<String>,
    pub(crate) source_cache: ComponentPropSourceCache,
}

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
