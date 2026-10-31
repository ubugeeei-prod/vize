use crate::{HelpLevel, LintPreset, LintResult, Linter};
use serde_json::{Value, json};
use vize_l0::config::ConfigLintRuleOptions;

const RULE: &str = "vue/v-on-event-hyphenation";
const SOURCE: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/linter/v-on-hyphenation-custom-tags-10822/ColorPalette.vue.txt"
);

fn linter() -> Linter {
    Linter::with_preset(LintPreset::Opinionated)
        .with_type_aware_lint(false)
        .with_help_level(HelpLevel::Full)
        .with_enabled_rules(Some(vec![RULE.into()]))
}

fn configure(linter: Linter, options: Value) -> Linter {
    let options: ConfigLintRuleOptions =
        serde_json::from_value(json!({ "vue/v-on-event-hyphenation": options })).unwrap();
    linter.with_config_rule_options(&options)
}

fn packet(result: &LintResult) -> Value {
    json!({
        "filename": result.filename.as_str(),
        "errors": result.error_count,
        "warnings": result.warning_count,
        "diagnostics": result.diagnostics.iter().map(|finding| json!({
            "rule": finding.rule_name,
            "severity": finding.severity,
            "message": finding.message.as_str(),
            "start": finding.start, "end": finding.end,
            "help": finding.help.as_ref().map(|help| help.as_str()),
            "labels": finding.labels.iter().map(|label| json!({
                "message": label.message.as_str(), "start": label.start, "end": label.end,
            })).collect::<Vec<_>>(),
            "fix": finding.fix,
        })).collect::<Vec<_>>(),
    })
}

#[test]
fn ignore_suppresses_only_the_original_matching_listener_and_preserves_the_other_whole_finding() {
    let original = linter().lint_sfc(SOURCE, "ColorPalette.vue");
    assert_eq!(original.error_count, 0);
    assert_eq!(original.warning_count, 2);
    let positions = original
        .diagnostics
        .iter()
        .map(|finding| (finding.start, finding.end))
        .collect::<Vec<_>>();
    assert_eq!(positions, [(135, 172), (279, 311)]);
    let configured =
        configure(linter(), json!({ "ignore": ["update:"] })).lint_sfc(SOURCE, "ColorPalette.vue");
    let mut expected = packet(&original);
    expected["warnings"] = json!(1);
    expected["diagnostics"] = json!(
        expected["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .skip(1)
            .cloned()
            .collect::<Vec<_>>()
    );
    assert_eq!(packet(&configured), expected);
}

#[test]
fn matching_is_case_sensitive_literal_substring_and_an_empty_list_restores_the_whole_original_packet()
 {
    let original = packet(&linter().lint_sfc(SOURCE, "ColorPalette.vue"));
    for ignore in [
        json!([]),
        json!(["Update:"]),
        json!(["update:*"]),
        json!(["unrelated"]),
    ] {
        let result =
            configure(linter(), json!({ "ignore": ignore })).lint_sfc(SOURCE, "ColorPalette.vue");
        assert_eq!(packet(&result), original);
    }
    for ignore in [json!(["date:model"]), json!(["update:", "row"])] {
        let result =
            configure(linter(), json!({ "ignore": ignore })).lint_sfc(SOURCE, "ColorPalette.vue");
        assert_eq!(
            result.warning_count,
            if ignore.as_array().unwrap().len() == 1 {
                1
            } else {
                0
            }
        );
    }
}

#[test]
fn ignore_applies_before_never_style_and_canonical_empty_object_resets_direct_policy() {
    let source = r#"<template><Widget @update:model-value="a" @row-click="b" /></template>"#;
    let result = configure(linter(), json!({ "style": "never", "ignore": ["update:"] }))
        .lint_sfc(source, "Never.vue");
    assert_eq!(result.warning_count, 1);
    assert_eq!(
        result.diagnostics.first().unwrap().message,
        "Custom event listeners on components should not be hyphenated: 'row-click'"
    );
    let direct = linter().with_v_on_event_hyphenation_options(
        super::VOnEventHyphenationStyle::Never,
        vec!["row".into()],
    );
    assert_eq!(
        packet(&configure(direct, json!({})).lint_sfc(SOURCE, "ColorPalette.vue")),
        packet(&linter().lint_sfc(SOURCE, "ColorPalette.vue"))
    );
}

#[test]
fn configuring_an_unselected_rule_does_not_enable_it() {
    let result = configure(
        linter().with_enabled_rules(Some(vec![])),
        json!({ "ignore": ["unrelated"] }),
    )
    .lint_sfc(SOURCE, "ColorPalette.vue");
    assert!(result.diagnostics.is_empty());
}
