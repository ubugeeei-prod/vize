//! Exact public native answers for template lexical scopes (#8849).
//! Complete original source and pinned foreign packets accompany every case;
//! script/component exposure remains deliberately outside this repair.

use serde_json::{Value, json};
use vize_patina::{LintResult, Linter};

fn complete(result: &LintResult) -> Value {
    json!({
        "filename": result.filename.as_str(),
        "error_count": result.error_count,
        "warning_count": result.warning_count,
        "diagnostics": result.diagnostics.iter().map(|d| json!({
            "rule_name": d.rule_name, "severity": d.severity,
            "message": d.message.as_str(), "start": d.start, "end": d.end,
            "help": d.help.as_ref().map(|help| help.as_str()),
            "labels": d.labels.iter().map(|label| json!({
                "message": label.message.as_str(), "start": label.start, "end": label.end
            })).collect::<Vec<_>>(),
            "fix": d.fix
        })).collect::<Vec<_>>()
    })
}

fn linter() -> Linter {
    Linter::new().with_enabled_rules(Some(vec!["vue/no-template-shadow".into()]))
}

#[test]
fn template_scope_positive_and_negative_controls_keep_complete_native_results() {
    let contract: Value = serde_json::from_str(include_str!(
        "fixtures/template-shadow-scopes/controls.json"
    ))
    .unwrap();
    let cases = contract["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 51);
    let linter = linter();
    for case in cases {
        let source = case["source"].as_str().unwrap();
        assert_eq!(
            complete(&linter.lint_sfc(source, "Probe.vue")),
            case["expectedNative"],
            "{}",
            case["id"]
        );
    }
}

#[test]
fn original_n8n_template_bindings_keep_identifier_warnings_and_sibling_boundaries() {
    let contract: Value = serde_json::from_str(include_str!(
        "fixtures/template-shadow-scopes/original-sources.json"
    ))
    .unwrap();
    let sources = [
        include_str!("fixtures/template-shadow-scopes/MessagesList.vue.txt"),
        include_str!("fixtures/template-shadow-scopes/ChatMessageQueue.vue.txt"),
    ];
    let linter = linter();
    for (case, source) in contract.as_array().unwrap().iter().zip(sources) {
        assert_eq!(
            complete(&linter.lint_sfc(source, "Probe.vue")),
            case["expectedNative"],
            "{}",
            case["file"]
        );
    }
}
