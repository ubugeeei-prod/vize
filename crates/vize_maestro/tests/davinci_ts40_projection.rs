//! TS-40 baseline for the current Canon and Maestro projection implementations.
//!
//! This is a migration oracle, not evidence that Davinci or S2 already owns the
//! projection. The two current generators deliberately remain independent.

mod davinci_ts40_projection_support;

use davinci_ts40_projection_support::{Drift, capture_fixture, load_matrix, verify_exact};
use vize_carton::{String, cstr};

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
                "mixed-vue2__legacy-generators-enabled"
            } else {
                "mixed-vue2__legacy-generators-disabled"
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
    let matrix = load_matrix();
    let fixture = matrix
        .fixtures
        .iter()
        .find(|fixture| fixture.id == "parent-local-import")
        .expect("real local-import mapping fixture");
    let baseline = capture_fixture(fixture);
    assert!(baseline.canon.mapping_count > 0);
    assert!(baseline.canon.import_rewrite_count > 0);
    let mut drifted = baseline.clone();
    corrupt(&mut drifted.canon.mappings_sha256);
    assert_eq!(verify_exact(&baseline, &drifted), Err(Drift::Mapping));
}

#[test]
fn diagnostic_drift_fails_closed() {
    let matrix = load_matrix();
    let fixture = matrix
        .fixtures
        .iter()
        .find(|fixture| fixture.id == "parse-recovery")
        .expect("real recovery diagnostic fixture");
    let baseline = capture_fixture(fixture);
    assert!(baseline.content_mapper.diagnostic_count > 0);
    let mut drifted = baseline.clone();
    corrupt(&mut drifted.content_mapper.diagnostics_sha256);
    assert_eq!(verify_exact(&baseline, &drifted), Err(Drift::Diagnostic));
}

fn corrupt(hash: &mut String) {
    let replacement = if hash.starts_with('a') { "b" } else { "a" };
    hash.replace_range(..1, replacement);
}
