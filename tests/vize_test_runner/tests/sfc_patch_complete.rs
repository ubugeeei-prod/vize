//! Keep the complete original SFC patch battery in ordinary source CI.

use std::{collections::BTreeSet, path::PathBuf};
use vize_test_runner::{load_expected, load_fixture, run_fixture_tests};

#[test]
fn every_original_sfc_patch_compares_its_complete_module() {
    let tests = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the test-runner crate is below tests")
        .to_path_buf();
    let fixture = tests.join("fixtures/sfc/patches.pkl");
    let reference = tests.join("expected/sfc/patches.snap");
    let original = load_fixture(&fixture).expect("the original complete PKL fixture must load");
    assert_eq!(original.cases.len(), 58);
    let expected_cases = load_expected(&reference);
    assert_eq!(expected_cases.len(), 58);
    let original_names: BTreeSet<_> = original.cases.iter().map(|case| &case.name).collect();
    let reference_names: BTreeSet<_> = expected_cases.iter().map(|case| &case.name).collect();
    assert_eq!(
        original_names.len(),
        58,
        "all original identities are unique"
    );
    assert_eq!(
        reference_names.len(),
        58,
        "all reference identities are unique"
    );
    assert_eq!(
        original_names, reference_names,
        "every original has exactly one full reference"
    );
    assert_eq!(
        expected_cases
            .iter()
            .map(|case| case.has_errors)
            .collect::<Vec<_>>(),
        vec![false; 58],
        "every original patch must compile; no error-case exemption"
    );
    let results = run_fixture_tests(&fixture, &reference);
    let expected: Vec<_> = original
        .cases
        .iter()
        .map(|case| (case.name.as_str(), true, None::<&str>))
        .collect();
    let actual: Vec<_> = results
        .iter()
        .map(|case| (case.name.as_str(), case.passed, case.error.as_deref()))
        .collect();
    assert_eq!(
        actual, expected,
        "all ordered original patch identities and whole module comparisons must pass"
    );
}
