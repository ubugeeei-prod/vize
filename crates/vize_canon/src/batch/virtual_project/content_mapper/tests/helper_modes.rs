//! Existing attribute-only opt-in and new type-sharing opt-in stay distinct.

use std::path::Path;

use crate::batch::{
    ContentMapperTransformOptions, generate_vue_content_mapper_transform_with_options,
};

fn packet(options: ContentMapperTransformOptions) -> serde_json::Value {
    let source = "<script setup lang='ts'>const props = defineProps<{label: string}>();</script>\n<template>{{ props.label }}</template>";
    serde_json::to_value(
        generate_vue_content_mapper_transform_with_options(
            Path::new("src/App.vue"),
            source,
            options,
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn existing_attribute_helper_opt_in_retains_the_complete_legacy_packet() {
    let attrs = ContentMapperTransformOptions::default().with_external_component_attrs_module(true);
    assert_eq!(
        packet(attrs),
        packet(attrs.with_external_component_type_helpers(false))
    );
    assert_ne!(
        packet(attrs),
        packet(attrs.with_external_component_type_helpers(true))
    );
}

#[test]
fn type_sharing_alone_does_not_change_self_contained_packets() {
    let standalone = ContentMapperTransformOptions::default();
    assert_eq!(
        packet(standalone),
        packet(standalone.with_external_component_type_helpers(true))
    );
}
