//! Physical Art script conclusions over independently scoped original variants.

use serde_json::{Value, json};
use vize_patina::{LintPreset, LintResult, Linter};

const CASES: &str =
    include_str!("../../../tests/_fixtures/differential/linter/art-setup-bindings-7940/cases.json");

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
fn all_original_variants_share_script_reads_but_keep_local_scopes() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 22);
    for case in cases {
        let source = case["source"].as_str().unwrap();
        assert_eq!(source.len(), case["sourceBytes"].as_u64().unwrap() as usize);
        let rules = case["rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|rule| rule.as_str().unwrap().into())
            .collect();
        let result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(rules))
            .lint_sfc(source, case["filename"].as_str().unwrap());
        assert_eq!(complete(&result), case["service"], "{}", case["id"]);
    }
}
