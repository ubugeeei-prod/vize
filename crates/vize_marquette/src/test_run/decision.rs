use serde::{Deserialize, Serialize};

use crate::ContractDiagnostic;

use super::admission::{TestRunCandidate, admit_test_run};
use super::model::TestRunEvidence;

/// Stable machine-readable cause class of one admission denial.
///
/// The vocabulary is shared by every backend family: a JavaScript, Rust, Go,
/// or JVM host must derive the same codes from the same diagnostics, as
/// pinned by the shared `tests/fixtures/test-run-evidence` decision fixtures.
/// Codes are append-only: they are never renamed, renumbered, reused, or
/// removed, and a new rejection cause always ships with a new code. Variants
/// are declared in the lexicographic order of their serialized names, so the
/// derived ordering matches the documented decision ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TestRunDenialCode {
    /// The admission id fails the `test-run:<64 lowercase hex>` grammar.
    AdmissionIdMalformed,
    /// The admission id does not name the record's canonical fingerprint.
    AdmissionIdMismatch,
    /// The admission time is not a millisecond-precision UTC instant.
    AdmissionTimeMalformed,
    /// The record does not bind the candidate application.
    CandidateApplicationMismatch,
    /// The record does not bind the candidate artifact fingerprint.
    CandidateArtifactFingerprintMismatch,
    /// The record does not bind the candidate contract fingerprint.
    CandidateContractFingerprintMismatch,
    /// The record does not bind the candidate environment.
    CandidateEnvironmentMismatch,
    /// The record does not bind the candidate release.
    CandidateReleaseMismatch,
    /// The record does not bind the candidate source revision.
    CandidateSourceRevisionMismatch,
    /// The retained tests check does not bind the caller's candidate.
    CheckCandidateMismatch,
    /// The retained tests check record itself failed validation.
    CheckInvalid,
    /// The tests check observer is not independent from the run's runner.
    CheckObserverNotIndependent,
    /// The record is expired at the admission time.
    RecordExpired,
    /// The record failed structural or consistency validation.
    RecordInvalid,
    /// The record accounts skipped tests, which admission never approves.
    SkippedTestsRecorded,
    /// The transition does not extend the verified chain tip exactly.
    TransitionChainBroken,
    /// The transition record itself failed validation.
    TransitionInvalid,
    /// The transition re-accepts evidence its predecessor already accepted.
    TransitionReplayed,
    /// The transition's accepted state does not match its decision.
    TransitionStateMismatch,
    /// The independent verification did not accept the run.
    VerificationNotAccepted,
}

/// Every denial code, in the stable lexicographic decision order.
pub const TEST_RUN_DENIAL_CODES: [TestRunDenialCode; 20] = [
    TestRunDenialCode::AdmissionIdMalformed,
    TestRunDenialCode::AdmissionIdMismatch,
    TestRunDenialCode::AdmissionTimeMalformed,
    TestRunDenialCode::CandidateApplicationMismatch,
    TestRunDenialCode::CandidateArtifactFingerprintMismatch,
    TestRunDenialCode::CandidateContractFingerprintMismatch,
    TestRunDenialCode::CandidateEnvironmentMismatch,
    TestRunDenialCode::CandidateReleaseMismatch,
    TestRunDenialCode::CandidateSourceRevisionMismatch,
    TestRunDenialCode::CheckCandidateMismatch,
    TestRunDenialCode::CheckInvalid,
    TestRunDenialCode::CheckObserverNotIndependent,
    TestRunDenialCode::RecordExpired,
    TestRunDenialCode::RecordInvalid,
    TestRunDenialCode::SkippedTestsRecorded,
    TestRunDenialCode::TransitionChainBroken,
    TestRunDenialCode::TransitionInvalid,
    TestRunDenialCode::TransitionReplayed,
    TestRunDenialCode::TransitionStateMismatch,
    TestRunDenialCode::VerificationNotAccepted,
];

/// Structured allow-or-deny admission decision for one exact candidate.
///
/// The decision carries the machine-readable cause classes next to the exact
/// diagnostics, so a deployment gate in any language can act on one bounded
/// vocabulary while operators keep the full explanation. Serialization
/// follows `schema/test-run-admission.schema.json`; decisions are outputs,
/// so no deserializer is provided and a gate must never trust a decision it
/// did not compute itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestRunAdmissionDecision {
    /// Whether the record admits the candidate; true only with no diagnostics.
    pub allowed: bool,
    /// Deduplicated denial causes sorted lexicographically; empty when allowed.
    pub denial_codes: Vec<TestRunDenialCode>,
    /// Complete diagnostics in the stable path, code, message order.
    pub diagnostics: Vec<ContractDiagnostic>,
}

/// Returns the stable denial code one admission diagnostic maps to.
///
/// The mapping is total and identical in every host family: admission codes
/// `VIZE_MARQUETTE_141` through `VIZE_MARQUETTE_148` map to their exact
/// cause, `VIZE_MARQUETTE_144` distinguishes the mismatched candidate
/// binding by its diagnostic path, `VIZE_MARQUETTE_149` through
/// `VIZE_MARQUETTE_151` map to their tests-check cause,
/// `VIZE_MARQUETTE_156` through `VIZE_MARQUETTE_159` map to their
/// transition cause, every other diagnostic at a `check.` path is a
/// [`TestRunDenialCode::CheckInvalid`] tests-check validation failure,
/// every other diagnostic at a `transition.` path is a
/// [`TestRunDenialCode::TransitionInvalid`] transition validation failure,
/// and every remaining diagnostic is a
/// [`TestRunDenialCode::RecordInvalid`] record-validation failure.
pub fn test_run_denial_code(diagnostic: &ContractDiagnostic) -> TestRunDenialCode {
    denial_code_for(diagnostic.code, diagnostic.path.as_str())
}

/// Total (code, path) form of the mapping shared with retained diagnostics.
pub(crate) fn denial_code_for(code: &str, path: &str) -> TestRunDenialCode {
    match (code, path) {
        ("VIZE_MARQUETTE_141", _) => TestRunDenialCode::AdmissionIdMalformed,
        ("VIZE_MARQUETTE_142", _) => TestRunDenialCode::AdmissionIdMismatch,
        ("VIZE_MARQUETTE_144", "application") => TestRunDenialCode::CandidateApplicationMismatch,
        ("VIZE_MARQUETTE_144", "artifact.fingerprint") => {
            TestRunDenialCode::CandidateArtifactFingerprintMismatch
        }
        ("VIZE_MARQUETTE_144", "contractFingerprint") => {
            TestRunDenialCode::CandidateContractFingerprintMismatch
        }
        ("VIZE_MARQUETTE_144", "environment") => TestRunDenialCode::CandidateEnvironmentMismatch,
        ("VIZE_MARQUETTE_144", "release") => TestRunDenialCode::CandidateReleaseMismatch,
        ("VIZE_MARQUETTE_144", "sourceRevision") => {
            TestRunDenialCode::CandidateSourceRevisionMismatch
        }
        ("VIZE_MARQUETTE_145", _) => TestRunDenialCode::RecordExpired,
        ("VIZE_MARQUETTE_146", _) => TestRunDenialCode::VerificationNotAccepted,
        ("VIZE_MARQUETTE_147", _) => TestRunDenialCode::SkippedTestsRecorded,
        ("VIZE_MARQUETTE_148", _) => TestRunDenialCode::AdmissionTimeMalformed,
        ("VIZE_MARQUETTE_149", _) => TestRunDenialCode::CheckCandidateMismatch,
        ("VIZE_MARQUETTE_150", _) => TestRunDenialCode::CheckInvalid,
        ("VIZE_MARQUETTE_151", _) => TestRunDenialCode::CheckObserverNotIndependent,
        ("VIZE_MARQUETTE_156", _) => TestRunDenialCode::TransitionStateMismatch,
        ("VIZE_MARQUETTE_157", _) => TestRunDenialCode::TransitionChainBroken,
        ("VIZE_MARQUETTE_158", _) => TestRunDenialCode::TransitionReplayed,
        ("VIZE_MARQUETTE_159", _) => TestRunDenialCode::TransitionStateMismatch,
        (_, path) if path.starts_with("check.") => TestRunDenialCode::CheckInvalid,
        (_, path) if path.starts_with("transition.") => TestRunDenialCode::TransitionInvalid,
        _ => TestRunDenialCode::RecordInvalid,
    }
}

/// Builds the structured decision for one complete diagnostic set.
pub(crate) fn decision_from_diagnostics(
    diagnostics: Vec<ContractDiagnostic>,
) -> TestRunAdmissionDecision {
    let mut denial_codes: Vec<TestRunDenialCode> =
        diagnostics.iter().map(test_run_denial_code).collect();
    denial_codes.sort_unstable();
    denial_codes.dedup();
    TestRunAdmissionDecision {
        allowed: diagnostics.is_empty(),
        denial_codes,
        diagnostics,
    }
}

/// Decides one candidate and returns the structured admission decision.
///
/// The decision wraps [`admit_test_run`]: `diagnostics` is exactly its
/// result, `denial_codes` maps every diagnostic through
/// [`test_run_denial_code`] and then deduplicates and sorts the codes
/// lexicographically, and `allowed` is true only when both are empty. Inputs
/// carry the same obligations as [`admit_test_run`].
pub fn decide_test_run_admission(
    evidence: &TestRunEvidence,
    candidate: &TestRunCandidate,
    admission_id: &str,
    now: &str,
) -> TestRunAdmissionDecision {
    decision_from_diagnostics(admit_test_run(evidence, candidate, admission_id, now))
}

#[cfg(test)]
mod tests {
    use crate::test_run::admission::test_run_admission_id;
    use crate::test_run::model_tests::example_evidence;

    use super::*;

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

    const NOW: &str = "2026-07-22T00:00:00.000Z";

    #[test]
    fn the_vocabulary_is_sorted_and_serializes_kebab_case() {
        let serialized: [vize_s0::String; 20] = TEST_RUN_DENIAL_CODES
            .map(|code| serde_json::to_value(code).unwrap().as_str().unwrap().into());
        let mut sorted = serialized.clone();
        sorted.sort();
        assert_eq!(serialized, sorted);
        assert_eq!(serialized[0], "admission-id-malformed");
        assert_eq!(serialized[10], "check-invalid");
        assert_eq!(serialized[13], "record-invalid");
        assert_eq!(serialized[16], "transition-invalid");

        let mut ordered = TEST_RUN_DENIAL_CODES;
        ordered.sort_unstable();
        assert_eq!(ordered, TEST_RUN_DENIAL_CODES);
    }

    #[test]
    fn an_admitted_candidate_has_no_denial_codes() {
        let evidence = example_evidence();
        let id = test_run_admission_id(&evidence).unwrap();
        let decision = decide_test_run_admission(&evidence, &example_candidate(), &id, NOW);
        assert!(decision.allowed);
        assert_eq!(decision.denial_codes, Vec::new());
        assert_eq!(decision.diagnostics, Vec::new());
    }

    #[test]
    fn every_binding_mismatch_has_a_distinct_code() {
        let evidence = example_evidence();
        let id = test_run_admission_id(&evidence).unwrap();
        let candidate = TestRunCandidate {
            application: "other".into(),
            environment: "other".into(),
            contract_fingerprint: "other".into(),
            source_revision: "other".into(),
            release: "other".into(),
            artifact_fingerprint: "other".into(),
        };
        let decision = decide_test_run_admission(&evidence, &candidate, &id, NOW);
        assert!(!decision.allowed);
        assert_eq!(
            decision.denial_codes,
            [
                TestRunDenialCode::CandidateApplicationMismatch,
                TestRunDenialCode::CandidateArtifactFingerprintMismatch,
                TestRunDenialCode::CandidateContractFingerprintMismatch,
                TestRunDenialCode::CandidateEnvironmentMismatch,
                TestRunDenialCode::CandidateReleaseMismatch,
                TestRunDenialCode::CandidateSourceRevisionMismatch,
            ]
        );
        assert_eq!(decision.diagnostics.len(), 6);
    }

    #[test]
    fn repeated_causes_deduplicate_into_sorted_codes() {
        let mut evidence = example_evidence();
        evidence.application = "not-the-candidate".into();
        evidence.artifact.fingerprint = "tampered".into();
        evidence.contract_fingerprint = "also-tampered".into();
        let decision = decide_test_run_admission(
            &evidence,
            &example_candidate(),
            "test-run:junk",
            "2099-01-01T00:00:00.000+0",
        );
        assert!(!decision.allowed);
        assert_eq!(
            decision.denial_codes,
            [
                TestRunDenialCode::AdmissionIdMalformed,
                TestRunDenialCode::AdmissionTimeMalformed,
                TestRunDenialCode::CandidateApplicationMismatch,
                TestRunDenialCode::CandidateArtifactFingerprintMismatch,
                TestRunDenialCode::CandidateContractFingerprintMismatch,
                TestRunDenialCode::RecordInvalid,
            ]
        );
        assert!(decision.diagnostics.len() > decision.denial_codes.len());
    }
}
