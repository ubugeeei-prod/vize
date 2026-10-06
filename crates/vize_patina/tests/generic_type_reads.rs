use serde_json::{Value, json};
use vize_patina::{LintResult, Linter, RuleRegistry, rules::facts::NoUnusedSetupBindings};

const CASES: &str =
    include_str!("../../../tests/_fixtures/differential/linter/generic-type-reads-7938/cases.json");

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
fn full_original_and_generic_reference_matrix_keep_complete_diagnostics() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoUnusedSetupBindings));
    let linter = Linter::with_registry(registry);
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 19);
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let actual = linter.lint_sfc(source, "src/kind-picker.vue");
        assert_eq!(complete(&actual), case["service"], "{}", case["id"]);
    }
}
