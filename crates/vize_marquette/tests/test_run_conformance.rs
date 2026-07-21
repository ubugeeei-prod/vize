//! Cross-language conformance over the shared test-run evidence fixtures.
//!
//! The TypeScript package reads the same files, so both implementations must
//! agree on validation codes, canonical bytes, and the admitted fingerprint.

use std::{fs, path::PathBuf};

use vize_marquette::{
    TestRunEvidence, canonical_test_run_json, parse_test_run_admission_id, test_run_admission_id,
    test_run_fingerprint,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/test-run-evidence")
        .join(name)
}

#[test]
fn accepts_the_shared_valid_record_and_matches_canonical_artifacts() {
    let evidence: TestRunEvidence =
        serde_json::from_slice(&fs::read(fixture("valid.json")).unwrap()).unwrap();
    assert!(evidence.validate().is_empty());

    let canonical = canonical_test_run_json(&evidence).unwrap();
    let expected = fs::read_to_string(fixture("valid.canonical")).unwrap();
    assert_eq!(canonical.as_slice(), expected.trim().as_bytes());

    let fingerprint = fs::read_to_string(fixture("valid.sha256")).unwrap();
    assert_eq!(
        test_run_fingerprint(&evidence).unwrap().as_str(),
        fingerprint.trim()
    );

    let admission_id = test_run_admission_id(&evidence).unwrap();
    assert_eq!(
        parse_test_run_admission_id(&admission_id).unwrap(),
        fingerprint.trim()
    );
}

#[test]
fn returns_the_shared_invalid_diagnostic_codes() {
    let evidence: TestRunEvidence =
        serde_json::from_slice(&fs::read(fixture("invalid.json")).unwrap()).unwrap();
    let actual = evidence
        .validate()
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    let expected: Vec<vize_carton::String> =
        serde_json::from_slice(&fs::read(fixture("invalid.expected.json")).unwrap()).unwrap();
    assert_eq!(actual, expected);
}
