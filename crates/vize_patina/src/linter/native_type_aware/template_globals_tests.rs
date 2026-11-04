use super::tests::assertion::assert_absent_note;
use super::{RULE_NO_UNSAFE_TEMPLATE_BINDING, lint_sfc_with_corsa, tests::corsa_available};
use crate::{LintPreset, Linter};

#[test]
fn public_instance_attrs_are_a_safe_template_spread() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const visible = true
</script>

<template>
  <div v-if="visible" v-bind="$attrs"></div>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "AttrsFixture.vue");

    assert_absent_note(
        &result,
        RULE_NO_UNSAFE_TEMPLATE_BINDING,
        "public instance attributes should retain a safe structural type",
    );
}
