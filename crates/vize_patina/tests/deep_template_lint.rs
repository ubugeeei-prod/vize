//! Full legacy API corpus for ordinary-stack traversal at the real parser bound.

use serde::Deserialize;
use serde_json::{Value, json};
use vize_patina::{
    HelpLevel, LintResult, Linter, Locale, RuleRegistry, rules::a11y::AriaUnsupportedElements,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    source: String,
    expected: Value,
}

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> =
        result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                let labels: Vec<_> = diagnostic.labels.iter().map(|label| json!({
            "message": label.message.as_str(), "start": label.start, "end": label.end,
        })).collect();
                json!({"rule_name": diagnostic.rule_name, "severity": diagnostic.severity,
            "message": diagnostic.message.as_str(), "start": diagnostic.start,
            "end": diagnostic.end, "help": diagnostic.help.as_ref().map(|help| help.as_str()),
            "labels": labels, "fix": diagnostic.fix})
            })
            .collect();
    json!({"filename": result.filename.as_str(), "error_count": result.error_count,
        "warning_count": result.warning_count, "diagnostics": diagnostics})
}

#[test]
fn original_depth_corpus_keeps_complete_results_on_the_ordinary_test_stack() {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/deep-template-lint/cases.json")).unwrap();
    assert_eq!(cases.len(), 3);
    for case in cases {
        for (name, locale) in [("En", Locale::En), ("Ja", Locale::Ja), ("Zh", Locale::Zh)] {
            let mut registry = RuleRegistry::new();
            registry.register(Box::new(AriaUnsupportedElements));
            let linter = Linter::with_registry(registry)
                .with_locale(locale)
                .with_help_level(HelpLevel::Full);
            let actual = linter.lint_sfc(&case.source, "deep-template.vue");
            assert_eq!(
                complete(&actual),
                case.expected[name],
                "{}: {name}",
                case.id
            );
            assert_eq!(
                complete(&linter.lint_sfc(&case.source, "deep-template.vue")),
                complete(&actual)
            );
        }
    }
}
