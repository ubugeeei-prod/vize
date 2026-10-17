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

#[derive(Deserialize)]
struct BoundaryCorpus {
    records: Vec<BoundaryCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BoundaryCase {
    id: String,
    source: String,
    native_messages: Vec<Value>,
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
    let overlay: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-event-object-reference.json"
    )).unwrap();
    let corrections = overlay["originalCorrections"].as_array().unwrap();
    assert_eq!(corrections.len(), 1);
    for case in corpus.records {
        let mut expected: Vec<_> = native
            .records
            .iter()
            .find(|record| record.id == case.id)
            .unwrap()
            .messages
            .clone();
        if let Some(correction) = corrections.iter().find(|row| row["id"] == case.id) {
            assert!(
                expected.is_empty(),
                "historical provider blind spot remains frozen"
            );
            expected = correction["nativeMessages"].as_array().unwrap().clone();
        }
        assert_case(&case.id, &case.source, &mut expected);
    }
}

#[test]
fn template_authority_boundaries_preserve_whole_diagnostics_and_explicit_refusals() {
    let corpus: BoundaryCorpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-boundaries.json"
    ))
    .unwrap();
    assert_eq!(corpus.records.len(), 42);
    let overlay: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-authority-review-corrections.json"
    )).unwrap();
    assert_eq!(overlay["legacyCorrections"].as_array().unwrap().len(), 6);
    for mut case in corpus.records {
        if let Some(correction) = overlay["legacyCorrections"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == case.id)
        {
            assert_eq!(correction["source"], case.source);
            case.native_messages = correction["nativeMessages"].as_array().unwrap().clone();
        }
        assert_case(&case.id, &case.source, &mut case.native_messages);
    }
    let guards = overlay["guardRecords"].as_array().unwrap();
    assert_eq!(guards.len(), 4);
    for case in guards {
        let mut expected = case["nativeMessages"].as_array().unwrap().clone();
        assert_case(
            case["id"].as_str().unwrap(),
            case["source"].as_str().unwrap(),
            &mut expected,
        );
    }
}

#[test]
fn positive_component_bindings_exempt_only_template_values_and_keep_script_warnings() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-component-bindings.json"
    ))
    .unwrap();
    let native: NativeCorpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-component-bindings-native.json"
    ))
    .unwrap();
    assert_eq!(corpus.records.len(), 30);
    assert_eq!(
        native.records.len(),
        15,
        "scopedSlots remains explicit research"
    );
    let overlay: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-normal-setup-refusals.json"
    )).unwrap();
    let corrections = overlay["corrections"].as_array().unwrap();
    assert_eq!(corrections.len(), 3);
    for mut packet in native.records {
        if let Some(correction) = corrections.iter().find(|row| row["id"] == packet.id) {
            assert_eq!(
                correction["historicalNativeMessages"].as_array().unwrap(),
                &packet.messages
            );
            packet.messages = correction["nativeMessages"].as_array().unwrap().clone();
        }
        let case = corpus
            .records
            .iter()
            .find(|case| case.id == packet.id)
            .unwrap();
        assert_case(&case.id, &case.source, &mut packet.messages);
    }
    let counterlaws = overlay["counterlaws"].as_array().unwrap();
    assert_eq!(counterlaws.len(), 4);
    for case in counterlaws {
        let mut expected = case["nativeMessages"].as_array().unwrap().clone();
        assert_case(
            case["id"].as_str().unwrap(),
            case["source"].as_str().unwrap(),
            &mut expected,
        );
    }
}

#[test]
fn setup_declaration_forms_keep_whole_diagnostics_and_template_this_observation_frontier() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-component-declaration-bindings.json"
    )).unwrap();
    let native: NativeCorpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-component-declaration-bindings-native.json"
    )).unwrap();
    assert_eq!(corpus.records.len(), 40);
    assert_eq!(
        native.records.len(),
        20,
        "scopedSlots remains explicit research"
    );
    for mut packet in native.records {
        let case = corpus
            .records
            .iter()
            .find(|case| case.id == packet.id)
            .unwrap();
        assert_case(&case.id, &case.source, &mut packet.messages);
    }
}

fn assert_case(id: &str, source: &str, expected: &mut Vec<Value>) {
    expected.sort_by_key(diagnostic_key);
    for preset in [LintPreset::Essential, LintPreset::HappyPath] {
        let linter = Linter::with_preset(preset)
            .with_enabled_rules(Some(vec![
                "script/no-deprecated-dollar-listeners-api".into(),
                "script/no-deprecated-events-api".into(),
            ]))
            .with_help_level(HelpLevel::Full);
        let result = linter.lint_sfc(source, "Component.vue");
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
        assert_eq!(actual.as_slice(), expected.as_slice(), "{id} ({preset:?})");
        let disabled = Linter::with_preset(preset)
            .with_disabled_rules(vec![
                "script/no-deprecated-dollar-listeners-api".into(),
                "script/no-deprecated-events-api".into(),
            ])
            .lint_sfc(source, "Component.vue");
        assert!(
            !disabled
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_name
                    == "script/no-deprecated-dollar-listeners-api"
                    || diagnostic.rule_name == "script/no-deprecated-events-api"),
            "{}",
            id
        );
    }
}
