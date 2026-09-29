use std::path::Path;

use crate::batch::generate_vue_content_mapper_transform;

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
