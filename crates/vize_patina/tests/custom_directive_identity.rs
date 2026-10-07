//! #8226 preserves complete custom and historical built-in diagnostic packets.

use serde::Deserialize;
use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_l0::{String, cstr};
use vize_patina::rules::vue::NoDuplicateAttributes;
use vize_patina::{LintResult, Linter, Severity, rule::RuleRegistry};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    source: String,
    allow_class: bool,
    allow_style: bool,
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
                    json!({
                        "message": label.message, "start": label.start, "end": label.end,
                    })
                })
                .collect();
            json!({
                "rule": diagnostic.rule_name,
                "severity": match diagnostic.severity {
                    Severity::Error => "Error",
                    Severity::Warning => "Warning",
                },
                "message": diagnostic.message,
                "start": diagnostic.start,
                "end": diagnostic.end,
                "help": diagnostic.help,
                "labels": labels,
                "fix": diagnostic.fix,
            })
        })
        .collect();
    json!({
        "filename": result.filename,
        "errorCount": result.error_count,
        "warningCount": result.warning_count,
        "diagnostics": diagnostics,
    })
}

#[test]
fn complete_custom_and_historical_directive_packets_match_the_corpus() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/custom-directive-identity-8226");
    let cases: Vec<String> =
        serde_json::from_slice(&fs::read(root.join("controls.json")).unwrap()).unwrap();
    assert_eq!(cases.len(), 32);
    for id in cases {
        let case: Case = serde_json::from_slice(
            &fs::read(
                root.join("controls")
                    .join(id.as_str())
                    .with_extension("json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(case.id, id);
        for _ in 0..2 {
            let mut registry = RuleRegistry::new();
            registry.register(Box::new(NoDuplicateAttributes {
                allow_coexist_class: case.allow_class,
                allow_coexist_style: case.allow_style,
            }));
            let linter = Linter::with_registry(registry);
            let filename = cstr!("{id}.vue");
            let result = linter.lint_template_rules_only(&case.source, &filename);
            assert_eq!(complete(&result), case.expected, "{id}");
        }
    }
}
