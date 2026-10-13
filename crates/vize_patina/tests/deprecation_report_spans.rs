//! Complete native results for the three upstream deprecation targets.

use serde::Deserialize;
use serde_json::{Value, json};
use vize_patina::{
    HelpLevel, LintResult, Linter, Locale, RuleRegistry, Severity,
    rules::vue::{
        NoDeprecatedFunctionalTemplate, NoDeprecatedSlotScopeAttribute, NoDeprecatedVBindSync,
    },
};

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
    mode: String,
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

fn reference_point(source: &str, offset: u32) -> (usize, usize) {
    let prefix = source
        .get(..offset as usize)
        .expect("authored UTF-8 boundary");
    let (mut line, mut column) = (1, 1);
    let mut chars = prefix.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            line += 1;
            column = 1;
        } else if matches!(ch, '\n' | '\u{2028}' | '\u{2029}') {
            line += 1;
            column = 1;
        } else {
            column += ch.len_utf16();
        }
    }
    (line, column)
}

#[test]
fn deprecation_targets_keep_complete_native_packets_and_pinned_reference_spans() {
    let corpus: Corpus =
        serde_json::from_str(include_str!("fixtures/deprecation-report-spans/cases.json")).unwrap();
    assert_eq!(corpus.reference_package, "eslint-plugin-vue");
    assert_eq!(corpus.reference_version, "10.9.2");
    assert_eq!(corpus.original_matrix["budget"]["falsePositives"], 0);
    assert_eq!(corpus.original_matrix["budget"]["falseNegatives"], 0);
    assert!(!corpus.original_observations.is_null());
    assert_eq!(corpus.cases.len(), 24);

    for case in corpus.cases {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedVBindSync));
        registry.register(Box::new(NoDeprecatedSlotScopeAttribute));
        registry.register(Box::new(NoDeprecatedFunctionalTemplate));
        let linter = Linter::with_registry(registry)
            .with_locale(Locale::En)
            .with_help_level(HelpLevel::Full);
        let actual = match case.mode.as_str() {
            "sfc" => linter.lint_sfc(&case.source, &case.filename),
            "html" => linter.lint_standalone_html(&case.source, &case.filename),
            other => panic!("{}: unsupported corpus mode {other}", case.id),
        };
        assert_eq!(complete(&actual), case.expected, "{}", case.id);
        for (diagnostic, reference) in actual.diagnostics.iter().zip(&case.reference) {
            let (line, column) = reference_point(&case.source, diagnostic.start);
            let (end_line, end_column) = reference_point(&case.source, diagnostic.end);
            let severity = match diagnostic.severity {
                Severity::Error => 2,
                Severity::Warning => 1,
            };
            assert_eq!(
                json!({"ruleId": diagnostic.rule_name, "severity": severity,
                    "line": line, "column": column,
                    "endLine": end_line, "endColumn": end_column}),
                json!({"ruleId": reference["ruleId"], "severity": reference["severity"],
                    "line": reference["line"], "column": reference["column"],
                    "endLine": reference["endLine"], "endColumn": reference["endColumn"]}),
                "{}: direct pinned-reference parity",
                case.id
            );
        }
        assert_eq!(
            actual.diagnostics.len(),
            case.reference.len(),
            "{}",
            case.id
        );
    }
}
