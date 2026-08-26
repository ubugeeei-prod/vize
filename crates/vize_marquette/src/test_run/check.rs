use serde::{Deserialize, Serialize};
use vize_s0::{String, cstr};

use crate::ContractDiagnostic;

use super::admission::{TestRunCandidate, admit_test_run, parse_test_run_admission_id};
use super::decision::{TestRunAdmissionDecision, decision_from_diagnostics};
use super::model::TestRunEvidence;
use super::validate::is_strict_timestamp_value;
use super::validate::rules::{
    check_digest, check_identifier, check_source_revision, check_timestamp,
};

/// Serialized `format` marker for retained tests-check records.
///
/// Readers must reject any other value before trusting the record.
pub const TEST_RUN_CHECK_FORMAT: &str = "vize.test-run.check";

/// Current serialized tests-check format.
///
/// Readers must reject a higher value until they explicitly support it.
pub const TEST_RUN_CHECK_FORMAT_VERSION: u32 = 1;

/// Retained, release-bound `tests` check for one deployment decision.
///
/// The record replaces every generic test-result reference — a summary blob,
/// a report path, or a green workflow label — with the exact
/// `test-run:<sha256>` admission id of an independently verified run, the
/// six candidate facts the run was admitted for, and the identity and
/// instant of the independent observer that recorded the admission. A
/// release decision retaining anything else as its tests evidence cannot
/// pass [`verify_test_run_check`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestRunCheck {
    /// Serialized format marker; always [`TEST_RUN_CHECK_FORMAT`].
    pub format: String,
    /// Serialized format version.
    ///
    /// Defaults to [`TEST_RUN_CHECK_FORMAT_VERSION`].
    #[serde(default = "default_test_run_check_format_version")]
    pub format_version: u32,
    /// Exact `test-run:<sha256>` admission id of the observed run.
    pub evidence: String,
    /// Exact candidate facts the run was admitted for.
    pub candidate: TestRunCandidate,
    /// Identity of the independent observer that recorded the admission.
    ///
    /// The observer is the trusted promotion boundary, never the runner
    /// that executed the tests.
    pub observer: String,
    /// Millisecond-precision UTC instant the admission was observed.
    pub observed_at: String,
}

const fn default_test_run_check_format_version() -> u32 {
    TEST_RUN_CHECK_FORMAT_VERSION
}

/// Validates a retained tests-check record structurally.
///
/// Diagnostics use `check.` paths and are deterministic and sorted by path,
/// code, and message. A generic evidence reference fails here with
/// `VIZE_MARQUETTE_141`: only an exact `test-run:<sha256>` admission id can
/// name retained test evidence. Structural validity never admits anything by
/// itself; [`verify_test_run_check`] must confirm the record against the
/// caller's candidate and the retained run.
pub fn validate_test_run_check(check: &TestRunCheck) -> Vec<ContractDiagnostic> {
    let mut diagnostics = Vec::new();

    if check.format.as_str() != TEST_RUN_CHECK_FORMAT {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_101",
            "check.format",
            "unsupported tests-check format marker",
        ));
    }
    if check.format_version != TEST_RUN_CHECK_FORMAT_VERSION {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_102",
            "check.formatVersion",
            "unsupported tests-check format version",
        ));
    }

    if parse_test_run_admission_id(&check.evidence).is_none() {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_141",
            "check.evidence",
            "check evidence must be test-run: followed by 64 lowercase hexadecimal characters",
        ));
    }

    let candidate = &check.candidate;
    check_identifier(
        &candidate.application,
        "check.candidate.application",
        &mut diagnostics,
    );
    check_identifier(
        &candidate.environment,
        "check.candidate.environment",
        &mut diagnostics,
    );
    check_digest(
        &candidate.contract_fingerprint,
        "check.candidate.contractFingerprint",
        &mut diagnostics,
    );
    check_source_revision(
        &candidate.source_revision,
        "check.candidate.sourceRevision",
        &mut diagnostics,
    );
    if candidate.release.is_empty() || candidate.release.len() > 256 {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_106",
            "check.candidate.release",
            "release must be between 1 and 256 characters",
        ));
    }
    check_digest(
        &candidate.artifact_fingerprint,
        "check.candidate.artifactFingerprint",
        &mut diagnostics,
    );

    check_identifier(&check.observer, "check.observer", &mut diagnostics);
    check_timestamp(&check.observed_at, "check.observedAt", &mut diagnostics);

    diagnostics.sort_by(|left, right| {
        (&left.path, left.code, &left.message).cmp(&(&right.path, right.code, &right.message))
    });
    diagnostics
}

/// Verifies one retained tests check against the caller's own facts.
///
/// The caller supplies the candidate it is deciding from its own trusted
/// facts; the retained check must validate structurally, bind that candidate
/// exactly, name an observer independent from the run's runner, and be
/// observed no earlier than the run's completed verification. The referenced
/// record is then admitted exactly like
/// [`admit_test_run`](super::admit_test_run): canonical fingerprint,
/// candidate bindings, expiry at `now`, verification outcome, and
/// skipped-test accounting all fail closed. Diagnostics, denial codes, and
/// ordering are identical in every host family, as pinned by the shared
/// `tests/fixtures/test-run-evidence` check-decision fixtures.
pub fn verify_test_run_check(
    check: &TestRunCheck,
    candidate: &TestRunCandidate,
    evidence: &TestRunEvidence,
    now: &str,
) -> TestRunAdmissionDecision {
    let mut diagnostics = validate_test_run_check(check);

    let bindings = [
        (
            check.candidate.application.as_str(),
            candidate.application.as_str(),
            "check.candidate.application",
            "application",
        ),
        (
            check.candidate.environment.as_str(),
            candidate.environment.as_str(),
            "check.candidate.environment",
            "environment",
        ),
        (
            check.candidate.contract_fingerprint.as_str(),
            candidate.contract_fingerprint.as_str(),
            "check.candidate.contractFingerprint",
            "contract fingerprint",
        ),
        (
            check.candidate.source_revision.as_str(),
            candidate.source_revision.as_str(),
            "check.candidate.sourceRevision",
            "source revision",
        ),
        (
            check.candidate.release.as_str(),
            candidate.release.as_str(),
            "check.candidate.release",
            "release",
        ),
        (
            check.candidate.artifact_fingerprint.as_str(),
            candidate.artifact_fingerprint.as_str(),
            "check.candidate.artifactFingerprint",
            "artifact fingerprint",
        ),
    ];
    for (recorded, expected, path, field) in bindings {
        if recorded != expected {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_149",
                path,
                cstr!("check does not bind the candidate {field}"),
            ));
        }
    }

    if check.observer == evidence.runner.identity {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_151",
            "check.observer",
            "check observer must be independent from the run's runner",
        ));
    }
    if is_strict_timestamp_value(&check.observed_at)
        && check.observed_at < evidence.verification.completed_at
    {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_150",
            "check.observedAt",
            "observation must not precede the completed verification",
        ));
    }

    diagnostics.extend(admit_test_run(evidence, candidate, &check.evidence, now));
    diagnostics.sort_by(|left, right| {
        (&left.path, left.code, &left.message).cmp(&(&right.path, right.code, &right.message))
    });
    decision_from_diagnostics(diagnostics)
}

#[cfg(test)]
mod tests {
    use crate::test_run::admission::test_run_admission_id;
    use crate::test_run::decision::TestRunDenialCode;
    use crate::test_run::model_tests::example_evidence;

    use super::*;

    const NOW: &str = "2026-07-22T00:00:00.000Z";

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

    fn example_check() -> TestRunCheck {
        TestRunCheck {
            format: TEST_RUN_CHECK_FORMAT.into(),
            format_version: TEST_RUN_CHECK_FORMAT_VERSION,
            evidence: test_run_admission_id(&example_evidence()).unwrap(),
            candidate: example_candidate(),
            observer: "release.gate".into(),
            observed_at: "2026-07-21T00:12:00.000Z".into(),
        }
    }

    #[test]
    fn a_release_bound_check_verifies() {
        let decision = verify_test_run_check(
            &example_check(),
            &example_candidate(),
            &example_evidence(),
            NOW,
        );
        assert!(decision.allowed);
        assert_eq!(decision.diagnostics, Vec::new());
    }

    #[test]
    fn generic_test_result_references_fail_closed() {
        let mut check = example_check();
        check.evidence = "reports/junit-summary.xml".into();
        let decision =
            verify_test_run_check(&check, &example_candidate(), &example_evidence(), NOW);
        assert!(!decision.allowed);
        assert_eq!(
            decision.denial_codes,
            [TestRunDenialCode::AdmissionIdMalformed]
        );
    }

    #[test]
    fn substituted_checks_and_dependent_observers_are_rejected() {
        let mut check = example_check();
        check.candidate.release = "0.999.0".into();
        check.observer = example_evidence().runner.identity;
        check.observed_at = "2026-07-21T00:10:30.000Z".into();
        let decision =
            verify_test_run_check(&check, &example_candidate(), &example_evidence(), NOW);
        assert_eq!(
            decision.denial_codes,
            [
                TestRunDenialCode::CheckCandidateMismatch,
                TestRunDenialCode::CheckInvalid,
                TestRunDenialCode::CheckObserverNotIndependent,
            ]
        );
    }

    #[test]
    fn malformed_checks_are_check_invalid() {
        let mut check = example_check();
        check.format = "vize.test-results".into();
        check.observer = "Release Gate!".into();
        let decision =
            verify_test_run_check(&check, &example_candidate(), &example_evidence(), NOW);
        assert_eq!(decision.denial_codes, [TestRunDenialCode::CheckInvalid]);
        assert_eq!(decision.diagnostics.len(), 2);
    }
}
