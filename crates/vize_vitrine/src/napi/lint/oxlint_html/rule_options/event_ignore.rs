use super::{PatinaLintOptionsNapi, apply_rule_options, parse_rule_option};
use serde_json::json;
use vize_patina::{LintPreset, Linter};

const RULE: &str = "vue/v-on-event-hyphenation";

#[test]
fn oxlint_projects_literal_exemptions_and_replaces_them_with_legacy_strings() {
    let mut options = PatinaLintOptionsNapi::default();
    parse_rule_option(
        RULE,
        Some(&json!({ "style": "never", "ignore": ["update:"] })),
        &mut options,
    )
    .unwrap();
    assert_eq!(options.v_on_event_hyphenation.as_deref(), Some("never"));
    assert_eq!(
        options.v_on_event_hyphenation_ignore.as_deref(),
        Some(["update:".to_owned()].as_slice())
    );
    let linter = apply_rule_options(
        Linter::with_preset(LintPreset::Opinionated).with_enabled_rules(Some(vec![RULE.into()])),
        options,
    );
    let result = linter.lint_template(
        r#"<MyComponent @update:model-value="a" @row-click="b" />"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 1);
    assert_eq!(
        result.diagnostics.first().unwrap().message,
        "Custom event listeners on components should not be hyphenated: 'row-click'"
    );

    let mut options = PatinaLintOptionsNapi::default();
    parse_rule_option(RULE, Some(&json!({ "ignore": ["row"] })), &mut options).unwrap();
    parse_rule_option(RULE, Some(&json!("never")), &mut options).unwrap();
    assert!(options.v_on_event_hyphenation_ignore.is_none());
}

#[test]
fn malformed_oxlint_objects_do_not_partially_replace_the_previous_policy() {
    let mut options = PatinaLintOptionsNapi::default();
    parse_rule_option(
        RULE,
        Some(&json!({ "style": "never", "ignore": ["row"] })),
        &mut options,
    )
    .unwrap();
    for invalid in [
        json!({ "style": "always", "ignore": ["row", "row"] }),
        json!({ "ignore": ["\u{feff}"] }),
        json!({ "ignore": ["row:exit"] }),
        json!({ "ignore": [false] }),
        json!({ "style": null }),
        json!({ "autofix": "true" }),
        json!({ "autofix": null }),
    ] {
        assert!(parse_rule_option(RULE, Some(&invalid), &mut options).is_err());
        assert_eq!(options.v_on_event_hyphenation.as_deref(), Some("never"));
        assert_eq!(
            options.v_on_event_hyphenation_ignore.as_deref(),
            Some(["row".to_owned()].as_slice())
        );
    }
}

#[test]
fn event_options_leave_an_unselected_rule_disabled() {
    let mut options = PatinaLintOptionsNapi::default();
    parse_rule_option(
        RULE,
        Some(&json!({ "ignore": ["unrelated"] })),
        &mut options,
    )
    .unwrap();
    let result = apply_rule_options(
        Linter::with_preset(LintPreset::Opinionated).with_enabled_rules(Some(vec![])),
        options,
    )
    .lint_template(r#"<MyComponent @rowClick="b" />"#, "App.vue");
    assert!(result.diagnostics.is_empty());
}
