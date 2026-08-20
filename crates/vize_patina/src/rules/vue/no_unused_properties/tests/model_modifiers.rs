//! Framework-consumed modifier props generated alongside `defineModel`.

use super::{findings, lint_sfc, none, owned, unused};

#[test]
fn ignores_default_model_modifiers_for_a_generic_component() {
    let sfc = r#"<script setup lang="ts" generic="T extends string | number">
defineModel<T>({ default: "" as T });
defineProps<{
  modelModifiers?: Record<string, boolean>;
}>();
</script>

<template>
  <label>generic input</label>
</template>
"#;

    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn keeps_the_destructured_generic_component_repro_silent() {
    let sfc = r#"<script setup lang="ts" generic="T extends string | number">
const modelValue = defineModel<T>({ default: "" as T });
const { modelModifiers = {} } = defineProps<{
  modelModifiers?: Record<string, boolean>;
}>();
</script>

<template>
  <label>generic<input v-model="modelValue" /></label>
</template>
"#;

    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_named_model_modifiers() {
    let sfc = r#"<script setup lang="ts">
defineModel<string>("title");
defineProps<{
  titleModifiers?: Record<string, boolean>;
}>();
</script>

<template>
  <label>named model</label>
</template>
"#;

    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn reports_a_modifiers_suffix_without_a_matching_model() {
    let sfc = r#"<script setup lang="ts">
defineProps<{
  keyboardModifiers?: Record<string, boolean>;
}>();
</script>

<template>
  <label>ordinary prop</label>
</template>
"#;

    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(
            sfc,
            "keyboardModifiers",
            "keyboardModifiers?: Record<string, boolean>;"
        )]
    );
}
