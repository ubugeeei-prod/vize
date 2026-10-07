use vize_patina::{LintPreset, Linter};

const RULE: &str = "vue/require-component-registration";
const SOURCE: &str = include_str!("fixtures/global-component-registration/Card.art.vue");

#[test]
fn preview_globals_allow_both_vue_spellings_without_hiding_missing_components() {
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_additional_rules(vec![RULE.into()])
        .with_enabled_rules(Some(vec![RULE.into()]));
    let before = linter.lint_sfc(SOURCE, "Card.art.vue");
    assert_eq!(before.diagnostics.len(), 3);
    let configured = linter.with_component_registration_globals(vec!["MyButton".into()]);
    let after = configured.lint_sfc(SOURCE, "Card.art.vue");
    assert_eq!(after.diagnostics.len(), 1);
    assert_eq!(after.diagnostics[0].rule_name, RULE);
    assert_eq!(
        after.diagnostics[0].start as usize,
        SOURCE.find("MissingWidget").unwrap()
    );
}

#[test]
fn global_options_do_not_enable_an_unselected_rule() {
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_component_registration_globals(vec!["MyButton".into()])
        .lint_sfc(SOURCE, "Card.art.vue");
    assert!(result.diagnostics.iter().all(|item| item.rule_name != RULE));
}
