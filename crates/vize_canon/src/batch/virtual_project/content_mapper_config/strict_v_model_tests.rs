//! `strictVModel` reaches the Content Mapper projection (#11643).
//!
//! `vize check` checks what a component `v-model` writes back under
//! `strictVModel` or `strictTemplates`. The mapper read neither, so the
//! `update:modelValue` payload check was never emitted and TypeScript reported
//! nothing for a payload that does not fit the bound ref.

use std::path::Path;

use super::super::generate_vue_content_mapper_transform_with_options;
use super::ContentMapperProjectConfiguration;

const PAGE: &str = r#"<script setup lang="ts">
import { ref } from "vue";
import KitField from "./KitField.vue";
const climate = ref<"warm" | "cold">("warm");
</script>
<template>
  <KitField v-model="climate" label="Plant" />
</template>
"#;

/// The write-back statement `model_update` emits for the bound target.
const WRITE_BACK: &str = "climate = __vize_model_value;";

fn projection(vue_compiler_options: &str) -> vize_l0::String {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tsconfig.json");
    std::fs::write(
        &path,
        vize_l0::cstr!(r#"{{"vueCompilerOptions":{{{vue_compiler_options}}}}}"#).as_str(),
    )
    .unwrap();
    let configuration = ContentMapperProjectConfiguration::load(&path).unwrap();
    generate_vue_content_mapper_transform_with_options(
        Path::new("src/Page.vue"),
        PAGE,
        configuration.transform_options,
    )
    .unwrap()
    .text
}

#[test]
fn strict_templates_checks_the_model_write_back() {
    let text = projection(r#""strictTemplates": true"#);
    assert_eq!(text.matches(WRITE_BACK).count(), 1, "{text}");
}

#[test]
fn strict_v_model_overrides_strict_templates() {
    let on = projection(r#""strictVModel": true"#);
    assert_eq!(on.matches(WRITE_BACK).count(), 1, "{on}");
    let off = projection(r#""strictTemplates": true, "strictVModel": false"#);
    assert_eq!(off.matches(WRITE_BACK).count(), 0, "{off}");
}

#[test]
fn default_options_leave_the_model_write_back_unchecked() {
    let text = projection("");
    assert_eq!(text.matches(WRITE_BACK).count(), 0, "{text}");
}

const MISSING: &str = r#"<script setup lang="ts">
import { reactive } from "vue";
import KitField from "./KitField.vue";
const state = reactive({ name: "" });
</script>
<template>
  <KitField v-model="state.missing" />
</template>
"#;

/// `protocol::DIRECTIVE_POLICY_IGNORE`.
const IGNORE: usize = 0;

/// Whether an IGNORE directive suppresses a diagnostic starting at `at`.
fn ignored(transform: &crate::batch::ContentMapperTransform, at: usize) -> bool {
    transform
        .diagnostic_directives
        .iter()
        .flat_map(|table| &table.directives)
        .any(|directive| {
            let [_, _, start, end, policy, _] = directive.0;
            policy == IGNORE && start <= at && at < end
        })
}

/// The write-back reads the target as well as writing it, so it owns the
/// target's own errors. The props literal's copy of the target is its echo,
/// or `state.missing` would report TS2339 once per copy.
#[test]
fn write_back_owns_the_bound_target_diagnostics() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tsconfig.json");
    std::fs::write(&path, r#"{"vueCompilerOptions":{"strictTemplates":true}}"#).unwrap();
    let configuration = ContentMapperProjectConfiguration::load(&path).unwrap();
    let transform = generate_vue_content_mapper_transform_with_options(
        Path::new("src/Page.vue"),
        MISSING,
        configuration.transform_options,
    )
    .unwrap();
    let text = transform.text.as_str();
    let write_back = text
        .find("state.missing = __vize_model_value")
        .expect("write-back");
    let literal = text
        .find("\"modelValue\": state.missing")
        .expect("props literal")
        + "\"modelValue\": ".len();
    let authored = MISSING.find("state.missing").unwrap();
    let maps_authored = |at: usize| {
        transform.mappings.iter().any(|span| {
            let [generated, length, original, original_length, _, _] = span.0;
            generated == at
                && length == "state.missing".len()
                && original == authored
                && original_length == "state.missing".len()
        })
    };
    assert!(
        maps_authored(write_back) && !ignored(&transform, write_back),
        "{text}"
    );
    assert!(
        maps_authored(literal) && ignored(&transform, literal),
        "{text}"
    );
}
