use std::path::Path;

use super::{CONTENT_MAPPER_SPAN_FEATURES_ALL, generate_vue_content_mapper_transform};

#[test]
fn emits_dynamic_generic_event_navigation_inside_the_v_if_guard() {
    let source = r#"<script setup lang="ts">
import ConditionalGenericChild from "./ConditionalGenericChild.vue";
const conditionalValue: "conditional" | null = null;
const handleConfirm = (value: "conditional") => value;
</script>
<template>
  <ConditionalGenericChild
    v-if="conditionalValue"
    :value="conditionalValue"
    @confirm="handleConfirm"
  />
</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");
    let guard_comment = result
        .text
        .find("Navigation-only guard")
        .expect("navigation guard comment");
    let guard = guard_comment
        + result.text[guard_comment..]
            .find("if ((conditionalValue))")
            .expect("navigation guard");
    let resolver = result
        .text
        .find("const __vize_events_resolved_")
        .expect("event resolver");

    assert!(guard < resolver, "{}", result.text);
    assert!(
        result.text[resolver..].contains("\"value\": conditionalValue"),
        "{}",
        result.text
    );
    assert_eq!(
        result.text.matches("const __vize_events_resolved_").count(),
        1
    );
    let handler_inference = result
        .text
        .find("const __vize_emit_props_")
        .expect("handler inference");
    assert!(
        result.text[handler_inference..].contains("= (() => {")
            && result.text[handler_inference..]
                .contains("if ((conditionalValue)) return (undefined as unknown as"),
        "{}",
        result.text
    );

    let original = source.find("@confirm").unwrap() + 1;
    assert!(result.mappings.iter().any(|mapping| {
        mapping.0[2] == original
            && mapping.0[3] == "confirm".len()
            && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_ALL
    }));
}
