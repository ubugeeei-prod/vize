//! Whole original release-blocker inputs and configured service regressions.

use serde_json::{Value, json};
use vize_patina::{LintPreset, LintResult, Linter, Severity};

const CORPUS: &str =
    include_str!("../../../tests/_fixtures/differential/linter/no-dupe-keys-447/cases.json");
const RULE: &str = "script/no-dupe-keys";

fn linter() -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_additional_rules(vec![RULE.into()])
        .with_enabled_rules(Some(vec![RULE.into()]))
}

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let labels: Vec<_> = diagnostic.labels.iter().map(|label| {
            json!({"message": label.message.as_str(), "start": label.start, "end": label.end})
        }).collect();
            json!({
                "rule_name": diagnostic.rule_name, "severity": diagnostic.severity,
                "message": diagnostic.message.as_str(), "start": diagnostic.start,
                "end": diagnostic.end, "help": diagnostic.help.as_ref().map(|help| help.as_str()),
                "labels": labels, "fix": diagnostic.fix,
            })
        })
        .collect();
    json!({"filename": result.filename.as_str(), "error_count": result.error_count,
        "warning_count": result.warning_count, "diagnostics": diagnostics})
}

#[test]
fn whole_original_51_files_preserve_all_88_collision_owners() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 51);
    assert_eq!(corpus["originalFN"], 88);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/no-dupe-keys-447");
    let linter = linter();
    let mut diagnostics = 0;
    for case in cases {
        let source =
            std::fs::read_to_string(root.join(case["sourcePath"].as_str().unwrap())).unwrap();
        let actual = linter.lint_sfc(&source, case["filename"].as_str().unwrap());
        assert_eq!(complete(&actual), case["service"], "{}", case["id"]);
        diagnostics += actual.diagnostics.len();
    }
    assert_eq!(diagnostics, 88);
}

#[test]
fn whole_positive_negative_scope_and_ownership_controls() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let controls = corpus["controls"].as_array().unwrap();
    assert_eq!(controls.len(), 39);
    let linter = linter();
    for case in controls {
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        let actual = if case["mode"] == "plain" {
            linter.lint_script(source, filename)
        } else {
            linter.lint_sfc(source, filename)
        };
        assert_eq!(complete(&actual), case["service"], "{}", case["id"]);
    }
}

#[test]
fn configured_warning_and_disabled_rule_preserve_complete_results() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let case = &corpus["controls"][0];
    let source = case["source"].as_str().unwrap();
    let filename = case["filename"].as_str().unwrap();
    let warning = linter()
        .with_rule_severity_overrides([(RULE.into(), Severity::Warning)].into_iter().collect());
    let mut expected = case["service"].clone();
    expected["error_count"] = json!(0);
    expected["warning_count"] = json!(1);
    expected["diagnostics"][0]["severity"] = json!("warning");
    assert_eq!(complete(&warning.lint_sfc(source, filename)), expected);
    let disabled = linter().with_disabled_rules(vec![RULE.into()]);
    assert_eq!(
        complete(&disabled.lint_sfc(source, filename)),
        json!({"filename": filename, "error_count": 0, "warning_count": 0,
            "diagnostics": []})
    );
}

#[test]
fn inherited_merged_and_closer_owner_boundaries_keep_complete_service_packets() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/linter/no-dupe-keys-447/neighbor-reference-replay-whole.json"
    )).unwrap();
    let rows = corpus["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 8);
    let linter = linter();
    for case in rows {
        let actual = linter.lint_sfc(
            case["source"].as_str().unwrap(),
            case["filename"].as_str().unwrap(),
        );
        assert_eq!(
            complete(&actual),
            case["expectedNativeService"],
            "{}",
            case["id"]
        );
    }
}
