//! Complete scoped original-source packets and independent full-route controls.

use serde_json::{Value, json};
use vize_patina::rules::vue::{SfcElementOrderGroup, SfcElementOrderOptions};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Severity};

fn whole(result: &LintResult) -> Value {
    json!({
        "filename": result.filename,
        "error_count": result.error_count,
        "warning_count": result.warning_count,
        "diagnostics": result.diagnostics.iter().map(|d| json!({
            "rule_name": d.rule_name, "severity": d.severity, "message": d.message,
            "start": d.start, "end": d.end, "help": d.help, "fix": d.fix,
            "labels": d.labels.iter().map(|label| json!({
                "message": label.message, "start": label.start, "end": label.end
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

fn corpus() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/runtime-prop-defaults-8564/cases.json"
    ))
    .unwrap()
}

#[test]
fn runtime_props_reach_whole_scoped_original_and_ownership_packets() {
    let corpus = corpus();
    let cases = corpus["cases"].as_array().unwrap();
    let rule = corpus["rule"].as_str().unwrap();
    assert_eq!(cases.len(), 18);
    assert_eq!(
        cases
            .iter()
            .filter(|case| case["original"].is_object())
            .count(),
        7
    );
    assert_eq!(
        cases
            .iter()
            .map(|case| case["diagnostics"].as_array().unwrap().len())
            .sum::<usize>(),
        19
    );
    for case in cases {
        let result = Linter::with_preset(LintPreset::Incremental)
            .with_additional_rules(vec![rule.into()])
            .with_rule_severity_overrides(vec![(rule.into(), Severity::Error)])
            .with_help_level(HelpLevel::Full)
            .lint_sfc(
                case["source"].as_str().unwrap(),
                case["filename"].as_str().unwrap(),
            );
        assert_eq!(whole(&result), case["apiAfterPacket"], "{}", case["id"]);
    }
}

#[test]
fn supplemental_runtime_binding_and_syntax_controls_keep_complete_packets() {
    let corpus = corpus();
    let supplement: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/runtime-prop-defaults-8564/ownership-supplement.json"
    ))
    .unwrap();
    let cases = supplement["cases"].as_array().unwrap();
    let rule = corpus["rule"].as_str().unwrap();
    assert_eq!(cases.len(), 12);
    assert_eq!(
        cases
            .iter()
            .map(|case| case["diagnostics"].as_array().unwrap().len())
            .sum::<usize>(),
        1
    );
    for case in cases {
        let filename = case["filename"].as_str().unwrap();
        let diagnostics = case["diagnostics"].as_array().unwrap();
        let result = Linter::with_preset(LintPreset::Incremental)
            .with_additional_rules(vec![rule.into()])
            .with_rule_severity_overrides(vec![(rule.into(), Severity::Error)])
            .with_help_level(HelpLevel::Full)
            .lint_sfc(case["source"].as_str().unwrap(), filename);
        assert_eq!(
            whole(&result),
            json!({
                "filename": filename,
                "error_count": diagnostics.iter().filter(|d| d["severity"] == "error").count(),
                "warning_count": diagnostics.iter().filter(|d| d["severity"] == "warning").count(),
                "diagnostics": diagnostics
            }),
            "{}",
            case["id"]
        );
    }
}

#[test]
fn minimal_runtime_props_preserve_whole_preset_config_and_order_packets() {
    let corpus = corpus();
    let controls: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/runtime-prop-defaults-8564/default-route-controls.json"
    ))
    .unwrap();
    let projection: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/n8n-cli-adoption.json"
    ))
    .unwrap();
    let rule = corpus["rule"].as_str().unwrap();
    let controls = controls["cases"].as_array().unwrap();
    let projected_rules = projection["linter"]["rules"].as_object().unwrap();
    assert_eq!(controls.len(), 7);
    assert_eq!(projected_rules.len(), 51);
    assert!(!projected_rules.contains_key(rule));
    for control in controls {
        let case = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == control["id"])
            .unwrap();
        assert_eq!(case["sourceSha256"], control["sourceSha256"]);
        let projected_order = SfcElementOrderOptions {
            order: projection["linter"]["ruleOptions"]["vue/sfc-element-order"]["order"]
                .as_array()
                .unwrap()
                .iter()
                .map(|selector| SfcElementOrderGroup::new(vec![selector.as_str().unwrap().into()]))
                .collect(),
        };
        let routes = [
            ("default", Linter::new()),
            ("recommended", Linter::with_preset(LintPreset::HappyPath)),
            ("essential", Linter::with_preset(LintPreset::Essential)),
            ("ecosystem", Linter::with_preset(LintPreset::Ecosystem)),
            ("incremental", Linter::with_preset(LintPreset::Incremental)),
            (
                "explicit-off",
                Linter::new().with_disabled_rules(vec![rule.into()]),
            ),
            (
                "n8n-projection",
                Linter::with_preset(LintPreset::Incremental)
                    .with_additional_rules(
                        projected_rules
                            .keys()
                            .map(|name| name.as_str().into())
                            .collect(),
                    )
                    .with_rule_severity_overrides(
                        projected_rules
                            .keys()
                            .map(|name| (name.as_str().into(), Severity::Error))
                            .collect(),
                    )
                    .with_sfc_element_order_options(projected_order),
            ),
        ];
        assert_eq!(control["routes"].as_array().unwrap().len(), routes.len());
        for (name, linter) in routes {
            let expected = control["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["id"] == name)
                .unwrap();
            let result = linter.with_help_level(HelpLevel::Full).lint_sfc(
                case["source"].as_str().unwrap(),
                case["filename"].as_str().unwrap(),
            );
            assert_eq!(
                whole(&result),
                expected["apiPacket"],
                "{} / {name}",
                case["id"]
            );
        }
    }
}
