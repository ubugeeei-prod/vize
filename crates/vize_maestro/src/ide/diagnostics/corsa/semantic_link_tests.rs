#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use crate::DiagnosticService;
use tower_lsp::lsp_types::Url;

#[test]
fn plain_export_links_join_the_authored_declaration_to_the_public_alias() {
    let uri = Url::parse("file:///tmp/Plain.vue").unwrap();
    let source = "<script lang=\"ts\">\nexport const shared = 1;\nconst copy = shared;\n</script>\n<template><div /></template>";
    let result = DiagnosticService::generate_virtual_ts(&uri, source, false, false).unwrap();
    let endpoints: Vec<_> = result
        .semantic_links
        .iter()
        .map(|link| {
            (
                &result.code[link.source_range.clone()],
                &result.code[link.target_range.clone()],
                link.kind,
            )
        })
        .collect();
    assert_eq!(
        endpoints,
        vec![(
            "shared",
            "shared",
            vize_canon::virtual_ts::VizeSemanticLinkKind::VuePlainScriptExport
        )],
        "code: {}\nmappings: {:?}",
        result.code,
        result.source_mappings
    );
}

#[test]
fn editor_semantic_links_match_rewritten_virtual_ts_after_vue_imports() {
    let uri = Url::parse("file:///tmp/Host.vue").expect("parse uri");
    let content = r#"<script setup lang="ts">
import Child from './Child.vue'
import { ref } from 'vue'
const icon = "😀"
const café = ref(1)
void Child
</script>
<template>{{ icon }}{{ café }}</template>"#;
    let result = DiagnosticService::generate_virtual_ts(&uri, content, false, false)
        .expect("virtual ts generated");
    assert!(
        result.code.contains("'./Child.vue.ts'"),
        "expected rewritten import before semantic-link endpoints:\n{}",
        result.code
    );
    let link = result
        .semantic_links
        .iter()
        .find(|link| {
            &result.code[link.source_range.clone()] == "café"
                && &result.code[link.target_range.clone()] == "café"
        })
        .unwrap_or_else(|| {
            panic!(
                "semantic links must point into rewritten virtual TS:\ncode:\n{}\nlinks:\n{:#?}",
                result.code, result.semantic_links
            )
        });
    let (line, character) = crate::ide::offset_to_position(&result.code, link.target_range.start);
    assert_eq!(
        crate::ide::position_to_offset(&result.code, line, character),
        Some(link.target_range.start),
        "linked range must round-trip through UTF-16 LSP coordinates"
    );
}

#[test]
fn original_bare_prop_links_keep_emitted_binding_and_native_property_endpoints_after_rewrite() {
    use vize_canon::virtual_ts::VizeSemanticLinkKind;
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let imported = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lsp/bare-prop-definition/StatusBox.vue.txt"
    ));
    let local = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lsp/bare-prop-definition/StatusBoxLocal.vue.txt"
    ));
    for newline in ["\n", "\r\n"] {
        for source in [imported, local] {
            let source = ["<!-- 😀 -->\n", source].concat().replace('\n', newline);
            let result =
                DiagnosticService::generate_virtual_ts(&uri, &source, false, false).unwrap();
            let links: Vec<_> = result
                .semantic_links
                .iter()
                .filter(|link| link.kind == VizeSemanticLinkKind::VueTemplatePropBinding)
                .collect();
            assert_eq!(links.len(), 1);
            let link = links[0];
            assert_eq!(result.code.get(link.source_range.clone()), Some("status"));
            assert_eq!(result.code.get(link.target_range.clone()), Some("status"));
            assert_eq!(
                result
                    .code
                    .get(link.source_range.start - "const ".len()..link.source_range.start),
                Some("const ")
            );
            let property_prefix = &result.code[..link.target_range.start];
            assert!(
                property_prefix.ends_with("props[\"") || property_prefix.ends_with("props[(\"")
            );
            for endpoint in [link.source_range.start, link.target_range.start] {
                let (line, character) = crate::ide::offset_to_position(&result.code, endpoint);
                assert_eq!(
                    crate::ide::position_to_offset(&result.code, line, character),
                    Some(endpoint)
                );
            }
        }
    }
}
