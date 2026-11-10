use crate::{LintPreset, Linter};
use vize_l0::config::ConfigLintRuleOptions;

#[test]
fn config_reaches_the_selected_rule() {
    let source = r#"<template><div v-for="item in items" v-if="item.active">{{ item }}</div><div v-for="item in items" v-if="showAll">{{ item }}</div></template>"#;
    let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/no-use-v-if-with-v-for": { "allowUsingIterationVar": true }
    }))
    .unwrap();
    let result = Linter::with_preset(LintPreset::Essential)
        .with_enabled_rules(Some(vec!["vue/no-use-v-if-with-v-for".into()]))
        .with_config_rule_options(&options)
        .lint_sfc(source, "App.vue");

    assert_eq!(result.warning_count, 1, "{:#?}", result.diagnostics);
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.rule_name, diagnostic.message.as_str()))
            .collect::<Vec<_>>(),
        vec![(
            "vue/no-use-v-if-with-v-for",
            "Avoid using v-if with v-for on the same element. The v-if condition does not have access to v-for scope variables",
        )]
    );
}
