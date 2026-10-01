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
    for (((id, source, prefix), (expected_id, expected)), captured) in
        fixtures::CASES.iter().zip(CASES).zip(observed)
    {
        assert_eq!(id, expected_id, "fixture order changed");
        assert_eq!(captured.get("id").and_then(Value::as_str), Some(*id));
        let expected = serde_json::from_str::<Value>(expected).expect("immutable case result");
        assert_eq!(captured.get("options"), expected.get("options"));
        assert_eq!(captured.get("output"), expected.get("output"));
        assert_eq!(
            fixtures::option_payload(*prefix),
            expected.get("options").expect("captured options").clone(),
            "Vapor options changed for {id}"
        );
        assert_eq!(
            fixtures::observe(source, *prefix),
            expected
                .get("output")
                .expect("captured complete output")
                .clone(),
            "complete raw Vapor output changed for {id}"
        );
    }
}
