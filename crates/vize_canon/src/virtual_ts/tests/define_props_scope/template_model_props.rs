//! Template reads of `defineModel()` props resolve to the props projection
//! (misskey MkTab: `modelValue === option.key`).

use crate::batch::{ImportRewriter, generate_vue_document_virtual_ts};
use crate::virtual_ts::VirtualTsOptions;
use std::path::Path;

fn generate(source: &str) -> vize_carton::String {
    generate_vue_document_virtual_ts(
        Path::new("Tab.vue"),
        source,
        &VirtualTsOptions::default(),
        &ImportRewriter::new(),
        false,
    )
    .unwrap()
    .pre_rewrite_code
}

/// Trimmed generated lines declaring or reading `name` off the public instance.
fn model_value_lines(code: &str) -> Vec<&str> {
    code.lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("const modelValue")
                || line.starts_with("var modelValue")
                || line.starts_with("void (__vize_public_instance.modelValue")
        })
        .collect()
}

#[test]
fn model_props_read_in_template_are_not_unknown_instance_members() {
    let code = generate(
        r#"<script setup lang="ts">
defineProps<{ tabs: string[] }>();
const model = defineModel<string>();
void model;
</script>
<template><div v-for="t in tabs" :key="t" :class="{ a: modelValue === t }">{{ t }}</div></template>"#,
    );
    assert_eq!(
        model_value_lines(&code),
        ["const modelValue = props[\"modelValue\"];"],
        "{code}"
    );
}

#[test]
fn undeclared_names_beside_model_props_stay_unknown() {
    let code = generate(
        r#"<script setup lang="ts">
const count = defineModel<number>('count');
void count;
</script>
<template><div>{{ count }} {{ modelValue }}</div></template>"#,
    );
    assert_eq!(
        model_value_lines(&code),
        [
            "var modelValue!: $TemplateInstanceMember<\"modelValue\">; void (modelValue);",
            "void (__vize_public_instance.modelValue);",
        ],
        "{code}"
    );
}
