use std::path::Path;

use crate::batch::{
    ContentMapperTransformOptions, generate_vue_content_mapper_transform,
    generate_vue_content_mapper_transform_with_options,
};

#[test]
fn installed_mapper_imports_context_free_helpers_without_ambient_macros() {
    let source = r#"<script setup lang="ts">
import Child from './Child.vue'
defineProps<{ label: string }>()
</script>
<template><Child :label="label" /></template>"#;
    let shared = generate_vue_content_mapper_transform_with_options(
        Path::new("Consumer.vue"),
        source,
        ContentMapperTransformOptions::default().with_external_component_attrs_module(true),
    )
    .expect("installed mapper");
    for alias in [
        "__DefineProps",
        "__VizeComponentInput",
        "__VizeFallthroughAttrs",
        "__VizeReplaceComponentProps",
    ] {
        assert!(shared.text.contains(alias), "{alias} must be imported");
        assert!(
            !shared.text.contains(&format!("type {alias}<")),
            "{alias} must share one declaration identity"
        );
    }
    assert!(shared.text.contains("function defineProps<"));
    let standalone = generate_vue_content_mapper_transform(Path::new("Standalone.vue"), source)
        .expect("standalone mapper");
    assert!(standalone.text.contains("type __VizeComponentInput<"));
    assert!(!standalone.text.contains("vize/content-mapper-helpers"));
}
