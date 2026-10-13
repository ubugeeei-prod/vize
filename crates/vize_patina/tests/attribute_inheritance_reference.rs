//! Whole native packets for original attribute inheritance failures and scope controls.

use serde::Deserialize;
use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintResult, Linter, Locale};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Corpus {
    reference_version: String,
    reference_package: String,
    original_matrix: Value,
    original_observations: Value,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    source: String,
    filename: String,
    reference: Vec<Value>,
    expected: Value,
}

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let labels: Vec<_> = diagnostic
                .labels
                .iter()
                .map(|label| {
                    json!({"message": label.message.as_str(),
                        "start": label.start, "end": label.end})
                })
                .collect();
            json!({"rule_name": diagnostic.rule_name, "severity": diagnostic.severity,
                "message": diagnostic.message.as_str(), "start": diagnostic.start,
                "end": diagnostic.end,
                "help": diagnostic.help.as_ref().map(|help| help.as_str()),
                "labels": labels, "fix": diagnostic.fix})
        })
        .collect();
    json!({"filename": result.filename.as_str(), "error_count": result.error_count,
        "warning_count": result.warning_count, "diagnostics": diagnostics})
}

#[test]
fn attribute_inheritance_matches_whole_pinned_inputs_and_scope_controls() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "fixtures/attribute-inheritance-reference/cases.json"
    ))
    .unwrap();
    assert_eq!(corpus.reference_package, "eslint-plugin-vue");
    assert_eq!(corpus.reference_version, "10.9.2");
    assert_eq!(corpus.original_matrix["budget"]["falsePositives"], 0);
    assert_eq!(corpus.original_matrix["budget"]["falseNegatives"], 0);
    assert!(!corpus.original_observations.is_null());
    assert_eq!(corpus.cases.len(), 27);

    for case in corpus.cases {
        let linter = Linter::new()
            .with_enabled_rules(Some(vec!["script/no-duplicate-attr-inheritance".into()]))
            .with_locale(Locale::En)
            .with_help_level(HelpLevel::Full);
        let actual = linter.lint_sfc(&case.source, &case.filename);
        assert_eq!(complete(&actual), case.expected, "{}", case.id);
        assert_eq!(
            actual.diagnostics.len(),
            case.reference.len(),
            "{}",
            case.id
        );
    }
}
