#![expect(clippy::string_slice, reason = "tests assert by panicking")]
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
    let resolved_value = result.text[resolver..]
        .find("\"value\": conditionalValue")
        .expect("resolved value");
    assert!(resolver + resolved_value > guard, "{}", result.text);
    assert_eq!(
        result.text.matches("const __vize_events_resolved_").count(),
        1
    );
    let handler_inference = result
        .text
        .find("const __vize_emit_props_")
        .expect("handler inference");
    let direct_inference = result.text[handler_inference..]
        .find("= (undefined as unknown as __ConditionalGenericChild_")
        .expect("direct handler inference");
    let inferred_value = result.text[handler_inference..]
        .find("\"value\": conditionalValue")
        .expect("inferred value");
    assert!(direct_inference < inferred_value, "{}", result.text);

    let original = source.find("@confirm").unwrap() + 1;
    assert!(result.mappings.iter().any(|mapping| {
        mapping.0[2] == original
            && mapping.0[3] == "confirm".len()
            && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_ALL
    }));
}
