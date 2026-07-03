use super::{RULE_NO_UNSAFE_TEMPLATE_BINDING, corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

#[test]
fn vue2_options_api_runtime_props_are_safe_template_bindings() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Nuxt).with_type_aware_lint(true);
    let source = r#"<template>
  <v-dialog :value="isOpened" :width="width">
    <span>{{ title }}</span>
  </v-dialog>
</template>

<script lang="ts">
import { defineComponent } from '@nuxtjs/composition-api';

export default defineComponent({
  props: {
    isOpened: { type: Boolean, required: true },
    title: { type: String, default: '' },
    width: { type: String, default: '1024px' },
  },
});
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "AppDialog.vue");

    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diag| diag.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING),
        "Options API props should stay typed in template: {:?}",
        result.diagnostics
    );
}
