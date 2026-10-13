//! Complete raw Vapor output references for seven historical fixes.

use serde_json::Value;

mod support;
use support::fix_history as fixtures;

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
        let captured = expected.get("output").expect("captured output");
        let current = match CORRECTED.iter().find(|(corrected, _)| corrected == id) {
            Some((_, corrected)) => {
                let corrected =
                    serde_json::from_str::<Value>(corrected).expect("valid corrected reference");
                assert_eq!(corrected.get("options"), expected.get("options"));
                let output = corrected.get("output").expect("corrected output").clone();
                assert_ne!(&output, captured, "{id} no longer needs a correction");
                output
            }
            None => captured.clone(),
        };
        assert_eq!(
            fixtures::observe(source, *prefix),
            current,
            "complete raw Vapor output changed for {id}"
        );
    }
}

/// Cases whose captured output a later fix deliberately replaced. The capture
/// stays as it was recorded; the corrected reference is what the compiler
/// must produce now.
const CORRECTED: &[(&str, &str)] = &[(
    // The loop value's `count: total = 0` default was dropped; it is lazy now,
    // as Vue's compiler emits it.
    "destructured-aliases",
    include_str!("fixtures/fix-history/destructured-aliases.corrected-8852.expected.json"),
)];
