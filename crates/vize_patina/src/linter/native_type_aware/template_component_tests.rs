use super::{RULE_NO_UNSAFE_TEMPLATE_BINDING, lint_sfc_with_corsa, tests::corsa_available};
use crate::{LintPreset, Linter};

#[test]
fn camelized_native_shorthand_retains_safe_and_unsafe_types() {
    if !corsa_available() {
        return;
    }

    let source = include_str!("../../../tests/fixtures/same-name-native-types.vue");
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, source, "CamelizedShorthand.vue");
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING)
        .collect();
    assert_eq!(
        warnings.len(),
        1,
        "unexpected unsafe bindings: {warnings:?}"
    );
    let unsafe_start = source.find(":aria-description").unwrap() as u32 + 1;
    assert_eq!(warnings[0].start, unsafe_start);
    assert_eq!(
        warnings[0].end,
        unsafe_start + "aria-description".len() as u32
    );
}

#[test]
fn typed_dynamic_component_target_is_a_safe_template_binding() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
type RenderTarget =
  | keyof HTMLElementTagNameMap
  | object
  | ((...args: never[]) => unknown)

const props = defineProps<{
  readonly as: RenderTarget
}>()
</script>

<template>
  <component :is="props.as"></component>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "DynamicFixture.vue");

    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING),
        "a typed render target should retain a safe template type: {:?}",
        result.diagnostics
    );
}
