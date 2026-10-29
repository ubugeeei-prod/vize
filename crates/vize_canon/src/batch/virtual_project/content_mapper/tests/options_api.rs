use std::path::Path;

use super::super::{
    ContentMapperTransformOptions, generate_vue_content_mapper_transform_with_options,
};

pub(super) fn assert_transform_setting_controls_instance_bindings() {
    let source = r#"<script lang="ts">
export default {
  data() { return { count: 1 } }
}
</script>
<template>{{ count }}:{{ missing }}</template>
"#;
    let enabled = generate_vue_content_mapper_transform_with_options(
        Path::new("Options.vue"),
        source,
        ContentMapperTransformOptions::default().with_options_api(true),
    )
    .expect("enabled transform");
    let disabled = generate_vue_content_mapper_transform_with_options(
        Path::new("Options.vue"),
        source,
        ContentMapperTransformOptions::default().with_options_api(false),
    )
    .expect("disabled transform");

    assert_eq!(
        enabled
            .text
            .matches("var count: $OptionsBinding<typeof __default__, \"count\">")
            .count(),
        1,
        "enabled Options API must emit one typed lexical binding:\n{}",
        enabled.text
    );
    assert_eq!(
        disabled.text.matches("$OptionsBinding").count(),
        0,
        "disabled Options API must not emit typed bindings:\n{}",
        disabled.text
    );
    assert_eq!(
        disabled
            .text
            .matches("$TemplateInstanceMember<\"count\">")
            .count(),
        0,
        "disabled Options API members must remain authored TS2304 reads:\n{}",
        disabled.text
    );
    assert_eq!(
        disabled
            .text
            .matches("void (__vize_public_instance.count);")
            .count(),
        0,
        "disabled Options API members must not become TS2339 reads:\n{}",
        disabled.text
    );
    assert_eq!(
        disabled
            .text
            .matches("void (__vize_public_instance.missing);")
            .count(),
        1,
        "unknown names must retain public-instance resolution:\n{}",
        disabled.text
    );
}
