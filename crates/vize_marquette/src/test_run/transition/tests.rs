use vize_s0::String;

use crate::test_run::decision::TestRunDenialCode;
use crate::test_run::model_tests::example_evidence;

use super::super::admission::TestRunCandidate;
use super::*;

fn admission_id(fill: char) -> String {
    let mut id = String::from("test-run:");
    for _ in 0..64 {
        id.push(fill);
    }
    id
}

fn example_candidate() -> TestRunCandidate {
    let evidence = example_evidence();
    TestRunCandidate {
        application: evidence.application,
        environment: evidence.environment,
        contract_fingerprint: evidence.contract_fingerprint,
        source_revision: evidence.source_revision,
        release: evidence.release,
        artifact_fingerprint: evidence.artifact.fingerprint,
    }
}

fn allowed_decision() -> TestRunRetainedDecision {
    TestRunRetainedDecision {
        allowed: true,
        denial_codes: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn genesis() -> TestRunTransition {
    TestRunTransition {
        format: TEST_RUN_TRANSITION_FORMAT.into(),
        format_version: TEST_RUN_TRANSITION_FORMAT_VERSION,
        sequence: 1,
        previous: None,
        decided_at: "2026-07-21T00:12:00.000Z".into(),
        candidate: example_candidate(),
        evidence: admission_id('a'),
        decision: allowed_decision(),
        accepted: vec![admission_id('a')],
    }
}

fn next(previous: &TestRunTransition, evidence: String) -> TestRunTransition {
    let mut accepted = previous.accepted.clone();
    accepted.push(evidence.clone());
    accepted.sort_unstable();
    accepted.dedup();
    TestRunTransition {
        sequence: previous.sequence + 1,
        previous: Some(test_run_transition_fingerprint(previous).unwrap()),
        decided_at: "2026-07-22T00:00:00.000Z".into(),
        evidence,
        accepted,
        ..previous.clone()
    }
}

#[test]
fn an_atomic_chain_verifies_transition_by_transition() {
    let first = genesis();
    assert_eq!(validate_test_run_transition(&first), Vec::new());
    let opening = verify_test_run_transition(&first, None);
    assert!(opening.allowed);

    let second = next(&first, admission_id('b'));
    let extension = verify_test_run_transition(&second, Some(&first));
    assert!(extension.allowed);
    assert_eq!(extension.diagnostics, Vec::new());
}

#[test]
fn a_denied_decision_preserves_the_accepted_state() {
    let first = genesis();
    let mut denial = next(&first, admission_id('c'));
    denial.decision = TestRunRetainedDecision {
        allowed: false,
        denial_codes: vec![TestRunDenialCode::RecordExpired],
        diagnostics: vec![TestRunRetainedDiagnostic {
            code: "VIZE_MARQUETTE_145".into(),
            severity: crate::DiagnosticSeverity::Error,
            path: "validUntil".into(),
            message: "record is expired at the admission time".into(),
        }],
    };
    denial.accepted = first.accepted.clone();
    assert!(verify_test_run_transition(&denial, Some(&first)).allowed);
}

#[test]
fn replayed_evidence_cannot_be_accepted_again() {
    let first = genesis();
    let mut replay = next(&first, admission_id('a'));
    replay.accepted = first.accepted.clone();
    let decision = verify_test_run_transition(&replay, Some(&first));
    assert!(!decision.allowed);
    assert_eq!(
        decision.denial_codes,
        [TestRunDenialCode::TransitionReplayed]
    );
}

#[test]
fn partial_and_split_states_fail_closed() {
    let first = genesis();

    let mut partial = next(&first, admission_id('b'));
    partial.accepted = first.accepted.clone();
    assert_eq!(
        verify_test_run_transition(&partial, Some(&first)).denial_codes,
        [TestRunDenialCode::TransitionStateMismatch]
    );

    let mut split = next(&first, admission_id('b'));
    split.previous = Some(crate::test_run::model_tests::filled('9', 64));
    assert_eq!(
        verify_test_run_transition(&split, Some(&first)).denial_codes,
        [TestRunDenialCode::TransitionChainBroken]
    );
}

#[test]
fn structural_faults_are_transition_invalid() {
    let mut broken = genesis();
    broken.format = "vize.release.log".into();
    broken.accepted = vec![admission_id('b'), admission_id('a')];
    let decision = verify_test_run_transition(&broken, None);
    assert!(!decision.allowed);
    assert_eq!(
        decision.denial_codes,
        [
            TestRunDenialCode::TransitionInvalid,
            TestRunDenialCode::TransitionStateMismatch,
        ]
    );
}

#[test]
fn canonicalization_is_order_independent_and_binding() {
    let first = genesis();
    let second = next(&first, admission_id('b'));
    let mut shuffled = second.clone();
    shuffled.accepted.reverse();
    assert_eq!(
        canonical_test_run_transition_json(&second).unwrap(),
        canonical_test_run_transition_json(&shuffled).unwrap()
    );

    let mut tampered = second.clone();
    tampered.accepted.remove(0);
    assert_ne!(
        test_run_transition_fingerprint(&second).unwrap(),
        test_run_transition_fingerprint(&tampered).unwrap()
    );
}
