use super::{PatinaLintOptionsNapi, apply_rule_options, parse_rule_option};
use serde_json::json;
use vize_patina::{LintPreset, Linter};

const RULE: &str = "vue/v-on-event-hyphenation";

#[test]
fn typed_autofix_reaches_the_argument_and_legacy_strings_clear_it() {
    let mut options = PatinaLintOptionsNapi::default();
    parse_rule_option(
        RULE,
        Some(&json!({ "style": "never", "autofix": true })),
        &mut options,
    )
    .unwrap();
    assert_eq!(options.v_on_event_hyphenation_autofix, Some(true));
    let linter = apply_rule_options(
        Linter::with_preset(LintPreset::Opinionated).with_enabled_rules(Some(vec![RULE.into()])),
        options,
    );
    let source = r#"<MyWidget @row-click.once="choose" />"#;
    let result = linter.lint_template(source, "App.vue");
    assert_eq!(result.diagnostics.len(), 1);
    let fix = result.diagnostics[0].fix.as_ref().unwrap();
    assert_eq!(fix.apply(source), r#"<MyWidget @rowClick.once="choose" />"#);
    let mut options = PatinaLintOptionsNapi::default();
    parse_rule_option(RULE, Some(&json!({ "autofix": true })), &mut options).unwrap();
    parse_rule_option(RULE, Some(&json!("always")), &mut options).unwrap();
    assert_eq!(options.v_on_event_hyphenation_autofix, None);
}

#[test]
fn malformed_autofix_does_not_partially_replace_the_last_complete_policy() {
    let mut options = PatinaLintOptionsNapi::default();
    parse_rule_option(
        RULE,
        Some(&json!({ "style": "never", "autofix": true, "ignore": ["row"] })),
        &mut options,
    )
    .unwrap();
    for value in [
        json!({ "style": "always", "autofix": "true" }),
        json!({ "autofix": null }),
        json!({ "autofix": 1 }),
        json!({ "autofixTypo": true }),
    ] {
        assert!(parse_rule_option(RULE, Some(&value), &mut options).is_err());
        assert_eq!(options.v_on_event_hyphenation.as_deref(), Some("never"));
        assert_eq!(options.v_on_event_hyphenation_autofix, Some(true));
        assert_eq!(
            options.v_on_event_hyphenation_ignore.as_deref(),
            Some(["row".to_owned()].as_slice())
        );
    }
}
