//! Complete public Vapor output references for five further historical fixes.
//!
//! The expected payloads come from a clean, source-bound observer build.
//! They pin raw module bytes, ordered templates, nullable maps and errors.

use serde_json::Value;

mod support;
use support::fix_history_next as fixtures;

const FIRST: &str = include_str!("fixtures/fix-history-next/capture-b11eb/first.stdout.json");
const REPEAT: &str = include_str!("fixtures/fix-history-next/capture-b11eb/repeat.stdout.json");
const CASES: &[(&str, &str)] = &[
    (
        "once-directive",
        include_str!("fixtures/fix-history-next/once-directive.expected.json"),
    ),
    (
        "insertion-placeholder",
        include_str!("fixtures/fix-history-next/insertion-placeholder.expected.json"),
    ),
    (
        "root-document-order",
        include_str!("fixtures/fix-history-next/root-document-order.expected.json"),
    ),
    (
        "hydration-sibling-element",
        include_str!("fixtures/fix-history-next/hydration-sibling-element.expected.json"),
    ),
    (
        "mounted-control-slot",
        include_str!("fixtures/fix-history-next/mounted-control-slot.expected.json"),
    ),
];

const CORRECTED_8625: &str =
    include_str!("fixtures/fix-history-next/root-document-order.corrected-8625.expected.json");

const CURRENT_CASES: &[&str] = &[
    include_str!("fixtures/fix-history-next/once-directive.expected.json"),
    include_str!("fixtures/fix-history-next/insertion-placeholder.expected.json"),
    include_str!("fixtures/fix-history-next/root-document-order.corrected-8859.expected.json"),
    include_str!("fixtures/fix-history-next/hydration-sibling-element.expected.json"),
    include_str!("fixtures/fix-history-next/mounted-control-slot.expected.json"),
];

#[test]
fn five_history_fixes_preserve_complete_vapor_outputs() {
    assert_eq!(
        FIRST.as_bytes(),
        REPEAT.as_bytes(),
        "two raw captures differ"
    );
    let first = serde_json::from_str::<Value>(FIRST).expect("valid immutable capture");
    let observed = first
        .get("cases")
        .and_then(Value::as_array)
        .expect("captured cases");
    assert_eq!(observed.len(), fixtures::CASES.len());
    assert_eq!(CASES.len(), fixtures::CASES.len());
    assert_eq!(CURRENT_CASES.len(), fixtures::CASES.len());
    for ((((id, source, prefix), (expected_id, expected)), captured), current) in fixtures::CASES
        .iter()
        .zip(CASES)
        .zip(observed)
        .zip(CURRENT_CASES)
    {
        assert_eq!(id, expected_id, "fixture order changed");
        assert_eq!(captured.get("id").and_then(Value::as_str), Some(*id));
        let expected = serde_json::from_str::<Value>(expected).expect("immutable case result");
        assert_eq!(captured.get("options"), expected.get("options"));
        assert_eq!(captured.get("output"), expected.get("output"));
        let current = serde_json::from_str::<Value>(current).expect("current complete case result");
        assert_eq!(current.get("options"), expected.get("options"));
        assert_eq!(
            fixtures::option_payload(*prefix),
            expected.get("options").expect("captured options").clone(),
            "Vapor options changed for {id}"
        );
        assert_eq!(
            fixtures::observe(source, *prefix),
            current
                .get("output")
                .expect("captured complete output")
                .clone(),
            "complete raw Vapor output changed for {id}"
        );
    }
}

#[test]
fn root_flag_correction_preserves_every_other_historical_output_byte() {
    let historical = serde_json::from_str::<Value>(CASES[2].1).expect("immutable original case");
    let corrected = serde_json::from_str::<Value>(CORRECTED_8625).expect("#8625 complete case");
    let current = serde_json::from_str::<Value>(CURRENT_CASES[2]).expect("current complete case");
    let old_code = historical["output"]["code"]
        .as_str()
        .expect("whole original module");
    let old_call = "_createComponentWithFallback(_component_Child, null, null, true)";
    assert_eq!(old_code.matches(old_call).count(), 1);
    let replaced = |call: &str| {
        let mut authorized = historical.clone();
        authorized["output"]["code"] = Value::String(old_code.replacen(old_call, call, 1));
        authorized
    };
    assert_eq!(
        corrected,
        replaced("_createComponentWithFallback(_component_Child, null, null, false)"),
        "only the explicit #8625 sibling-root flag changes"
    );
    // #8859 resolves the twice-used component once, so the call drops its
    // default arguments as upstream's does.
    assert_eq!(
        current,
        replaced("_createComponentWithFallback(_component_Child)"),
        "only the #8859 default arguments change after #8625"
    );
}
