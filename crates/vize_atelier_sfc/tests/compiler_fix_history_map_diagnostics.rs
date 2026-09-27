//! Raw complete map/diagnostic Results from original historical fix inputs.

use serde_json::Value;

mod support;
use support::fix_history_map_diagnostics as fixtures;

const EXPECTED: &str =
    include_str!("fixtures/fix-history/map-diagnostic-capture-72293/first.stdout.json");

#[test]
fn original_map_and_diagnostic_cases_keep_the_complete_measured_results() {
    let expected: Value = serde_json::from_str(EXPECTED).expect("valid immutable capture");
    let actual = fixtures::observe().expect("observe original public compiler routes");
    let cases = expected["cases"].as_array().expect("captured case array");
    assert_eq!(cases.len(), fixtures::CASE_IDS.len());
    for (case, id) in cases.iter().zip(fixtures::CASE_IDS) {
        assert_eq!(case["id"], *id, "missing or reordered case");
    }
    assert_eq!(
        actual, expected,
        "complete map/diagnostic Result or original options changed"
    );
}
