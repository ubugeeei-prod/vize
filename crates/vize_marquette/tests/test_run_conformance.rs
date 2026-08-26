//! Cross-language conformance over the shared test-run evidence fixtures.
//!
//! The TypeScript package reads the same files, so both implementations must
//! agree on validation codes, canonical bytes, and the admitted fingerprint.

use std::{fs, path::PathBuf};

use vize_marquette::{
    TestRunCandidate, TestRunCheck, TestRunEvidence, TestRunTransition, canonical_test_run_json,
    decide_test_run_admission, parse_test_run_admission_id, test_run_admission_id,
    test_run_fingerprint, verify_test_run_check, verify_test_run_transition,
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
    let expected: Vec<vize_s0::String> =
        serde_json::from_slice(&fs::read(fixture("invalid.expected.json")).unwrap()).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn reproduces_every_shared_admission_decision() {
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture("admission-decisions.json")).unwrap()).unwrap();
    let cases = document["cases"].as_array().unwrap();
    assert!(!cases.is_empty());

    for case in cases {
        let name = case["name"].as_str().unwrap();
        let evidence: TestRunEvidence =
            serde_json::from_slice(&fs::read(fixture(case["evidence"].as_str().unwrap())).unwrap())
                .unwrap();
        let candidate: TestRunCandidate =
            serde_json::from_value(case["candidate"].clone()).unwrap();
        let decision = decide_test_run_admission(
            &evidence,
            &candidate,
            case["admissionId"].as_str().unwrap(),
            case["now"].as_str().unwrap(),
        );
        assert!(case["decision"].is_object(), "{name} must pin a decision");
        assert_eq!(
            serde_json::to_value(&decision).unwrap(),
            case["decision"],
            "decision mismatch for case {name}",
        );
    }
}

#[test]
fn reproduces_every_shared_transition_decision() {
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture("transition-decisions.json")).unwrap()).unwrap();
    let transitions = &document["transitions"];
    let load = |name: &str| -> TestRunTransition {
        serde_json::from_value(transitions[name].clone()).unwrap()
    };
    let cases = document["cases"].as_array().unwrap();
    assert!(!cases.is_empty());

    for case in cases {
        let name = case["name"].as_str().unwrap();
        let current = load(case["current"].as_str().unwrap());
        let previous = case["previous"].as_str().map(load);
        let decision = verify_test_run_transition(&current, previous.as_ref());
        assert!(case["decision"].is_object(), "{name} must pin a decision");
        assert_eq!(
            serde_json::to_value(&decision).unwrap(),
            case["decision"],
            "decision mismatch for case {name}",
        );
    }
}

#[test]
fn reproduces_every_shared_check_decision() {
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture("check-decisions.json")).unwrap()).unwrap();
    let cases = document["cases"].as_array().unwrap();
    assert!(!cases.is_empty());

    for case in cases {
        let name = case["name"].as_str().unwrap();
        let evidence: TestRunEvidence =
            serde_json::from_slice(&fs::read(fixture(case["evidence"].as_str().unwrap())).unwrap())
                .unwrap();
        let check: TestRunCheck = serde_json::from_value(case["check"].clone()).unwrap();
        let candidate: TestRunCandidate =
            serde_json::from_value(case["candidate"].clone()).unwrap();
        let decision =
            verify_test_run_check(&check, &candidate, &evidence, case["now"].as_str().unwrap());
        assert!(case["decision"].is_object(), "{name} must pin a decision");
        assert_eq!(
            serde_json::to_value(&decision).unwrap(),
            case["decision"],
            "decision mismatch for case {name}",
        );
    }
}
