use super::{RULE_NO_UNSAFE_TEMPLATE_BINDING, lint_sfc_with_corsa, tests::corsa_available};
use crate::{LintPreset, Linter};

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
