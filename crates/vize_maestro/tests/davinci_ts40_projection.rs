//! TS-40 baseline for the current Canon and Maestro projection implementations.
//!
//! This is a migration oracle, not evidence that Davinci or S2 already owns the
//! projection. The two current generators deliberately remain independent.

mod davinci_ts40_projection_support;

use davinci_ts40_projection_support::{
    Drift, ProjectionRecord, capture_fixture, load_matrix, verify_exact,
};
use vize_carton::cstr;

#[test]
#[allow(clippy::disallowed_macros)] // `insta` expands to `format!`.
fn current_projection_matrix_is_exact_and_non_empty() {
    let matrix = load_matrix();
    assert!(
        !matrix.fixtures.is_empty(),
        "TS-40 matrix must not be vacuous"
    );

    for fixture in &matrix.fixtures {
        let record = capture_fixture(fixture);
        record.assert_non_empty(fixture);
        let feature = if fixture.legacy_vue2 {
            if cfg!(feature = "legacy") {
                "legacy"
            } else {
                "legacy-disabled"
            }
        } else {
            "default"
        };
        let snapshot = cstr!("davinci_ts40__{}__{feature}", fixture.id);
        insta::assert_snapshot!(snapshot.as_str(), record.render());
    }
}

#[test]
fn mapping_drift_fails_closed() {
    let baseline = ProjectionRecord::canary();
    let mut drifted = baseline.clone();
    drifted.canon.mappings_sha256.replace_range(..1, "f");
    assert_eq!(verify_exact(&baseline, &drifted), Err(Drift::Mapping));
}

#[test]
fn diagnostic_drift_fails_closed() {
    let baseline = ProjectionRecord::canary();
    let mut drifted = baseline.clone();
    drifted.canon.diagnostics_sha256.replace_range(..1, "f");
    assert_eq!(verify_exact(&baseline, &drifted), Err(Drift::Diagnostic));
}
