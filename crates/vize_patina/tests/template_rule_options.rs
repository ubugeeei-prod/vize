//! Configured template rules against the pinned eslint-plugin-vue reference.
//!
//! Each row lints one authored SFC with one rule and one option object. The
//! reported byte ranges must equal the ranges eslint-plugin-vue reports for
//! the same source and options.
use serde_json::{Value, json};
use vize_l0::config::ConfigLintRuleOptions;
use vize_patina::{LintPreset, Linter};

const CORPUS: &str =
    include_str!("../../../tests/_fixtures/differential/linter/template-rule-options/cases.json");
const REFERENCE: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/template-rule-options/eslint-plugin-vue.original.json"
);

fn source(file: &str) -> &'static str {
    match file {
        "DuplicateAttributes.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/template-rule-options/DuplicateAttributes.vue.txt"
        ),
        "TemplateRootComments.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/template-rule-options/TemplateRootComments.vue.txt"
        ),
        "LoneTemplates.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/template-rule-options/LoneTemplates.vue.txt"
        ),
        "StaticInlineStyles.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/template-rule-options/StaticInlineStyles.vue.txt"
        ),
        "DynamicIs.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/template-rule-options/DynamicIs.vue.txt"
        ),
        "VHtml.vue.txt" => include_str!(
            "../../../tests/_fixtures/differential/linter/template-rule-options/VHtml.vue.txt"
        ),
        _ => panic!("unknown pinned template source"),
    }
}

fn ranges(findings: &Value) -> Vec<(u64, u64)> {
    findings
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            (
                finding["start"].as_u64().unwrap(),
                finding["end"].as_u64().unwrap(),
            )
        })
        .collect()
}

#[test]
fn configured_template_rules_report_the_pinned_upstream_ranges() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let reference: Value = serde_json::from_str(REFERENCE).unwrap();
    assert_eq!(reference["provider"]["eslint-plugin-vue"], "10.12.0");
    let cases = corpus["cases"].as_array().unwrap();
    let rows = reference["rows"].as_array().unwrap();
    assert_eq!(cases.len(), rows.len());
    assert_eq!(cases.len(), 21);

    for (case, row) in cases.iter().zip(rows) {
        let id = case["id"].as_str().unwrap();
        assert_eq!(row["id"], id);
        let rule = case["rule"].as_str().unwrap();
        let options: ConfigLintRuleOptions = if case["options"].is_null() {
            ConfigLintRuleOptions::default()
        } else {
            serde_json::from_value(json!({ rule: case["options"] })).unwrap()
        };
        let result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![rule.into()]))
            .with_config_rule_options(&options)
            .lint_sfc(source(case["file"].as_str().unwrap()), "Probe.vue");
        let actual = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == rule)
            .map(|diagnostic| (u64::from(diagnostic.start), u64::from(diagnostic.end)))
            .collect::<Vec<_>>();
        let mut actual = actual;
        actual.sort_unstable();
        assert_eq!(actual, ranges(&row["findings"]), "{id}");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.fix.is_none()),
            "{id}: none of these rules is fixable upstream"
        );
    }
}
