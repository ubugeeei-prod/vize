use crate::DiagnosticService;
use tower_lsp::lsp_types::Url;

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
