use super::{LintPreset, Linter};

#[test]
fn test_lint_standalone_html_does_not_warn_custom_block() {
    // Regression for https://github.com/ubugeeei-prod/vize/issues/2245:
    // running `vize lint --preset nuxt` on a standalone `.html` file (e.g.
    // `.storybook/preview-head.html`) reported `vue/warn-custom-block` for
    // top-level HTML elements like `<link>`. Standalone HTML files are not
    // Vue SFCs, so the SFC custom-block rule must not fire on them.
    let linter = Linter::with_preset(LintPreset::Nuxt);
    let source = r#"<link rel="stylesheet" href="https://fonts.googleapis.com/css?family=Roboto" />
"#;

    let result = linter.lint_standalone_html(source, ".storybook/preview-head.html");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "vue/warn-custom-block"),
        "vue/warn-custom-block must not fire on standalone HTML files, got: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| d.rule_name)
            .collect::<Vec<_>>()
    );
}

#[test]
fn nuxt_preset_allows_options_api_components() {
    let linter = Linter::with_preset(LintPreset::Nuxt);
    let sfc = r#"<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'AppLoader',
  props: {
    active: Boolean
  }
})
</script>
"#;
    let result = linter.lint_sfc(sfc, "components/AppLoader.vue");

    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn nuxt_preset_allows_vapor_only_script_patterns_by_default() {
    let linter = Linter::with_preset(LintPreset::Nuxt);
    let sfc = r#"<script setup lang="ts">
import { getCurrentInstance, nextTick } from 'vue'

const instance = getCurrentInstance()
await nextTick()
</script>
"#;
    let result = linter.lint_sfc(sfc, "components/Dialog.vue");

    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/no-next-tick"),
        "Nuxt projects should not report script/no-next-tick unless the rule is enabled, got {:?}",
        result.diagnostics
    );
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/no-get-current-instance"),
        "Nuxt projects should not report script/no-get-current-instance unless the rule is enabled, got {:?}",
        result.diagnostics
    );
}

#[test]
fn nuxt_preset_allows_next_tick_in_standalone_scripts_by_default() {
    let result = Linter::with_preset(LintPreset::Nuxt).lint_script(
        r#"import { nextTick } from "@nuxtjs/composition-api";

await nextTick();
"#,
        "composables/useDialog.ts",
    );

    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/no-next-tick"),
        "Nuxt composables should not report script/no-next-tick unless the rule is enabled, got {:?}",
        result.diagnostics
    );
}

#[test]
fn nuxt_preset_allows_vuetify_kebab_components() {
    let linter = Linter::with_preset(LintPreset::Nuxt);
    let sfc = r#"<template>
  <v-dialog>
    <v-btn />
    <v-icon />
    <v-spacer />
  </v-dialog>
</template>
"#;
    let result = linter.lint_sfc(sfc, "components/Dialog.vue");

    for diagnostic in &result.diagnostics {
        assert!(
            diagnostic.rule_name != "vue/component-name-in-template-casing"
                && diagnostic.rule_name != "vue/html-self-closing",
            "Nuxt preset should not flag Vuetify v-* tags, got {diagnostic:?}",
        );
    }
}

#[test]
fn opinionated_preset_still_flags_vuetify_kebab_components() {
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let sfc = r#"<template>
  <v-btn />
</template>
"#;
    let result = linter.lint_sfc(sfc, "components/Dialog.vue");

    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.rule_name == "vue/component-name-in-template-casing"),
        "Opinionated preset should still flag Vuetify v-* tags as kebab-case, got {:?}",
        result.diagnostics
    );
}

#[test]
fn nuxt_preset_reports_next_tick_when_rule_is_enabled() {
    let linter = Linter::with_preset(LintPreset::Nuxt)
        .with_additional_rules(vec!["script/no-next-tick".into()]);
    let sfc = r#"<script setup lang="ts">
import { nextTick } from 'vue'

await nextTick()
</script>
"#;
    let result = linter.lint_sfc(sfc, "components/Dialog.vue");

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/no-next-tick"),
        "explicit script/no-next-tick should still report, got {:?}",
        result.diagnostics
    );
}
