use serde::{Deserialize, Serialize};
use vize_s0::{String, cstr};

use crate::ContractDiagnostic;

use super::canonical::{CanonicalTestRunError, test_run_fingerprint};
use super::model::{TestRunEvidence, TestRunVerificationOutcome};
use super::validate::validate_test_run;

/// Prefix of every test-run deployment admission id.
pub const TEST_RUN_ADMISSION_PREFIX: &str = "test-run:";

/// Exact release candidate a deployment gate wants evidence for.
///
/// Every field must match the record exactly; admission never falls back to
/// a newer, older, or partially matching record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestRunCandidate {
    /// Application the gate is deploying.
    pub application: String,
    /// Deployment environment the gate is promoting into.
    pub environment: String,
    /// Lowercase SHA-256 fingerprint of the application contract.
    pub contract_fingerprint: String,
    /// Exact source revision of the candidate.
    pub source_revision: String,
    /// Release the candidate belongs to.
    pub release: String,
    /// Lowercase SHA-256 fingerprint of the exact artifact being promoted.
    pub artifact_fingerprint: String,
}

/// Returns the fingerprint named by a `test-run:<sha256>` admission id.
///
/// Returns `None` unless the prefix, length, and lowercase hexadecimal
/// grammar are all exact.
pub fn parse_test_run_admission_id(id: &str) -> Option<&str> {
    let fingerprint = id.strip_prefix(TEST_RUN_ADMISSION_PREFIX)?;
    (fingerprint.len() == 64
        && fingerprint
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')))
    .then_some(fingerprint)
}

/// Returns the `test-run:<sha256>` admission id for one record.
pub fn test_run_admission_id(evidence: &TestRunEvidence) -> Result<String, CanonicalTestRunError> {
    let fingerprint = test_run_fingerprint(evidence)?;
    let mut id = String::with_capacity(TEST_RUN_ADMISSION_PREFIX.len() + fingerprint.len());
    id.push_str(TEST_RUN_ADMISSION_PREFIX);
    id.push_str(&fingerprint);
    Ok(id)
}

/// Decides whether one record admits one exact candidate at one instant.
///
/// An empty result admits the deployment. Any diagnostic rejects it: the
/// record must validate cleanly, its canonical fingerprint must be the one
/// named by `admission_id`, every candidate binding must match exactly, the
/// record must not be expired at `now`, the independent verification must
/// have accepted the run, and no skipped test may remain unaccounted for.
///
/// `now` must be a millisecond-precision UTC timestamp such as
/// `2026-01-01T00:00:00.000Z`; the fixed-width format keeps the expiry
/// comparison exact. Callers own retrieval: fetch the canonical bytes from
/// an immutable store within their own deadline, refuse oversized content,
/// and hand the parsed record here.
pub fn admit_test_run(
    evidence: &TestRunEvidence,
    candidate: &TestRunCandidate,
    admission_id: &str,
    now: &str,
) -> Vec<ContractDiagnostic> {
    let mut diagnostics = validate_test_run(evidence);

    let now_is_exact = super::validate::is_strict_timestamp_value(now);
    if !now_is_exact {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_148",
            "admission.now",
            "admission time must be a millisecond-precision UTC instant",
        ));
    }

    match parse_test_run_admission_id(admission_id) {
        None => diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_141",
            "admission.id",
            "admission id must be test-run: followed by 64 lowercase hexadecimal characters",
        )),
        Some(expected) => match test_run_fingerprint(evidence) {
            Ok(actual) if actual.as_str() == expected => {}
            Ok(_) => diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_142",
                "admission.id",
                "admission id does not name this record's canonical fingerprint",
            )),
            Err(_) => diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_142",
                "admission.id",
                "record could not be canonically fingerprinted",
            )),
        },
    }

    let bindings = [
        (
            evidence.application.as_str(),
            candidate.application.as_str(),
            "application",
        ),
        (
            evidence.environment.as_str(),
            candidate.environment.as_str(),
            "environment",
        ),
        (
            evidence.contract_fingerprint.as_str(),
            candidate.contract_fingerprint.as_str(),
            "contractFingerprint",
        ),
        (
            evidence.source_revision.as_str(),
            candidate.source_revision.as_str(),
            "sourceRevision",
        ),
        (
            evidence.release.as_str(),
            candidate.release.as_str(),
            "release",
        ),
        (
            evidence.artifact.fingerprint.as_str(),
            candidate.artifact_fingerprint.as_str(),
            "artifact.fingerprint",
        ),
    ];
    for (recorded, expected, field) in bindings {
        if recorded != expected {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_144",
                field,
                cstr!("record does not bind the candidate {field}"),
            ));
        }
    }

    if now_is_exact && evidence.valid_until.as_str() <= now {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_145",
            "validUntil",
            "record is expired at the admission time",
        ));
    }
    if evidence.verification.outcome != TestRunVerificationOutcome::Accepted {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_146",
            "verification.outcome",
            "only an accepted verification can admit a deployment",
        ));
    }
    if evidence.verification.skipped > 0 {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_147",
            "verification.skipped",
            "skipped tests are not approved for deployment admission",
        ));
    }

    diagnostics.sort_by(|left, right| {
        (&left.path, left.code, &left.message).cmp(&(&right.path, right.code, &right.message))
    });
    diagnostics
}

#[cfg(test)]
mod tests {
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

    fn codes(diagnostics: &[ContractDiagnostic]) -> Vec<&'static str> {
        diagnostics.iter().map(|value| value.code).collect()
    }

    #[test]
    fn an_exact_candidate_is_admitted() {
        let evidence = example_evidence();
        let id = test_run_admission_id(&evidence).unwrap();
        assert_eq!(parse_test_run_admission_id(&id).unwrap(), &id[9..]);
        assert_eq!(
            admit_test_run(&evidence, &example_candidate(), &id, NOW),
            Vec::new()
        );
    }

    #[test]
    fn admission_ids_must_be_exact() {
        assert!(parse_test_run_admission_id("test-run:").is_none());
        assert!(parse_test_run_admission_id("tests:abc").is_none());
        let mut uppercase = String::from(TEST_RUN_ADMISSION_PREFIX);
        for _ in 0..64 {
            uppercase.push('A');
        }
        assert!(parse_test_run_admission_id(&uppercase).is_none());

        let evidence = example_evidence();
        let rejected = admit_test_run(&evidence, &example_candidate(), "test-run:junk", NOW);
        assert_eq!(codes(&rejected), ["VIZE_MARQUETTE_141"]);
    }

    #[test]
    fn a_different_record_cannot_reuse_an_admission_id() {
        let evidence = example_evidence();
        let id = test_run_admission_id(&evidence).unwrap();
        let mut tampered = evidence.clone();
        tampered.suites[1].passed = 23;
        tampered.verification.passed = 143;
        assert_eq!(
            codes(&admit_test_run(&tampered, &example_candidate(), &id, NOW)),
            ["VIZE_MARQUETTE_142"]
        );
    }

    #[test]
    fn every_candidate_binding_must_match() {
        let evidence = example_evidence();
        let id = test_run_admission_id(&evidence).unwrap();
        let mut candidate = example_candidate();
        candidate.release = "0.299.0".into();
        candidate.environment = "staging".into();
        assert_eq!(
            codes(&admit_test_run(&evidence, &candidate, &id, NOW)),
            ["VIZE_MARQUETTE_144", "VIZE_MARQUETTE_144"]
        );
    }

    #[test]
    fn expired_and_unverified_records_are_rejected() {
        let evidence = example_evidence();
        let id = test_run_admission_id(&evidence).unwrap();
        let expired = admit_test_run(
            &evidence,
            &example_candidate(),
            &id,
            "2026-07-28T00:10:00.000Z",
        );
        assert_eq!(codes(&expired), ["VIZE_MARQUETTE_145"]);

        let malformed_now = admit_test_run(&evidence, &example_candidate(), &id, "yesterday");
        assert_eq!(codes(&malformed_now), ["VIZE_MARQUETTE_148"]);
    }

    #[test]
    fn skipped_tests_are_not_admitted() {
        let mut evidence = example_evidence();
        evidence.suites[0].skipped = 1;
        evidence.suites[0].passed = 119;
        evidence.verification.skipped = 1;
        evidence.verification.passed = 143;
        let id = test_run_admission_id(&evidence).unwrap();
        assert_eq!(
            codes(&admit_test_run(&evidence, &example_candidate(), &id, NOW)),
            ["VIZE_MARQUETTE_147"]
        );
    }
}
