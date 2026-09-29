use super::Linter;
use crate::{LintPreset, Severity};

const OUTSIDE_PRESET: &str = r##"<template>
  <ul>
    <li v-for="(item, index) in items" :key="index">{{ item }}</li>
  </ul>
  <MissingItem />
  <a href="#">Top</a>
</template>
"##;

#[test]
fn configured_rules_outside_preset_report_on_essential() {
    let linter = Linter::with_preset(LintPreset::Essential)
        .with_additional_rules(vec![
            "vue/no-array-index-key".into(),
            "a11y/anchor-is-valid".into(),
        ])
        .with_rule_severity_overrides(vec![
            ("vue/no-array-index-key".into(), Severity::Error),
            ("a11y/anchor-is-valid".into(), Severity::Error),
        ]);

    assert!(linter.registry().has_rule("vue/no-array-index-key"));
    assert!(linter.registry().has_rule("a11y/anchor-is-valid"));
    assert!(
        !linter
            .registry()
            .has_rule("vue/require-component-registration")
    );
    assert!(linter.registry().has_rule("vue/require-v-for-key"));

    let result = linter.lint_sfc(OUTSIDE_PRESET, "TodoList.vue");
    let index_key = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.rule_name == "vue/no-array-index-key")
        .expect("explicit vue/no-array-index-key should report");
    assert_eq!(index_key.severity, Severity::Error);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "a11y/anchor-is-valid"
                && diagnostic.severity == Severity::Error)
    );
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "vue/require-component-registration")
    );
}

#[test]
fn configured_rules_outside_preset_off_still_disables() {
    let source = r##"<template>
  <ul>
    <li v-for="(item, index) in items" :key="index">{{ item }}</li>
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>
"##;
    let linter = Linter::with_preset(LintPreset::Essential)
        .with_additional_rules(vec!["vue/no-array-index-key".into()])
        .with_disabled_rules(vec!["vue/require-v-for-key".into()])
        .with_rule_severity_overrides(vec![("vue/no-array-index-key".into(), Severity::Error)]);
    let result = linter.lint_sfc(source, "TodoList.vue");

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "vue/no-array-index-key")
    );
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "vue/require-v-for-key")
    );
}
