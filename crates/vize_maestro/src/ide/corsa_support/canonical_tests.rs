use tower_lsp::lsp_types::Url;

use super::canonical::{
    CanonicalVirtualDocument, canonical_request_path, canonical_source_offset_to_position,
};
use super::request_file_uri;

#[test]
fn canonical_source_offset_maps_template_expression_to_generated_position() {
    let uri = Url::parse("file:///tmp/TypedTemplate.vue").expect("uri");
    let source = r#"<script setup lang="ts">
const user = { name: 'Ada' as string }
</script>

<template>
  {{ user.name }}
</template>
"#;
    let virtual_result =
        crate::ide::DiagnosticService::generate_virtual_ts(&uri, source, false, false)
            .expect("virtual ts");
    let doc = CanonicalVirtualDocument {
        request_uri: request_file_uri(canonical_request_path(&uri).as_str()),
        virtual_result,
    };

    let source_offset = source.rfind("name").unwrap() + "na".len();
    let (line, character) =
        canonical_source_offset_to_position(&doc, source_offset).expect("mapped position");
    let generated_offset =
        crate::ide::position_to_offset(&doc.virtual_result.code, line, character)
            .expect("generated offset");
    let expected_offset = doc.virtual_result.code.find("user.name").unwrap() + "user.na".len();

    assert_eq!(generated_offset, expected_offset);
}

#[test]
fn canonical_source_offset_accounts_for_vue_import_rewrite_before_script_body() {
    let uri = Url::parse("file:///tmp/Parent.vue").expect("uri");
    let source = r#"<script setup lang="ts">
import Child from "./Child.vue";
const selected = Child;
</script>
"#;
    let virtual_result =
        crate::ide::DiagnosticService::generate_virtual_ts(&uri, source, false, false)
            .expect("virtual ts");
    let doc = CanonicalVirtualDocument {
        request_uri: request_file_uri(canonical_request_path(&uri).as_str()),
        virtual_result,
    };

    let source_offset = source.rfind("Child").unwrap() + "Ch".len();
    let (line, character) =
        canonical_source_offset_to_position(&doc, source_offset).expect("mapped position");
    let generated_offset =
        crate::ide::position_to_offset(&doc.virtual_result.code, line, character)
            .expect("generated offset");
    let expected_offset = doc.virtual_result.code.rfind("Child").unwrap() + "Ch".len();

    assert_eq!(generated_offset, expected_offset);
}
