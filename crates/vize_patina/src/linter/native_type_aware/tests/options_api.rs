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
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/linter/options-api-boolean-prop-13392/Component.vue.txt"
    ));
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
fn options_api_prop_annotations_preserve_unsafe_shadowing_loop_and_slot_bindings() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE_NO_UNSAFE_TEMPLATE_BINDING.into()]))
        .with_type_aware_lint(true);
    for template in [
        r#"<span v-for="open in items" :data-open="open" />"#,
        r#"<Unknown v-slot="{ open }"><span :data-open="open" /></Unknown>"#,
        r#"<Unknown v-slot="{ value: open }"><span :data-open="open" /></Unknown>"#,
    ] {
        let source = format!(
            r#"<script lang="ts">
export default {{
  props: {{ open: {{ type: Boolean, required: true }} }},
  data() {{ return {{ items: [] as any[] }} }},
}}
</script>
<template><span :data-open="open" />{template}<span :data-open="open" /></template>"#
        );
        let result = lint_sfc_with_corsa(&linter, &source, "ShadowedOptionsProp.vue");
        let shadowed = source.find(template).unwrap()
            + template.find(":data-open=\"open\"").unwrap()
            + ":data-open=\"".len();
        assert!(
            result.diagnostics.iter().any(|diagnostic| {
                diagnostic.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING
                    && diagnostic.start as usize == shadowed
                    && diagnostic.end as usize == shadowed + "open".len()
            }),
            "the shadowing binding must remain unsafe: {template}\n{result:#?}"
        );
        for unshadowed in [
            source.find(":data-open=\"open\"").unwrap(),
            source.rfind(":data-open=\"open\"").unwrap(),
        ] {
            let offset = unshadowed + ":data-open=\"".len();
            assert!(
                !result.diagnostics.iter().any(|diagnostic| {
                    diagnostic.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING
                        && diagnostic.start as usize == offset
                }),
                "the sibling prop must retain its Boolean type: {result:#?}"
            );
        }
    }
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

#[test]
fn options_api_boolean_props_with_comments_keep_their_checked_type() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE_NO_UNSAFE_TEMPLATE_BINDING.into()]))
        .with_type_aware_lint(true);
    for expression in [
        "open /* explanation */",
        "/* explanation */ open",
        "open // explanation\n",
    ] {
        let source = format!(
            r#"<script lang="ts">
export default {{ props: {{ open: {{ type: Boolean, required: true }} }} }};
</script><template><span :data-open="{expression}" /></template>"#
        );
        let result = lint_sfc_with_corsa(&linter, &source, "CommentedOptionsProp.vue");
        assert_absent_note(&result, RULE_NO_UNSAFE_TEMPLATE_BINDING, expression);
    }
}

#[test]
fn options_api_setup_and_data_bindings_shadow_props_without_hiding_any() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE_NO_UNSAFE_TEMPLATE_BINDING.into()]))
        .with_type_aware_lint(true);
    for option in ["setup", "data"] {
        let source = format!(
            r#"<script lang="ts">
export default {{
  props: {{ open: {{ type: Boolean, required: true }} }},
  {option}() {{ return {{ open: 1 as any }}; }},
}};
</script><template><span :data-open="open" /></template>"#
        );
        let result = lint_sfc_with_corsa(&linter, &source, "SetupShadowsProp.vue");
        assert_reports(&result, RULE_NO_UNSAFE_TEMPLATE_BINDING);
    }
}

#[test]
fn options_api_explicit_any_props_stay_unsafe() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE_NO_UNSAFE_TEMPLATE_BINDING.into()]))
        .with_type_aware_lint(true);
    let source = r#"<script lang="ts">
import type { PropType } from 'vue';
export default { props: { open: { type: null as unknown as PropType<any>, required: true } } };
</script><template><span :data-open="open" /></template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "AnyOptionsProp.vue");
    assert_reports(&result, RULE_NO_UNSAFE_TEMPLATE_BINDING);
}
