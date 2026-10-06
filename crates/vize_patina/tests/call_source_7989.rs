use serde_json::{Value, json};
use vize_patina::{LintPreset, LintResult, Linter, Severity};

const CASES: &str =
    include_str!("../../../tests/_fixtures/differential/linter/call-source-7989/cases.json");

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let labels: Vec<_> = diagnostic
                .labels
                .iter()
                .map(|label| {
                    json!({"message": label.message.as_str(), "start": label.start, "end": label.end})
                })
                .collect();
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

fn configured(rules: &[&str]) -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(rules.iter().map(|rule| (*rule).into()).collect()))
        .with_rule_severity_overrides(
            rules
                .iter()
                .map(|rule| ((*rule).into(), Severity::Warning))
                .collect(),
        )
}

#[test]
fn original_and_controls_keep_complete_configured_service_results() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let script_rules = ["script/prefer-ref-over-reactive", "script/prefer-use-id"];
    let rules = [
        script_rules[0],
        script_rules[1],
        "css/no-v-bind-performance",
        "css/no-important",
    ];
    let linter = configured(&rules);
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 41);
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        let actual = if filename.ends_with(".vue") {
            linter.lint_sfc(source, filename)
        } else {
            linter.lint_script(source, filename)
        };
        let actual = complete(&actual);
        eprintln!("{}", json!({"case": case["id"], "complete": actual}));
        assert_eq!(actual, case["service"], "{}", case["id"]);
    }
    // The entire original remains intact. Its script slice is clean; the
    // four-rule vector above retains all three unresolved CSS findings.
    let original = &cases[0];
    let actual = configured(&script_rules).lint_sfc(
        original["source"].as_str().unwrap(),
        original["filename"].as_str().unwrap(),
    );
    assert_eq!(complete(&actual), corpus["originalScriptOnly"]);
}
