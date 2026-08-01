use super::{LintPreset, Linter};

#[test]
fn nuxt_preset_contains_all_four_upstream_compatibility_rules() {
    assert_eq!(
        crate::preset::builtin_script_rule_names(LintPreset::Nuxt),
        &[
            "nuxt/prefer-import-meta",
            "nuxt/no-page-meta-runtime-values",
            "nuxt/no-nuxt-config-test-key",
            "nuxt/nuxt-config-keys-order",
        ]
    );
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
        "Expected config key \"modules\" to come before \"ssr\""
    );
    assert_eq!(
        diagnostic.fix.as_ref().unwrap().apply(source),
        "export default { modules: [], ssr: true, }"
    );
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
