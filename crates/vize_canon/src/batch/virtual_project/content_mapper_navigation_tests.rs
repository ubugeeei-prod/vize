use std::path::Path;

use super::{
    CONTENT_MAPPER_SPAN_FEATURES_ALL, CONTENT_MAPPER_SPAN_FEATURES_ATOM, ContentMapperSpanKind,
    generate_vue_content_mapper_transform,
};

#[test]
fn maps_component_event_navigation_to_the_authored_name() {
    let source = r#"<script setup lang="ts">
import Child from "./Child.vue";
const handler = (value: number) => value;
</script>
<template><Child @sa="handler" /></template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");
    let original = source.find("@sa").unwrap() + 1;
    let navigation = result.text.find("__vize_events_nav_0.sa").unwrap();
    let generated = navigation + "__vize_events_nav_0.".len();
    let handler = result.text.find("// @sa handler").unwrap();

    assert!(
        navigation < handler,
        "completion projection must win reverse mapping"
    );
    assert!(result.mappings.iter().any(|mapping| {
        mapping.0[0] == generated
            && mapping.0[1] == "sa".len()
            && mapping.0[2] == original
            && mapping.0[3] == "sa".len()
            && mapping.0[4] == ContentMapperSpanKind::Verbatim as usize
            && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_ALL
    }));
    assert!(result.mappings.iter().any(|mapping| {
        mapping.0[2] == original
            && mapping.0[3] == "sa".len()
            && mapping.0[4] == ContentMapperSpanKind::Atom as usize
            && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_ATOM
    }));
}

#[test]
fn maps_exported_emits_type_to_the_authored_macro() {
    let source = r#"<script setup lang="ts">
defineEmits<{ save: [value: number] }>();
</script>
<template><button /></template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("Child.vue"), source).expect("transform");
    let original = source.find("save: [value").unwrap();
    let exported = result.text.find("export type Emits").unwrap();
    let generated = exported + result.text[exported..].find("save: [value").unwrap();

    assert!(result.mappings.iter().any(|mapping| {
        generated >= mapping.0[0]
            && generated < mapping.0[0] + mapping.0[1]
            && original >= mapping.0[2]
            && original < mapping.0[2] + mapping.0[3]
            && generated - mapping.0[0] == original - mapping.0[2]
            && mapping.0[4] == ContentMapperSpanKind::Verbatim as usize
    }));
}
