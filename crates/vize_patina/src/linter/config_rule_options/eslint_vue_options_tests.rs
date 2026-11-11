use super::super::config::Linter;
use super::ConfigLintRuleOptions;
use crate::LintPreset;

#[test]
fn eslint_vue_compatibility_options_reach_selected_rules() {
    let source = r#"<script>
export default { z: 1, a: 2 }
</script>
<template>{{ condition ? { "quoted": value }.quoted : fallback }}</template>
<style>.card { color: red }</style>"#;
    let options: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/enforce-style-attribute": { "allow": ["plain", "scoped"] },
        "vue/no-restricted-syntax": { "selectors": ["ConditionalExpression"] },
        "vue/quote-props": "as-needed",
        "vue/sort-keys": {
            "order": "desc",
            "caseSensitive": true,
            "natural": false
        }
    }))
    .unwrap();
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![
            "vue/enforce-style-attribute".into(),
            "vue/no-restricted-syntax".into(),
            "vue/quote-props".into(),
            "vue/sort-keys".into(),
        ]))
        .with_config_rule_options(&options)
        .lint_sfc(source, "App.vue");
    let rules = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name)
        .collect::<Vec<_>>();
    assert_eq!(
        rules,
        ["vue/no-restricted-syntax", "vue/quote-props"],
        "{:#?}",
        result.diagnostics
    );
}
