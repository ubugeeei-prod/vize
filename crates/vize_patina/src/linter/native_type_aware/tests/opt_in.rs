use super::super::{
    RULE_NO_FLOATING_PROMISES, RULE_NO_UNSAFE_TEMPLATE_BINDING, has_active_type_aware_rules,
};
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

const FLOATING: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/explicit-type-rule-preset-10717/FloatingPromise.vue.txt"
);
const REPAIRED: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/explicit-type-rule-preset-10717/VoidRepair.vue.txt"
);
const AWAITED: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/explicit-type-rule-preset-10717/AwaitedCall.vue.txt"
);

/// A named type rule runs the same under any preset (#10717).
fn explicit(preset: LintPreset, source: &str, corsa_path: Option<&str>) -> crate::LintResult {
    let linter = Linter::with_preset(preset)
        .with_type_aware_lint(true)
        .with_corsa_path(corsa_path.map(Into::into))
        .with_enabled_rules(Some(vec![RULE_NO_FLOATING_PROMISES.into()]));
    linter.lint_sfc(source, "Component.vue")
}

fn packet(result: &crate::LintResult) -> std::string::String {
    format!("{result:?}")
}

#[test]
fn explicit_enabled_type_rule_registers_outside_its_preset() {
    let linter = Linter::with_preset(LintPreset::HappyPath)
        .with_enabled_rules(Some(vec![RULE_NO_FLOATING_PROMISES.into()]));
    assert!(linter.registry.has_rule(RULE_NO_FLOATING_PROMISES));
    assert!(has_active_type_aware_rules(&linter));
}

#[test]
fn explicit_type_rule_reports_a_missing_provider_under_every_preset() {
    let missing = std::env::temp_dir().join("vize-missing-corsa-10717");
    let missing = missing.to_str().unwrap();
    let opinionated = explicit(LintPreset::Opinionated, FLOATING, Some(missing));
    assert!(
        opinionated
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "type/corsa-runtime"),
        "{opinionated:?}"
    );
    for preset in [
        LintPreset::HappyPath,
        LintPreset::Essential,
        LintPreset::Nuxt,
    ] {
        let result = explicit(preset, FLOATING, Some(missing));
        assert_eq!(packet(&result), packet(&opinionated), "{preset:?}");
    }
}

#[test]
fn explicit_type_rule_output_matches_under_every_preset() {
    if !super::corsa_available() {
        return;
    }
    for (source, findings) in [(FLOATING, 1), (REPAIRED, 0), (AWAITED, 0)] {
        let opinionated = explicit(LintPreset::Opinionated, source, None);
        assert_eq!(opinionated.diagnostics.len(), findings, "{opinionated:?}");
        assert!(
            opinionated
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name == RULE_NO_FLOATING_PROMISES)
        );
        for preset in [
            LintPreset::HappyPath,
            LintPreset::Essential,
            LintPreset::Nuxt,
        ] {
            let result = explicit(preset, source, None);
            assert_eq!(packet(&result), packet(&opinionated), "{preset:?}");
        }
    }
}
