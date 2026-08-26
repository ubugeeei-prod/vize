use super::{LintPreset, Linter};
use vize_s0::ToCompactString;

#[test]
fn nuxt_preset_reports_and_fixes_legacy_process_flags() {
    let linter = Linter::with_preset(LintPreset::Nuxt);
    let source = r#"<script setup lang="ts">
const enabled = process.client && process.prerender
</script>
"#;
    let result = linter.lint_sfc(source, "app.vue");
    let diagnostics = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "nuxt/prefer-import-meta")
        .collect::<Vec<_>>();
    assert_eq!(diagnostics.len(), 2, "{:#?}", result.diagnostics);

    let mut edits = diagnostics
        .iter()
        .flat_map(|diagnostic| diagnostic.fix.as_ref().unwrap().edits.iter())
        .collect::<Vec<_>>();
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.start));
    let mut fixed = source.to_compact_string();
    for edit in edits {
        fixed.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
    }
    assert!(fixed.contains("import.meta.client && import.meta.prerender"));
    assert!(
        linter
            .lint_sfc(&fixed, "app.vue")
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "nuxt/prefer-import-meta")
    );
}

#[test]
fn non_nuxt_presets_keep_prefer_import_meta_disabled() {
    for preset in [LintPreset::Ecosystem, LintPreset::Opinionated] {
        let result = Linter::with_preset(preset).lint_script("process.client", "runtime.ts");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != "nuxt/prefer-import-meta"),
            "{preset:?} unexpectedly enabled the Nuxt-only rule"
        );
    }
}

#[test]
fn prefer_import_meta_can_be_enabled_explicitly() {
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_additional_rules(vec!["nuxt/prefer-import-meta".into()]);
    let result = linter.lint_script("process.server", "runtime.ts");
    assert_eq!(result.error_count, 1);
    assert_eq!(result.diagnostics[0].rule_name, "nuxt/prefer-import-meta");
}

#[test]
fn nuxt_preset_reports_eager_page_meta_runtime_values() {
    let linter = Linter::with_preset(LintPreset::Nuxt);
    let source = r#"<script setup lang="ts">
definePageMeta({ title: useRoute().path, middleware: () => useRoute() })
</script>
"#;
    let result = linter.lint_sfc(source, "pages/index.vue");
    let diagnostics = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "nuxt/no-page-meta-runtime-values")
        .collect::<Vec<_>>();

    assert_eq!(diagnostics.len(), 1, "{:#?}", result.diagnostics);
    assert_eq!(
        &source[diagnostics[0].start as usize..diagnostics[0].end as usize],
        "useRoute()"
    );
    assert!(diagnostics[0].fix.is_none());
}

#[test]
fn non_nuxt_presets_keep_page_meta_runtime_rule_disabled() {
    for preset in [LintPreset::Ecosystem, LintPreset::Opinionated] {
        let result = Linter::with_preset(preset)
            .lint_script("definePageMeta({ title: useRoute() })", "pages/index.ts");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != "nuxt/no-page-meta-runtime-values"),
            "{preset:?} unexpectedly enabled the Nuxt-only rule"
        );
    }
}

#[test]
fn page_meta_runtime_rule_can_be_enabled_explicitly() {
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_additional_rules(vec!["nuxt/no-page-meta-runtime-values".into()]);
    let result = linter.lint_script("definePageMeta({ title: useRoute() })", "pages/index.ts");
    assert_eq!(result.error_count, 1);
    assert_eq!(
        result.diagnostics[0].rule_name,
        "nuxt/no-page-meta-runtime-values"
    );
}

#[test]
fn nuxt_preset_reports_boolean_test_config_key() {
    let source = "export default defineNuxtConfig({ test: true })";
    let result = Linter::with_preset(LintPreset::Nuxt).lint_script(source, "nuxt.config.ts");
    let diagnostics = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "nuxt/no-nuxt-config-test-key")
        .collect::<Vec<_>>();

    assert_eq!(diagnostics.len(), 1, "{:#?}", result.diagnostics);
    assert_eq!(
        &source[diagnostics[0].start as usize..diagnostics[0].end as usize],
        "test: true"
    );
    assert!(diagnostics[0].fix.is_none());
}

#[test]
fn nuxt_config_test_key_prefilter_keeps_escaped_identifiers() {
    let source = r"export default { t\u0065st: true }";
    let result = Linter::with_preset(LintPreset::Nuxt).lint_script(source, "nuxt.config.ts");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "nuxt/no-nuxt-config-test-key"),
        "{:#?}",
        result.diagnostics
    );
}

#[test]
fn non_nuxt_presets_keep_config_test_key_rule_disabled() {
    for preset in [LintPreset::Ecosystem, LintPreset::Opinionated] {
        let result = Linter::with_preset(preset)
            .lint_script("export default { test: true }", "nuxt.config.ts");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != "nuxt/no-nuxt-config-test-key"),
            "{preset:?} unexpectedly enabled the Nuxt-only rule"
        );
    }
}

#[test]
fn config_test_key_rule_can_be_enabled_explicitly() {
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_additional_rules(vec!["nuxt/no-nuxt-config-test-key".into()])
        .lint_script("export default { test: false }", "nuxt.config.ts");
    assert_eq!(result.error_count, 1);
    assert_eq!(
        result.diagnostics[0].rule_name,
        "nuxt/no-nuxt-config-test-key"
    );
}

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
