use std::path::Path;

use super::{ContentMapperTransformOptions, generate_vue_content_mapper_transform_with_options};

const SOURCE: &str = r#"<script setup lang="ts">
const n = 1
</script>
<template>
  <NotImported />
  <p v-not-registered>{{ n }}</p>
</template>
"#;

#[test]
fn unknown_component_and_directive_options_reach_the_mapper_projection() {
    let enabled = generate_vue_content_mapper_transform_with_options(
        Path::new("App.vue"),
        SOURCE,
        ContentMapperTransformOptions {
            check_unknown_components: true,
            check_unknown_directives: true,
            ..Default::default()
        },
    )
    .unwrap();
    let disabled = generate_vue_content_mapper_transform_with_options(
        Path::new("App.vue"),
        SOURCE,
        ContentMapperTransformOptions {
            check_unknown_components: false,
            check_unknown_directives: false,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(enabled.text.matches("__vize_global_component_").count(), 2);
    assert_eq!(enabled.text.matches("__vize_unknown_directive_").count(), 2);
    assert_eq!(disabled.text.matches("__vize_global_component_").count(), 0);
    assert_eq!(
        disabled.text.matches("__vize_unknown_directive_").count(),
        0
    );
}
