//! Removed instance slots use the existing canonical template ownership packet.

use serde_json::{Value, json};
use vize_patina::{LintPreset, LintResult, Linter};

const RULE: &str = "script/no-deprecated-dollar-scopedslots-api";

fn whole(result: &LintResult) -> Value {
    json!({
        "filename": result.filename,
        "error_count": result.error_count,
        "warning_count": result.warning_count,
        "diagnostics": result.diagnostics.iter().map(|d| json!({
            "rule_name": d.rule_name, "severity": d.severity, "message": d.message,
            "start": d.start, "end": d.end, "help": d.help, "fix": d.fix,
            "labels": d.labels.iter().map(|l| json!({
                "message": l.message, "start": l.start, "end": l.end
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

#[test]
fn scoped_slots_template_packets_preserve_component_ownership_and_physical_ranges() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/lint-scoped-slots-template/cases.json"
    ))
    .unwrap();
    let original: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/n8n-cli-config/script-rule-research.json"
    ))
    .unwrap();
    let original = original["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == corpus["origin"]["originalCaseId"])
        .unwrap();
    assert_eq!(corpus["cases"][0]["source"], original["source"]);
    assert_eq!(
        corpus["origin"]["originalCliExpectation"],
        original["cliExpectations"][0]
    );
    let linter =
        Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![RULE.into()]));
    for case in corpus["cases"].as_array().unwrap() {
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        let diagnostics = &case["nativeDiagnostics"];
        assert_eq!(
            whole(&linter.lint_sfc(source, filename)),
            json!({
                "filename": filename, "error_count": diagnostics.as_array().unwrap().len(),
                "warning_count": 0, "diagnostics": diagnostics
            }),
            "{}",
            case["id"]
        );
    }
    let case = &corpus["cases"][0];
    let filename = case["filename"].as_str().unwrap();
    let disabled = linter.with_disabled_rules(vec![RULE.into()]);
    assert_eq!(
        whole(&disabled.lint_sfc(case["source"].as_str().unwrap(), filename)),
        json!({"filename": filename, "error_count": 0, "warning_count": 0, "diagnostics": []})
    );
}
