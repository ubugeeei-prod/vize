//! The real merged class expression must retain each authored value location.

use tower_lsp::lsp_types::{Location, Position, Range, TextEdit, Url, WorkspaceEdit};
use vize_canon::{LspLocation, LspPosition, LspRange};

use super::*;
use crate::ide::corsa_support::canonical_dependency_tests::host_document;
use crate::ide::{offset_to_position, position_to_offset};
use crate::server::ServerState;

fn range(source: &str, start: usize, end: usize) -> Range {
    let start = offset_to_position(source, start);
    let end = offset_to_position(source, end);
    Range::new(Position::new(start.0, start.1), Position::new(end.0, end.1))
}

fn read_location(uri: &str, range: Range) -> LspLocation {
    LspLocation {
        uri: uri.into(),
        range: LspRange {
            start: LspPosition {
                line: range.start.line,
                character: range.start.character,
            },
            end: LspPosition {
                line: range.end.line,
                character: range.end.character,
            },
        },
    }
}

#[test]
fn complete_original_classy_merged_value_maps_to_its_actual_fourth_binding() {
    let original = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lsp/rename-source-authority/8009/Classy.vue.txt"
    ));
    for newline in ["\n", "\r\n"] {
        let source = original.replace('\n', newline);
        let uri = Url::parse("file:///workspace/Classy.vue").unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.clone(), 1, "vue".into());
        let document = host_document(&uri, &source);
        let code = &document.virtual_result.code;
        let literal = "\"class\": [\"clear-button\", { 'with-gap': hasSuffix }]";
        assert_eq!(
            code.match_indices(literal).count(),
            1,
            "complete unchanged mapped class array"
        );
        let native_start = code.find(literal).unwrap() + literal.find("hasSuffix").unwrap();
        let authored_start = source.find("'with-gap': hasSuffix").unwrap() + "'with-gap': ".len();
        let native = range(code, native_start, native_start + "hasSuffix".len());
        let authored = range(&source, authored_start, authored_start + "hasSuffix".len());
        let ctx =
            IdeContext::new(&state, &uri, source.find("const hasSuffix").unwrap() + 6).unwrap();
        assert_eq!(
            super::super::exact_edits::map_exact_range(&source, &document.virtual_result, native),
            Some(authored)
        );
        assert_eq!(
            super::super::map_canonical_corsa_location(
                &ctx,
                &document,
                &read_location(&document.request_uri, native)
            ),
            Some(Location {
                uri: uri.clone(),
                range: authored
            })
        );
        let edit = map_canonical_corsa_workspace_edit(
            &ctx,
            &document,
            WorkspaceEdit {
                changes: Some(
                    [(
                        Url::parse(&document.request_uri).unwrap(),
                        vec![TextEdit {
                            range: native,
                            new_text: "showSuffix".into(),
                        }],
                    )]
                    .into(),
                ),
                ..WorkspaceEdit::default()
            },
        )
        .unwrap();
        assert_eq!(
            edit,
            WorkspaceEdit {
                changes: Some(
                    [(
                        uri.clone(),
                        vec![TextEdit {
                            range: authored,
                            new_text: "showSuffix".into()
                        }]
                    )]
                    .into()
                ),
                ..WorkspaceEdit::default()
            }
        );
        let actual = &edit.changes.as_ref().unwrap()[&uri][0];
        let start = position_to_offset(
            &source,
            actual.range.start.line,
            actual.range.start.character,
        )
        .unwrap();
        let end =
            position_to_offset(&source, actual.range.end.line, actual.range.end.character).unwrap();
        let mut applied = source.clone();
        applied.replace_range(start..end, &actual.new_text);
        assert_eq!(
            applied,
            source.replace("'with-gap': hasSuffix", "'with-gap': showSuffix")
        );
        assert!(applied.contains("item-kind=\"small\""));
    }
}

#[test]
fn each_repeated_merged_binding_keeps_its_own_lf_crlf_and_astral_source_range() {
    for newline in ["\n", "\r\n"] {
        let source = vize_l0::cstr!(
            "<script setup lang=\"ts\">{newline}import Child from './Child.vue'; const value = true;{newline}</script>{newline}<template>😀<Child class=\"base\" :class=\"{{ 'one': value }}\" :class=\"{{ 'two': value }}\" /></template>"
        );
        let uri = Url::parse("file:///workspace/Parent.vue").unwrap();
        let document = host_document(&uri, &source);
        let code = &document.virtual_result.code;
        let literal = "\"class\": [\"base\", { 'one': value }, { 'two': value }]";
        let start = code
            .find(literal)
            .expect("whole unchanged three-value array");
        for prefix in ["'one': value", "'two': value"] {
            let native = start + literal.find(prefix).unwrap() + prefix.len() - "value".len();
            let authored = source.find(prefix).unwrap() + prefix.len() - "value".len();
            assert_eq!(
                super::super::exact_edits::map_exact_range(
                    &source,
                    &document.virtual_result,
                    range(code, native, native + "value".len())
                ),
                Some(range(&source, authored, authored + "value".len()))
            );
        }
        assert_eq!(
            super::super::exact_edits::map_exact_range(
                &source,
                &document.virtual_result,
                range(code, start, start + literal.len())
            ),
            None,
            "the merged container does not own one contiguous authored value"
        );
    }
}

#[test]
fn identical_rewritten_merged_values_do_not_steal_each_others_authored_key() {
    for newline in ["\n", "\r\n"] {
        let source = vize_l0::cstr!(
            "<script setup lang=\"ts\">{newline}import Child from './Child.vue'; defineProps<{{ static: boolean }}>();{newline}</script>{newline}<template>😀<Child class=\"base\" :class=\"{{ 'one': static }}\" :class=\"{{ 'one': static }}\" /></template>"
        );
        let uri = Url::parse("file:///workspace/Parent.vue").unwrap();
        let document = host_document(&uri, &source);
        let code = &document.virtual_result.code;
        let literal =
            "\"class\": [\"base\", { 'one': props[\"static\"] }, { 'one': props[\"static\"] }]";
        let start = code
            .find(literal)
            .expect("whole repeated rewritten class array");
        let generated = literal
            .match_indices("props[\"static\"]")
            .map(|(offset, _)| start + offset + "props[\"".len())
            .collect::<Vec<_>>();
        let authored = source
            .match_indices(":class=\"{ 'one': static }")
            .map(|(offset, _)| offset + ":class=\"{ 'one': ".len())
            .collect::<Vec<_>>();
        assert_eq!(generated.len(), 2);
        assert_eq!(authored.len(), 2);
        for (native, original) in generated.into_iter().zip(authored) {
            assert_eq!(
                super::super::exact_edits::map_exact_range(
                    &source,
                    &document.virtual_result,
                    range(code, native, native + "static".len())
                ),
                Some(range(&source, original, original + "static".len()))
            );
        }
    }
}
