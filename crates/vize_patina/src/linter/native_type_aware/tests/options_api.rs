use super::assertion::{assert_absent_note, assert_reports};
use super::{RULE_NO_UNSAFE_TEMPLATE_BINDING, corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

#[test]
fn vue3_options_api_boolean_props_are_safe_template_bindings() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE_NO_UNSAFE_TEMPLATE_BINDING.into()]))
        .with_type_aware_lint(true);
    let source = r#"<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    title: String,
    open: Boolean,
  },
})
</script>

<template>
  <span :data-open="open">{{ title }}</span>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert_absent_note(
        &result,
        RULE_NO_UNSAFE_TEMPLATE_BINDING,
        "runtime Options API Boolean props must retain their type (#13392)",
    );
    let unsafe_source = r#"<script setup lang="ts">const value: any = 1</script>
<template><span :data-open="value" /></template>"#;
    let unsafe_result = lint_sfc_with_corsa(&linter, unsafe_source, "UnsafeControl.vue");
    assert_reports(&unsafe_result, RULE_NO_UNSAFE_TEMPLATE_BINDING);
}

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

    assert_absent_note(
        &result,
        RULE_NO_UNSAFE_TEMPLATE_BINDING,
        "Options API props should stay typed in template",
    );
}
