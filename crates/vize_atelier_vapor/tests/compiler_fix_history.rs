//! Complete raw Vapor output references for seven historical fixes.

use serde_json::Value;

#[path = "support/fix_history.rs"]
mod fixtures;

const EXPECTED: &str = include_str!("fixtures/fix-history/capture-941ff/first.stdout.json");

#[test]
fn seven_history_fixes_preserve_complete_vapor_outputs() {
    let expected = serde_json::from_str::<Value>(EXPECTED).expect("valid immutable capture");
    let cases = expected
        .get("cases")
        .and_then(Value::as_array)
        .expect("captured cases");
    assert_eq!(cases.len(), fixtures::CASES.len());
    for ((id, source, prefix), expected) in fixtures::CASES.iter().zip(cases) {
        assert_eq!(expected.get("id").and_then(Value::as_str), Some(*id));
        assert_eq!(
            fixtures::option_payload(*prefix),
            expected.get("options").expect("captured options").clone(),
            "Vapor options changed for {id}"
        );
        assert_eq!(
            fixtures::observe(source, *prefix),
            expected.get("output").expect("captured output").clone(),
            "complete raw Vapor output changed for {id}"
        );
    }
}
