use super::RequireSlotsAsFunctions;
use crate::{
    diagnostic::{LintDiagnostic, Severity},
    rules::script::{ScriptLintResult, ScriptLinter},
};
use serde_json::Value;

const CASES: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/slot-conditional-arms-10822/cases.json"
);
const SUCCESSORS: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/slot-conditional-arms-10822/semantic-successors.json"
);

/// Exactly one immutable old condition row gains ConditionalArms-v1 semantics.
/// Other rows retain their old spans or the separately pinned Typeof-v1 successor.
pub(super) fn current_native_spans(historical: &Value) -> Vec<Value> {
    if historical["id"] != "alternate-value-outside-slice" {
        return super::typeof_tests::current_native_spans(
            "slot-existence-conditions-10822",
            historical,
        );
    }
    let data: Value = serde_json::from_str(SUCCESSORS).unwrap();
    assert_eq!(
        data["schema"],
        "vize.slot-conditional-arm-semantic-successors.v1"
    );
    assert_eq!(data["version"], 1);
    assert_eq!(
        data["feature"],
        "immediate-authenticated-conditional-arms-v1"
    );
    let row = &data["conditionCase"];
    assert_eq!(row["predecessorCorpus"], "slot-existence-conditions-10822");
    assert_eq!(
        row["predecessorCorpusSha256"],
        "49fe9e0fe0544c6b41f047bb2c67eabcb067f7c5f8e62430085a65b038055da7"
    );
    assert_eq!(&row["historicalCase"], historical);
    assert_eq!(row["currentNativeSpans"], serde_json::json!([]));
    println!("vize-slot-conditional-semantic-successor complete={row}");
    Vec::new()
}

#[test]
fn immediate_arms_preserve_complete_native_packets_and_nested_findings() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let mut captured = Vec::new();
    for row in corpus["cases"].as_array().unwrap() {
        let source = row["script"].as_str().unwrap();
        for offset in [0, 17] {
            let mut linter = ScriptLinter::new();
            linter.add_rule(Box::new(RequireSlotsAsFunctions));
            let result = linter.lint(source, offset);
            println!(
                "vize-slot-conditional-api id={} source={source:?} offset={offset} result={result:#?}",
                row["id"].as_str().unwrap(),
            );
            captured.push((row, source, offset, result));
        }
    }
    for (row, source, offset, actual) in captured {
        let diagnostics = row["nativeSpans"]
            .as_array()
            .unwrap()
            .iter()
            .map(|span| {
                let needle = span["needle"].as_str().unwrap();
                assert_eq!(source.matches(needle).count(), 1);
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
            "{} / {offset}",
            row["id"]
        );
    }
}
