//! Existing attribute-only opt-in and new type-sharing opt-in stay distinct.

use std::path::Path;

use crate::batch::{
    ContentMapperTransformOptions, generate_vue_content_mapper_transform_with_options,
};

mod aria_ownership;
mod aria_witness;

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

/// A module that embeds its helpers has no `vize/virtual-ts-checker` import,
/// so it declares vue-tsc's props fallback itself. Without it, an attribute a
/// generic library component does not declare (`data-slot`) is an excess
/// property in the mapper lanes but not in `vize check`.
#[test]
fn embedded_modules_declare_the_props_fallback_unless_unknown_props_are_checked() {
    for (check_unknown_props, attrs_module, shared_virtual_module, expected) in [
        (false, false, false, 1),
        (false, true, false, 1),
        (true, false, false, 0),
        (false, true, true, 0),
    ] {
        let mut options = ContentMapperTransformOptions::default()
            .with_external_component_attrs_module(attrs_module)
            .with_external_component_type_helpers(attrs_module)
            .with_shared_virtual_module(shared_virtual_module);
        options.checks.check_unknown_props = check_unknown_props;
        let packet = packet(options);
        let text = packet["text"].as_str().unwrap_or_default();
        assert_eq!(
            text.matches("var __VLS_PROPS_FALLBACK: Record<string, unknown>;")
                .count(),
            expected,
            "{text}"
        );
    }
}
