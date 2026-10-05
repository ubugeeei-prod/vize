#![expect(
    clippy::disallowed_macros,
    reason = "whole compatibility fixtures and virtual-document snapshots use owned test strings"
)]

use super::tests::{READ, WRITE, highlights_at, state_for};
use crate::ide::CodeLensService;
use serde_json::json;

#[test]
fn script_only_setup_keeps_both_real_occurrences_without_string_or_comment_uses() {
    let input =
        "<script setup lang=\"ts\">\nconst value=1\nvalue\nconst text='value'\n// value\n</script>";
    for crlf in [false, true] {
        let source = if crlf {
            input.replace('\n', "\r\n")
        } else {
            input.into()
        };
        let (state, uri) = state_for(&source, "file:///ScriptOnly.vue", "vue");
        let packet = state.binding_occurrence_facts(&uri, &source).unwrap();
        assert!(!packet.is_legacy());
        assert!(packet.authored().is_some());
        for (line, column) in [(1, 7), (2, 2)] {
            assert_eq!(
                highlights_at(&state, &uri, &source, line, column),
                vec![(1, 6, 11, WRITE), (2, 0, 5, READ)]
            );
        }
        for (line, column) in [(3, 13), (4, 4)] {
            assert!(highlights_at(&state, &uri, &source, line, column).is_empty());
        }
        assert!(CodeLensService::get_lenses(&state, &source, &uri).is_empty());
    }
}

#[test]
fn plain_programs_keep_the_entire_existing_positive_route() {
    let source = "const value=1\nvalue\n";
    for (path, language) in [
        ("Plain.ts", "typescript"),
        ("Plain.js", "javascript"),
        ("Plain.tsx", "typescriptreact"),
        ("Plain.jsx", "javascriptreact"),
    ] {
        let uri = format!("file:///{path}");
        let (state, uri) = state_for(source, &uri, language);
        let packet = state.binding_occurrence_facts(&uri, source).unwrap();
        assert!(packet.is_legacy());
        assert!(packet.authored().is_none());
        assert_eq!(
            highlights_at(&state, &uri, source, 0, 7),
            vec![(0, 6, 11, WRITE), (1, 0, 5, READ)]
        );
    }
}

#[test]
fn unknown_template_ownership_keeps_legacy_vectors_without_claiming_authored_facts() {
    for header in [
        "<template lang=\"pug\">",
        "<template src=\"./external.html\">",
    ] {
        let body = if header.contains("pug") {
            "p= value"
        } else {
            "{{ value }}"
        };
        let source =
            format!("<script setup>\nconst value=1\n</script>\n{header}\n{body}\n</template>");
        let (state, uri) = state_for(&source, "file:///Unknown.vue", "vue");
        let packet = state.binding_occurrence_facts(&uri, &source).unwrap();
        assert!(packet.is_legacy());
        assert!(packet.authored().is_none());
        assert_eq!(
            highlights_at(&state, &uri, &source, 1, 7),
            vec![(1, 6, 11, WRITE), (4, 3, 8, READ)]
        );
        assert_eq!(
            serde_json::to_value(CodeLensService::get_lenses(&state, &source, &uri)).unwrap(),
            json!([{"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":0}},
                "command":{"title":"1 template/style reference","command":"vize.findReferences"}}])
        );
    }
}

#[test]
fn same_walk_script_only_capture_preserves_the_complete_virtual_document() {
    let source = "<script setup>\nconst value=1\nvalue\n</script>";
    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
    let mut ordinary = crate::virtual_code::VirtualCodeGenerator::new();
    let mut demanded = crate::virtual_code::VirtualCodeGenerator::new();
    let before = ordinary.generate(&descriptor, "ScriptOnly.vue");
    let (after, packet) = demanded.generate_with_occurrences(&descriptor, "ScriptOnly.vue", source);
    assert!(packet.is_some());
    assert_eq!(format!("{before:?}"), format!("{after:?}"));
}
