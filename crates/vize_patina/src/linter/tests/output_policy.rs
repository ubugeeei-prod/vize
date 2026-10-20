use super::Linter;
use crate::{HelpLevel, LintPreset};

#[test]
fn help_level_none_covers_script_css_and_sfc_rule_lanes() {
    let result = Linter::with_preset(LintPreset::Opinionated)
        .with_enabled_rules(Some(vec![
            "script/prefer-ref-over-reactive".into(),
            "css/no-hardcoded-values".into(),
            "vue/require-scoped-style".into(),
        ]))
        .with_help_level(HelpLevel::None)
        .lint_sfc(
            r#"<script setup>const state = reactive({ count: 0 })</script>
<template><div>{{ state.count }}</div></template>
<style>.panel { color: #ff0000 }</style>"#,
            "StatePanel.vue",
        );

    let rules: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name)
        .collect();
    assert!(
        rules.contains(&"script/prefer-ref-over-reactive"),
        "{rules:?}"
    );
    assert!(rules.contains(&"css/no-hardcoded-values"), "{rules:?}");
    assert!(rules.contains(&"vue/require-scoped-style"), "{rules:?}");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.help.is_none()),
        "{:?}",
        result.diagnostics
    );
}
