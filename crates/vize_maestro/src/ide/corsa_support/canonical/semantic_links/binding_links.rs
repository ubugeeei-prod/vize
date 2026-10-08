//! Exact producer-owned binding graph traversal for the existing native batch.

use vize_canon::LspRange;
use vize_canon::virtual_ts::VizeSemanticLinkKind;
use vize_l0::FxHashSet;

use super::{CanonicalSemanticPosition, CanonicalVirtualDocument};

/// Resolve every exact endpoint connected by Canon's authored binding metadata.
/// Complete the graph before the existing native follow-up batch, so a default
/// key, typed property access and bare template binding share one identity.
/// Component-navigation owner edges are intentionally not binding aliases.
pub(crate) fn linked_semantic_positions(
    document: &CanonicalVirtualDocument,
    uri: &str,
    range: &LspRange,
) -> Vec<CanonicalSemanticPosition> {
    let Some((request_uri, result)) = super::virtual_result(document, uri) else {
        return Vec::new();
    };
    let Some(start) =
        crate::ide::position_to_offset(&result.code, range.start.line, range.start.character)
    else {
        return Vec::new();
    };
    let Some(end) =
        crate::ide::position_to_offset(&result.code, range.end.line, range.end.character)
    else {
        return Vec::new();
    };
    linked_offsets(
        &result.semantic_links,
        &result.prop_default_key_links,
        start,
        end,
    )
    .into_iter()
    .map(|offset| {
        let (line, character) = crate::ide::offset_to_position(&result.code, offset);
        CanonicalSemanticPosition {
            request_uri: request_uri.clone(),
            line,
            character,
        }
    })
    .collect()
}

fn linked_offsets(
    links: &[vize_canon::virtual_ts::VizeSemanticLink],
    prop_default_key_links: &[vize_canon::virtual_ts::VizeSemanticLink],
    start: usize,
    end: usize,
) -> Vec<usize> {
    // Most native locations are not bridge endpoints. Preserve their scan-only
    // fast path without allocating graph state for unrelated references.
    if !links.iter().chain(prop_default_key_links).any(|link| {
        is_binding_link(link.kind)
            && ((link.source_range.start == start && link.source_range.end == end)
                || (link.target_range.start == start && link.target_range.end == end))
    }) {
        return Vec::new();
    }
    let origin = (start, end);
    let mut visited = FxHashSet::default();
    visited.insert(origin);
    let mut pending = vec![origin];
    let mut offsets = Vec::new();
    while let Some(endpoint) = pending.pop() {
        for link in links.iter().chain(prop_default_key_links) {
            if !is_binding_link(link.kind) {
                continue;
            }
            let source = (link.source_range.start, link.source_range.end);
            let target = (link.target_range.start, link.target_range.end);
            let neighbor = if endpoint == source {
                Some(target)
            } else if endpoint == target {
                Some(source)
            } else {
                None
            };
            if let Some(neighbor) = neighbor
                && visited.insert(neighbor)
            {
                offsets.push(neighbor.0);
                pending.push(neighbor);
            }
        }
    }
    offsets.sort_unstable();
    offsets.dedup();
    offsets
}

fn is_binding_link(kind: VizeSemanticLinkKind) -> bool {
    matches!(
        kind,
        VizeSemanticLinkKind::VueSetupTemplateRefUnwrap
            | VizeSemanticLinkKind::VueTemplatePropBinding
            | VizeSemanticLinkKind::VuePlainScriptExport
            | VizeSemanticLinkKind::VueOptionsApiBinding
            | VizeSemanticLinkKind::VueSetupImportSpecialization
    )
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;
    use vize_canon::virtual_ts::{VizeSemanticLink, VizeSemanticLinkKind};
    use vize_canon::{ImportSourceMap, LspPosition, LspRange};

    use super::{CanonicalVirtualDocument, linked_offsets, linked_semantic_positions};
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

        assert_eq!(linked_offsets(&links, &[], 115, 121), vec![130]);
        assert_eq!(linked_offsets(&links, &[], 130, 136), vec![115]);
    }

    #[test]
    fn binding_graph_reaches_all_exact_endpoints_without_crossing_owner_edges() {
        let links = vec![
            VizeSemanticLink {
                source_range: 10..14,
                target_range: 20..24,
                kind: VizeSemanticLinkKind::VueTemplatePropBinding,
            },
            VizeSemanticLink {
                source_range: 10..14,
                target_range: 30..34,
                kind: VizeSemanticLinkKind::VueTemplatePropBinding,
            },
            VizeSemanticLink {
                source_range: 30..34,
                target_range: 20..24,
                kind: VizeSemanticLinkKind::VueSetupTemplateRefUnwrap,
            },
            VizeSemanticLink {
                source_range: 30..34,
                target_range: 40..44,
                kind: VizeSemanticLinkKind::VueComponentPropNavigation,
            },
            VizeSemanticLink {
                source_range: 110..114,
                target_range: 120..124,
                kind: VizeSemanticLinkKind::VueTemplatePropBinding,
            },
        ];
        assert_eq!(linked_offsets(&links, &[], 10, 14), vec![20, 30]);
        assert_eq!(linked_offsets(&links, &[], 20, 24), vec![10, 30]);
        assert_eq!(linked_offsets(&links, &[], 30, 34), vec![10, 20]);
        assert!(linked_offsets(&links, &[], 10, 13).is_empty());
        assert!(linked_offsets(&links, &[], 40, 44).is_empty());
        assert_eq!(linked_offsets(&links, &[], 110, 114), vec![120]);
    }

    #[test]
    fn private_default_key_edges_join_the_existing_single_core_link() {
        let core = vec![VizeSemanticLink {
            source_range: 10..14,
            target_range: 20..24,
            kind: VizeSemanticLinkKind::VueTemplatePropBinding,
        }];
        let auxiliary = vec![VizeSemanticLink {
            source_range: 10..14,
            target_range: 30..34,
            kind: VizeSemanticLinkKind::VueTemplatePropBinding,
        }];
        assert_eq!(core.len(), 1);
        assert_eq!(linked_offsets(&core, &auxiliary, 10, 14), vec![20, 30]);
        assert_eq!(linked_offsets(&core, &auxiliary, 20, 24), vec![10, 30]);
        assert_eq!(linked_offsets(&core, &auxiliary, 30, 34), vec![10, 20]);
        assert!(linked_offsets(&core, &auxiliary, 30, 33).is_empty());
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
                prop_default_key_links: Vec::new(),
                import_source_map: ImportSourceMap::empty(),
            },
            dependencies: Vec::new(),
            materialized_sources: Vec::new(),
            session_project_roots: Vec::new(),
            source_catalogs: Vec::new(),
        };
        let (line, character) = crate::ide::offset_to_position(code, source_start);
        let (_, end_character) =
            crate::ide::offset_to_position(code, source_start + "shared".len());

        let linked = linked_semantic_positions(
            &document,
            "file:///workspace/App.vue.ts",
            &LspRange {
                start: LspPosition { line, character },
                end: LspPosition {
                    line,
                    character: end_character,
                },
            },
        );
        assert_eq!(linked.len(), 1);
        let linked = &linked[0];
        let expected = crate::ide::offset_to_position(code, target_start);

        assert_eq!((linked.line, linked.character), expected);
    }
}
