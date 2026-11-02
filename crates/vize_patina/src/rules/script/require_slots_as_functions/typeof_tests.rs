use super::RequireSlotsAsFunctions;
use crate::{
    diagnostic::{LintDiagnostic, Severity},
    rules::script::{ScriptLintResult, ScriptLinter},
};
use serde_json::Value;

const CASES: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/slot-typeof-presence-10822/cases.json"
);
const SUCCESSORS: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/slot-typeof-presence-10822/semantic-successors.json"
);

/// The two immutable former outside-slice cases now exercise direct-Typeof v1.
/// Their old rows and complete before packets stay in their original corpus and
/// the named successor custody. Every other row keeps its original expectation.
pub(super) fn current_native_spans(corpus: &str, historical: &Value) -> Vec<Value> {
    if historical["id"] != "typeof-outside-slice" {
        return historical["nativeSpans"].as_array().unwrap().clone();
    }
    let expected_sha = match corpus {
        "slot-existence-negation-10822" => {
            "683a9b923ca916109703e44c3f4c67b1a71ac27abcdfc2872add504334a2dd0d"
        }
        "slot-existence-conditions-10822" => {
            "49fe9e0fe0544c6b41f047bb2c67eabcb067f7c5f8e62430085a65b038055da7"
        }
        _ => panic!("unregistered typeof predecessor corpus"),
    };
    let data: Value = serde_json::from_str(SUCCESSORS).unwrap();
    assert_eq!(data["schema"], "vize.slot-typeof-semantic-successors.v1");
    assert_eq!(data["version"], 1);
    assert_eq!(data["feature"], "immediate-authenticated-typeof-operand-v1");
    let row = data["successors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["predecessorCorpus"] == corpus)
        .unwrap();
    assert_eq!(row["predecessorCorpusSha256"], expected_sha);
    assert_eq!(&row["historicalCase"], historical);
    assert_eq!(row["currentNativeSpans"], serde_json::json!([]));
    println!("vize-slot-typeof-semantic-successor corpus={corpus} complete={row}");
    Vec::new()
}

#[test]
fn immediate_typeof_preserves_full_native_packets_and_value_boundaries() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let mut captured = Vec::new();
    for row in corpus["cases"].as_array().unwrap() {
        let source = row["script"].as_str().unwrap();
        for offset in [0, 17] {
            let mut linter = ScriptLinter::new();
            linter.add_rule(Box::new(RequireSlotsAsFunctions));
            let result = linter.lint(source, offset);
            println!(
                "vize-slot-typeof-api id={} source={source:?} offset={offset} result={result:#?}",
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
