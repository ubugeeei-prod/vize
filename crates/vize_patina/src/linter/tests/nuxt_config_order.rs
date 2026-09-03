use super::{LintPreset, Linter};

#[test]
fn nuxt_preset_contains_all_four_upstream_compatibility_rules() {
    let rule_names = crate::preset::builtin_script_rule_names(LintPreset::Nuxt);
    for rule_name in [
        "nuxt/prefer-import-meta",
        "nuxt/no-page-meta-runtime-values",
        "nuxt/no-nuxt-config-test-key",
        "nuxt/nuxt-config-keys-order",
    ] {
        assert!(
            rule_names.contains(&rule_name),
            "Nuxt preset is missing {rule_name}: {rule_names:?}"
        );
    }
}

#[test]
fn nuxt_preset_reports_fixable_config_order() {
    let source = "export default { ssr: true, modules: [] }";
    let result = Linter::with_preset(LintPreset::Nuxt).lint_script(source, "nuxt.config.ts");
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.rule_name == "nuxt/nuxt-config-keys-order")
        .unwrap();
    assert_eq!(
        diagnostic.message,
        "Expected config key \"ssr\" to come after \"modules\""
    );
    assert_eq!((diagnostic.start, diagnostic.end), (17, 26));
    assert_eq!(
        diagnostic.fix.as_ref().unwrap().apply(source),
        "export default { modules: [], ssr: true, }"
    );
}

#[test]
fn nuxt_preset_keeps_explicit_nuxt_two_config_order_quiet() {
    let source = "export default defineNuxtConfig({ plugins: [], buildModules: [], modules: [], vize: { compatibility: { nuxtVersion: 2 } } })";
    let result = Linter::with_preset(LintPreset::Nuxt).lint_script(source, "nuxt.config.ts");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "nuxt/nuxt-config-keys-order"),
        "{:#?}",
        result.diagnostics
    );
}

#[test]
fn nuxt_preset_fails_closed_for_nested_compatibility_duplicates() {
    for properties in [
        "vize: { compatibility: { nuxtVersion: 3 }, compatibility: { nuxtVersion: 2 } }",
        "vize: { compatibility: { nuxtVersion: 3, nuxtVersion: 2 } }",
    ] {
        let source = format!(
            "export default defineNuxtConfig({{ plugins: [], modules: [], {properties} }})"
        );
        let result = Linter::with_preset(LintPreset::Nuxt).lint_script(&source, "nuxt.config.ts");
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_name == "nuxt/nuxt-config-keys-order"),
            "{source}: {:#?}",
            result.diagnostics
        );
    }
}

#[test]
fn nuxt_preset_ignores_unrelated_default_export_configs() {
    let linter = Linter::with_preset(LintPreset::Nuxt);
    for (filename, source) in [
        (
            "vitest.config.ts",
            "export default defineConfig({ resolve: {}, test: {} })",
        ),
        (
            "vize.config.ts",
            "export default defineConfig({ compiler: {}, vite: {} })",
        ),
    ] {
        let result = linter.lint_script(source, filename);
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != "nuxt/nuxt-config-keys-order"),
            "{filename}: {:#?}",
            result.diagnostics
        );
    }
}

#[test]
fn nuxt_preset_ignores_component_option_objects() {
    let source = r#"<script lang="ts">
export default defineComponent({ name: "DataTable", model: {} })
</script>
"#;
    let result = Linter::with_preset(LintPreset::Nuxt).lint_sfc(source, "components/DataTable.vue");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "nuxt/nuxt-config-keys-order"),
        "{:#?}",
        result.diagnostics
    );
}

#[test]
fn nuxt_preset_checks_supported_nuxt_config_path_forms() {
    let linter = Linter::with_preset(LintPreset::Nuxt);
    let source = "export default defineNuxtConfig({ ssr: true, modules: [] })";
    for filename in [
        "nuxt.config.ts",
        "apps/web/nuxt.config.mjs",
        ".config/nuxt.ts",
        "apps/web/.config/nuxt.cts",
        r"apps\web\nuxt.config.js",
        r"apps\web\.config\nuxt.mts",
    ] {
        let result = linter.lint_script(source, filename);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_name == "nuxt/nuxt-config-keys-order"),
            "{filename}: {:#?}",
            result.diagnostics
        );
    }
}

#[test]
fn config_order_rule_is_nuxt_only_but_can_be_selected_explicitly() {
    let source = "export default { ssr: true, modules: [] }";
    for preset in [LintPreset::Ecosystem, LintPreset::Opinionated] {
        let result = Linter::with_preset(preset).lint_script(source, "nuxt.config.ts");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| { diagnostic.rule_name != "nuxt/nuxt-config-keys-order" })
        );
    }

    let result = Linter::with_preset(LintPreset::Incremental)
        .with_additional_rules(vec!["nuxt/nuxt-config-keys-order".into()])
        .lint_script(source, "nuxt.config.ts");
    assert_eq!(result.error_count, 1);
    assert_eq!(
        result.diagnostics[0].rule_name,
        "nuxt/nuxt-config-keys-order"
    );
}
