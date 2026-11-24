//! Raw complete map/diagnostic Results from original historical fix inputs.

use serde_json::Value;

mod support;
use support::fix_history_map_diagnostics as fixtures;

const EXPECTED: &str =
    include_str!("fixtures/fix-history/map-diagnostic-capture-72293/first.stdout.json");

/// The capture's counter mappings: the script's segments, and nothing for the
/// render inlined after them.
const SCRIPT_MAPPINGS: &str = ";;AAKA;;;;;;;AAEA,MAAM,MAAM,EAAE,GAAG,CAAC,CAAC;AACnB,SAAS,IAAI,CAAC,EAAE;EACd,KAAK,CAAC,MAAM,CAAC,EAAE;AACjB";
/// #13547: the render maps back to the template. Its segments follow the
/// script's on later lines, so the archived mappings stay as they were.
const RENDER_MAPPINGS: &str = ";;;SATE,oCAAC,WAAQ,SAAO,yBAAS";

#[test]
fn original_map_and_diagnostic_cases_keep_the_complete_measured_results() {
    let mut expected: Value = serde_json::from_str(EXPECTED).expect("valid immutable capture");
    support::vue_binding_kinds::advance_at(
        &mut expected,
        "/cases/0/compileResult/Ok/bindings/bindings",
        &["ref"],
    )
    .expect("advance archived Vue import bindings");
    let mappings = expected
        .pointer_mut("/cases/0/compileResult/Ok/map/mappings")
        .expect("archived counter mappings");
    assert_eq!(
        *mappings, SCRIPT_MAPPINGS,
        "the capture maps the script only"
    );
    *mappings = Value::from([SCRIPT_MAPPINGS, RENDER_MAPPINGS].concat());
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
