//! Producer-owned binding links for the existing native follow-up query.

use vize_canon::virtual_ts::VizeSemanticLinkKind;
use vize_canon::LspRange;

use super::{CanonicalSemanticPosition, CanonicalVirtualDocument};

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
    let (request_uri, result) = super::virtual_result(document, uri)?;
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
        if !matches!(
            link.kind,
            VizeSemanticLinkKind::VueSetupTemplateRefUnwrap
                | VizeSemanticLinkKind::VueTemplatePropBinding
                | VizeSemanticLinkKind::VuePlainScriptExport
                | VizeSemanticLinkKind::VueOptionsApiBinding
                | VizeSemanticLinkKind::VueSetupImportSpecialization
        ) {
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
            },
            dependencies: Vec::new(),
            materialized_sources: Vec::new(),
            session_project_roots: Vec::new(),
            source_catalogs: Vec::new(),
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
