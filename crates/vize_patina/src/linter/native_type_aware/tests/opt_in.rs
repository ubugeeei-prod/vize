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
