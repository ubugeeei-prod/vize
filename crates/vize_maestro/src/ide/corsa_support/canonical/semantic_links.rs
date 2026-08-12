use vize_canon::{LspPosition, LspRange};
use vize_carton::{String, cstr};

use super::{CanonicalVirtualDocument, location_matches_uri};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CanonicalSemanticPosition {
    pub(crate) request_uri: String,
    pub(crate) line: u32,
    pub(crate) character: u32,
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
    let (request_uri, code) = virtual_code(document, uri)?;
    let start = crate::ide::position_to_offset(code, range.start.line, range.start.character)?;
    let end = crate::ide::position_to_offset(code, range.end.line, range.end.character)?;
    let linked_offset = linked_offset(code, start, end)?;
    let (line, character) = crate::ide::offset_to_position(code, linked_offset);
    Some(CanonicalSemanticPosition {
        request_uri: request_uri.clone(),
        line,
        character,
    })
}

fn linked_offset(code: &str, start: usize, end: usize) -> Option<usize> {
    let name = code.get(start..end)?;
    if name.is_empty() || name.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return None;
    }

    let anchor = cstr!("type __R_{name} = typeof {name};");
    let anchor_name_in_pattern = anchor.rfind(name)?;
    let anchors = code
        .match_indices(anchor.as_str())
        .map(|(offset, _)| offset + anchor_name_in_pattern)
        .collect::<Vec<_>>();
    let shadow = cstr!("var {name}: __U<__R_{name}> =");
    let shadows = code
        .match_indices(shadow.as_str())
        .map(|(offset, _)| offset + "var ".len())
        .collect::<Vec<_>>();

    let linked_offset = if anchors.contains(&start) {
        shadows.into_iter().filter(|offset| *offset > start).min()?
    } else if shadows.contains(&start) {
        anchors.into_iter().filter(|offset| *offset < start).max()?
    } else {
        return None;
    };
    Some(linked_offset)
}

fn virtual_code<'a>(
    document: &'a CanonicalVirtualDocument,
    uri: &str,
) -> Option<(&'a String, &'a str)> {
    if location_matches_uri(uri, document.request_uri.as_str()) {
        return Some((&document.request_uri, &document.virtual_result.code));
    }
    document
        .dependencies
        .iter()
        .find(|dependency| location_matches_uri(uri, dependency.request_uri.as_str()))
        .map(|dependency| {
            (
                &dependency.request_uri,
                dependency.virtual_result.code.as_str(),
            )
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
    use vize_carton::cstr;

    use super::linked_offset;

    #[test]
    fn links_the_matching_generated_pair_when_authored_text_collides() {
        let pair =
            "type __R_shared = typeof shared;\nvar shared: __U<__R_shared> = undefined as any;\n";
        let code = cstr!("{pair}// generated pair\n{pair}");
        let generated_start = code.rfind("typeof shared").unwrap() + "typeof ".len();
        let generated_shadow = code.rfind("var shared").unwrap() + "var ".len();

        assert_eq!(
            linked_offset(&code, generated_start, generated_start + "shared".len(),),
            Some(generated_shadow),
        );
        assert_eq!(
            linked_offset(&code, generated_shadow, generated_shadow + "shared".len(),),
            Some(generated_start),
        );
    }
}
