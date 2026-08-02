use super::{LintPreset, Linter};

const PREFER_NUXT_LINK_RULE: &str = "ecosystem/nuxt-prefer-nuxt-link";

#[test]
fn nuxt_preset_reports_internal_anchor() {
    let result = Linter::with_preset(LintPreset::Nuxt).lint_sfc(
        r#"<template><a href="/">Home</a></template>"#,
        "app/components/AppFooter.vue",
    );

    let diagnostics = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == PREFER_NUXT_LINK_RULE)
        .collect::<Vec<_>>();
    assert_eq!(diagnostics.len(), 1, "{:#?}", result.diagnostics);
}

#[test]
fn non_nuxt_presets_keep_prefer_nuxt_link_disabled() {
    for preset in [LintPreset::Ecosystem, LintPreset::Opinionated] {
        let result = Linter::with_preset(preset).lint_sfc(
            r#"<template><a href="/">Home</a></template>"#,
            "app/components/AppFooter.vue",
        );
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != PREFER_NUXT_LINK_RULE),
            "{preset:?} unexpectedly enabled the Nuxt-only rule: {:#?}",
            result.diagnostics
        );
    }
}

#[test]
fn prefer_nuxt_link_can_be_enabled_explicitly() {
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_additional_rules(vec![PREFER_NUXT_LINK_RULE.into()])
        .lint_template(r#"<a href="/settings">Settings</a>"#, "Navigation.vue");

    assert_eq!(result.warning_count, 1, "{:#?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].rule_name, PREFER_NUXT_LINK_RULE);
}

#[test]
fn all_rules_bundle_keeps_prefer_nuxt_link_enabled() {
    let result = Linter::with_registry(crate::RuleRegistry::with_all())
        .lint_template(r#"<a href="/settings">Settings</a>"#, "Navigation.vue");

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == PREFER_NUXT_LINK_RULE),
        "{:#?}",
        result.diagnostics
    );
}
