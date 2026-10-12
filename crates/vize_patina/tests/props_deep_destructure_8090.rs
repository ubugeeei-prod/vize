//! Independently authored whole packets for setup-owned prop destructuring.

use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Severity};

const RULE: &str = "script/no-deep-destructure-in-props";

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

fn lint(linter: &Linter, case: &Value) -> LintResult {
    let source = case["source"].as_str().unwrap();
    let filename = case["filename"].as_str().unwrap();
    if filename.ends_with(".vue") {
        linter.lint_sfc(source, filename)
    } else {
        linter.lint_script(source, filename)
    }
}

#[test]
fn sole_inline_top_level_macros_preserve_all_thirty_three_whole_packets() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/props-deep-destructure-8090/cases.json"
    ))
    .unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 33);
    assert_eq!(
        cases
            .iter()
            .map(|case| case["afterApi"]["warning_count"].as_u64().unwrap())
            .sum::<u64>(),
        10
    );
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_additional_rules(vec![RULE.into()])
        .with_rule_severity_overrides(vec![(RULE.into(), Severity::Warning)])
        .with_help_level(HelpLevel::Full);
    for case in cases {
        assert_eq!(
            whole(&lint(&linter, case)),
            case["afterApi"],
            "{}",
            case["id"]
        );
    }
}

#[test]
fn explicit_off_preserves_empty_whole_packets_for_real_and_foreign_macros() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/props-deep-destructure-8090/cases.json"
    ))
    .unwrap();
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_additional_rules(vec![RULE.into()])
        .with_disabled_rules(vec![RULE.into()])
        .with_help_level(HelpLevel::Full);
    for id in [
        "vue35-nested-object",
        "vue35-nested-array",
        "standalone-local-function",
    ] {
        let case = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == id)
            .unwrap();
        assert_eq!(
            whole(&lint(&linter, case)),
            json!({"filename": case["filename"], "error_count": 0,
                "warning_count": 0, "diagnostics": []}),
            "{id}"
        );
    }
}
