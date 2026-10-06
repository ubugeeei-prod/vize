use serde_json::{Value, json};
use vize_patina::{LintPreset, LintResult, Linter, Severity};

const CASES: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/plain-script-ownership-7934/cases.json"
);

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
fn original_and_boundary_corpus_keep_whole_configured_service_results() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let rules = [
        "script/no-export-in-script-setup",
        "script/require-typed-ref",
    ];
    let linter = Linter::with_preset(LintPreset::HappyPath)
        .with_additional_rules(rules.iter().map(|rule| (*rule).into()).collect())
        .with_rule_severity_overrides(
            rules
                .iter()
                .map(|rule| ((*rule).into(), Severity::Warning))
                .collect(),
        );
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 33);
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        let actual = if filename.ends_with(".vue") {
            linter.lint_sfc(source, filename)
        } else {
            linter.lint_script(source, filename)
        };
        assert_eq!(complete(&actual), case["service"], "{}", case["id"]);
    }
}
