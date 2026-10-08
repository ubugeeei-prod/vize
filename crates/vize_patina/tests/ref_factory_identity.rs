use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Severity};

fn fixture(name: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/lint/ref-factory-identity-8275")
        .join(name);
    Ok(serde_json::from_str(&std::fs::read_to_string(file)?)?)
}

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> =
        result
            .diagnostics
            .iter()
            .map(|d| {
                let labels: Vec<_> = d.labels.iter().map(|label| json!({
            "message": label.message.as_str(), "start": label.start, "end": label.end,
        })).collect();
                json!({
                    "rule_name": d.rule_name, "severity": d.severity, "message": d.message.as_str(),
                    "start": d.start, "end": d.end, "help": d.help.as_ref().map(|h| h.as_str()),
                    "labels": labels, "fix": d.fix,
                })
            })
            .collect();
    json!({
        "filename": result.filename.as_str(), "error_count": result.error_count,
        "warning_count": result.warning_count, "diagnostics": diagnostics,
    })
}

#[test]
fn complete_native_ref_identity_packets_match_every_independent_control() {
    let config = fixture("config.json").expect("owned fixture must be readable JSON");
    let selected = config["linter"]["rules"].as_object().unwrap();
    assert_eq!(selected.len(), 51);
    assert!(selected.values().all(|value| value == "error"));
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(
            selected.keys().map(|name| name.as_str().into()).collect(),
        ))
        .with_rule_severity_overrides(
            selected
                .keys()
                .map(|name| (name.as_str().into(), Severity::Error))
                .collect(),
        )
        .with_help_level(HelpLevel::Full)
        .with_attribute_hyphenation(vize_patina::rules::HyphenationStyle::Always)
        .with_component_name_in_template_casing(vize_patina::rules::ComponentCasing::PascalCase)
        .with_sfc_element_order_options(vize_patina::rules::SfcElementOrderOptions {
            order: ["script", "template", "style"]
                .into_iter()
                .map(|name| vize_patina::rules::SfcElementOrderGroup::new(vec![name.into()]))
                .collect(),
        });
    let golden = fixture("source-after.json").expect("owned fixture must be readable JSON");
    let oracle = fixture("independent.json").expect("owned fixture must be readable JSON");
    let cases = golden["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 37);
    assert_eq!(oracle["cases"].as_array().unwrap().len(), cases.len());
    for (case, independent) in cases.iter().zip(oracle["cases"].as_array().unwrap()) {
        assert_eq!(case["id"], independent["id"]);
        assert_eq!(case["source"], independent["source"]);
        assert_eq!(case["observations"][0], case["observations"][1]);
        assert_eq!(
            independent["observations"][0],
            independent["observations"][1]
        );
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        for _ in 0..2 {
            let result = linter.lint_sfc(source, filename);
            assert_eq!(complete(&result), case["observations"][0], "{}", case["id"]);
            let expected = &independent["observations"][0][0];
            assert_eq!(expected["fatalErrorCount"], 0);
            assert_eq!(expected["errorCount"], result.error_count);
            assert_eq!(expected["warningCount"], result.warning_count);
            let messages = expected["messages"].as_array().unwrap();
            assert_eq!(messages.len(), result.diagnostics.len());
            for (diagnostic, message) in result.diagnostics.iter().zip(messages) {
                assert_eq!(
                    config["oracleRules"][diagnostic.rule_name],
                    message["ruleId"]
                );
                assert_eq!(message["severity"], 2);
                assert_eq!(diagnostic.severity, Severity::Error);
                assert!(source.is_char_boundary(diagnostic.start as usize));
                assert!(source.is_char_boundary(diagnostic.end as usize));
                for label in &diagnostic.labels {
                    assert_eq!((label.start, label.end), (diagnostic.start, diagnostic.end));
                }
            }
        }
    }
}

#[test]
fn unchanged_full_audit_and_source_before_evidence_remain_retained() {
    let before = fixture("source-before.json").expect("owned fixture must be readable JSON");
    assert_eq!(
        before["sourceRevision"],
        "db6f2c09fe0b11f6660c8cda40a642ce5ba9e875"
    );
    assert_eq!(before["cases"].as_array().unwrap().len(), 58);
    let audit = fixture("audit.json").expect("owned fixture must be readable JSON");
    assert_eq!(audit["initial"]["cases"].as_array().unwrap().len(), 26);
    assert_eq!(audit["boundaries"]["cases"].as_array().unwrap().len(), 16);
    assert_eq!(audit["scope"]["cases"].as_array().unwrap().len(), 16);
    for case in before["cases"].as_array().unwrap() {
        assert_eq!(case["observations"][0], case["observations"][1]);
    }
}
