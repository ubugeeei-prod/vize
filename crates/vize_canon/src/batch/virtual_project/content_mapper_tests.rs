use std::path::Path;

use crate::batch::generate_vue_content_mapper_transform;

#[test]
fn emits_protocol_v1_spans_without_forbidden_overlaps() {
    let source = r#"<script setup lang="ts">
const message = "hello"
</script>
<template>{{ message }}</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");

    assert_eq!(result.script_kind, 3);
    assert!(result.text.contains("const message = \"hello\""));
    assert!(
        !result
            .text
            .contains("/// <reference types=\"vite/client\" />")
    );
    assert!(!result.mappings.is_empty());
    for pair in result.mappings.windows(2) {
        let left = pair[0].0;
        let right = pair[1].0;
        assert!(left[0] + left[1] <= right[0], "{left:?} overlaps {right:?}");
    }
    for (index, left) in result.mappings.iter().enumerate() {
        for right in &result.mappings[index + 1..] {
            let left = left.0;
            let right = right.0;
            let originals_overlap = left[2] < right[2] + right[3] && right[2] < left[2] + left[3];
            let originals_match = left[2] == right[2] && left[3] == right[3];
            assert!(
                !originals_overlap || originals_match,
                "{left:?} partially overlaps {right:?}"
            );
        }
    }
    assert!(result.mappings.iter().all(|mapping| mapping.0[5] == 3));
}

#[test]
fn keeps_mapper_offsets_in_utf8_bytes() {
    let source = r#"<script setup lang="ts">
const emoji = "😀"
</script>
<template>{{ emoji }}</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("Unicode.vue"), source).expect("transform");
    let original = source.rfind("emoji").expect("template identifier");
    assert!(
        result
            .mappings
            .iter()
            .any(|mapping| mapping.0[2] == original),
        "expected a UTF-8 byte mapping at {original}: {:?}",
        result.mappings
    );
}

#[test]
fn authored_parse_errors_are_mapper_diagnostics() {
    let source = "<template><div></template>";
    let result =
        generate_vue_content_mapper_transform(Path::new("Broken.vue"), source).expect("transform");

    assert!(!result.diagnostics.is_empty());
    assert!(result.mappings.is_empty());
    assert!(result.text.contains("__vize_component"));
    assert!(result.diagnostics[0].start <= source.len());
}

#[test]
fn jsx_scripts_report_tsx_script_kind() {
    let source = "<script lang=\"tsx\">export default () => <div /></script>";
    let result =
        generate_vue_content_mapper_transform(Path::new("Jsx.vue"), source).expect("transform");

    assert_eq!(result.script_kind, 4);
    assert!(
        result
            .text
            .starts_with("/// <reference types=\"vue/jsx\" />")
    );
}
