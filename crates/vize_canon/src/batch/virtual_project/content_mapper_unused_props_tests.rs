use super::{ContentMapperTransformOptions, generate_vue_content_mapper_transform_with_options};
use std::path::Path;

#[test]
fn unused_diagnostic_setting_does_not_anchor_unread_destructured_props() {
    let source = r#"<script setup lang="ts">
const { label, color = "gray" } = defineProps<{ label: string; color?: string }>()
const unusedLocal = 1
</script>
<template><span>{{ label }}</span></template>
"#;

    let result = generate_vue_content_mapper_transform_with_options(
        Path::new("MyBadge.vue"),
        source,
        ContentMapperTransformOptions::default().with_preserve_unused_diagnostics(true),
    )
    .expect("transform");

    assert_eq!(result.text.matches("void label;").count(), 1);
    assert_eq!(result.text.matches("void color;").count(), 0);
    assert_eq!(result.text.matches("void unusedLocal;").count(), 0);
}
