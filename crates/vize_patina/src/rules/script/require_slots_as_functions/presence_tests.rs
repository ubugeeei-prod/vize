use super::RequireSlotsAsFunctions;
use crate::{
    diagnostic::{LintDiagnostic, Severity},
    rules::script::{ScriptLintResult, ScriptLinter},
};
use serde_json::Value;

const CASES: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/slot-existence-negation-10822/cases.json"
);

#[test]
fn direct_negation_preserves_complete_packets_and_value_boundaries() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let rows = corpus["cases"].as_array().unwrap();
    let mut captured = Vec::new();
    for row in rows {
        let source = row["script"].as_str().unwrap();
        for offset in [0, 17] {
            let mut linter = ScriptLinter::new();
            linter.add_rule(Box::new(RequireSlotsAsFunctions));
            let result = linter.lint(source, offset);
            println!(
                "vize-slot-presence-api id={} source={source:?} offset={offset} result={result:#?}",
                row["id"].as_str().unwrap(),
            );
            captured.push((row, source, offset, result));
        }
    }
    // Every complete result is retained before checking the independent authored contract.
    for (row, source, offset, actual) in captured {
        let diagnostics = row["nativeSpans"]
            .as_array()
            .unwrap()
            .iter()
            .map(|span| {
                let needle = span["needle"].as_str().unwrap();
                assert_eq!(source.matches(needle).count(), 1, "{}", row["id"]);
                let start = offset
                    + source.find(needle).unwrap()
                    + span["shift"].as_u64().unwrap() as usize;
                LintDiagnostic {
                    rule_name: "script/require-slots-as-functions",
                    severity: Severity::Error,
                    message: "`$slots` properties are functions in Vue 3 and must be called."
                        .into(),
                    start: start as u32,
                    end: (start + span["length"].as_u64().unwrap() as usize) as u32,
                    help: None,
                    labels: Vec::new(),
                    fix: None,
                }
            })
            .collect::<Vec<_>>();
        let expected = ScriptLintResult {
            error_count: diagnostics.len(),
            warning_count: 0,
            diagnostics,
        };
        assert_eq!(
            format!("{actual:#?}"),
            format!("{expected:#?}"),
            "{} / offset {offset}",
            row["id"],
        );
    }
}
