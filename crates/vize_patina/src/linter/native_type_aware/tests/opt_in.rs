use super::super::{RULE_NO_UNSAFE_TEMPLATE_BINDING, has_active_type_aware_rules};
use crate::{LintPreset, Linter};

#[test]
fn opinionated_preset_keeps_native_type_aware_rules_disabled() {
    let linter = Linter::with_preset(LintPreset::Opinionated);
    assert!(!has_active_type_aware_rules(&linter));
}

#[test]
fn explicit_opt_in_enables_native_type_aware_rules() {
    let linter = Linter::with_preset(LintPreset::Opinionated).with_type_aware_lint(true);
    assert!(has_active_type_aware_rules(&linter));
}

#[test]
fn explicit_type_rule_enables_native_type_aware_rules() {
    let linter = Linter::with_preset(LintPreset::HappyPath)
        .with_additional_rules(vec![RULE_NO_UNSAFE_TEMPLATE_BINDING.into()]);
    assert!(has_active_type_aware_rules(&linter));
}

#[test]
fn explicit_type_rule_registration_enables_native_type_aware_rules() {
    let linter = Linter::with_preset(LintPreset::HappyPath)
        .with_rule(Box::new(crate::rules::type_aware::NoReactivityLoss::new()));
    assert!(has_active_type_aware_rules(&linter));
}

#[test]
fn type_aware_opt_in_warns_when_corsa_runtime_is_missing() {
    let missing_corsa = std::env::temp_dir().join("vize-missing-corsa-for-type-aware-lint");
    let linter = Linter::with_preset(LintPreset::Opinionated)
        .with_type_aware_lint(true)
        .with_corsa_path(Some(missing_corsa));
    let result = linter.lint_sfc(
        r#"<script setup lang="ts">
const payload: any = { title: "Untyped" };
</script>

<template>
  <p>{{ payload.title }}</p>
</template>
"#,
        "TypeAwareFixture.vue",
    );

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "type/corsa-runtime")
    );
}
