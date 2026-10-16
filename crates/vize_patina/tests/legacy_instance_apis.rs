//! Exact #9217 property spans, with explicit bounded provider blind spots.
use serde::Deserialize;
use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintPreset, Linter};

#[derive(Deserialize)]
struct Corpus {
    records: Vec<Case>,
}
#[derive(Deserialize)]
struct NativeCorpus {
    records: Vec<NativeCase>,
}
#[derive(Deserialize)]
struct NativeCase {
    id: String,
    messages: Vec<Value>,
}
#[derive(Deserialize)]
struct Case {
    id: String,
    source: String,
}

fn diagnostic_key(message: &Value) -> (u64, u64, String) {
    (
        message["start"].as_u64().unwrap(),
        message["end"].as_u64().unwrap(),
        message["ruleId"].as_str().unwrap().to_string(),
    )
}

#[test]
fn authored_controls_preserve_script_warnings_and_exact_template_property_spans() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/independent.json"
    ))
    .unwrap();
    assert_eq!(corpus.records.len(), 70);
    let native: NativeCorpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/native.json"
    ))
    .unwrap();
    assert_eq!(native.records.len(), corpus.records.len());
    for case in corpus.records {
        let mut expected: Vec<_> = native
            .records
            .iter()
            .find(|record| record.id == case.id)
            .unwrap()
            .messages
            .clone();
        expected.sort_by_key(diagnostic_key);
        for preset in [LintPreset::Essential, LintPreset::HappyPath] {
            let linter = Linter::with_preset(preset)
                .with_enabled_rules(Some(vec![
                    "script/no-deprecated-dollar-listeners-api".into(),
                    "script/no-deprecated-events-api".into(),
                ]))
                .with_help_level(HelpLevel::Full);
            let result = linter.lint_sfc(&case.source, "Component.vue");
            let mut actual: Vec<_> = result
                .diagnostics
                .into_iter()
                .map(|diagnostic| {
                    json!({
                        "ruleId": diagnostic.rule_name,
                        "severity": diagnostic.severity,
                        "message": diagnostic.message.as_str(),
                        "start": diagnostic.start,
                        "end": diagnostic.end,
                        "help": diagnostic.help.as_deref(),
                        "labels": diagnostic.labels.iter().map(|label| json!({
                            "message": label.message.as_str(), "start": label.start, "end": label.end,
                        })).collect::<Vec<_>>(),
                        "fix": diagnostic.fix,
                    })
                })
                .collect();
            actual.sort_by_key(diagnostic_key);
            assert_eq!(actual, expected, "{} ({preset:?})", case.id);
            let disabled = Linter::with_preset(preset)
                .with_disabled_rules(vec![
                    "script/no-deprecated-dollar-listeners-api".into(),
                    "script/no-deprecated-events-api".into(),
                ])
                .lint_sfc(&case.source, "Component.vue");
            assert!(
                !disabled
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.rule_name
                        == "script/no-deprecated-dollar-listeners-api"
                        || diagnostic.rule_name == "script/no-deprecated-events-api"),
                "{}",
                case.id
            );
        }
    }
}
