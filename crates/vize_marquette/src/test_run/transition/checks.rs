use vize_s0::String;

use crate::ContractDiagnostic;

use super::super::admission::parse_test_run_admission_id;
use super::super::decision::{
    TestRunAdmissionDecision, TestRunDenialCode, decision_from_diagnostics, denial_code_for,
};
use super::super::validate::is_strict_timestamp_value;
use super::super::validate::rules::{
    check_digest, check_identifier, check_safe_integer, check_source_revision, check_timestamp,
};
use super::{
    TEST_RUN_TRANSITION_FORMAT, TEST_RUN_TRANSITION_FORMAT_VERSION,
    TEST_RUN_TRANSITION_MAX_ACCEPTED, TestRunTransition, test_run_transition_fingerprint,
};

/// Validates one release transition structurally.
///
/// Diagnostics use `transition.` paths and are deterministic and sorted by
/// path, code, and message. Validation confirms the record alone is
/// internally coherent — grammar, decision consistency against the
/// published diagnostic mapping, and an allowed decision accepting its own
/// evidence — but only [`verify_test_run_transition`] can confirm the
/// record extends the durable chain.
pub fn validate_test_run_transition(transition: &TestRunTransition) -> Vec<ContractDiagnostic> {
    let mut diagnostics = Vec::new();

    if transition.format.as_str() != TEST_RUN_TRANSITION_FORMAT {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_101",
            "transition.format",
            "unsupported release-transition format marker",
        ));
    }
    if transition.format_version != TEST_RUN_TRANSITION_FORMAT_VERSION {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_102",
            "transition.formatVersion",
            "unsupported release-transition format version",
        ));
    }

    if transition.sequence == 0 {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_152",
            "transition.sequence",
            "transition sequence must be at least one",
        ));
    }
    check_safe_integer(transition.sequence, "transition.sequence", &mut diagnostics);
    match &transition.previous {
        Some(_) if transition.sequence == 1 => {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_153",
                "transition.previous",
                "genesis transition must not name a predecessor",
            ));
        }
        Some(previous) => check_digest(previous, "transition.previous", &mut diagnostics),
        None if transition.sequence > 1 => diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_153",
            "transition.previous",
            "transition must name its predecessor",
        )),
        None => {}
    }
    check_timestamp(
        &transition.decided_at,
        "transition.decidedAt",
        &mut diagnostics,
    );

    let candidate = &transition.candidate;
    check_identifier(
        &candidate.application,
        "transition.candidate.application",
        &mut diagnostics,
    );
    check_identifier(
        &candidate.environment,
        "transition.candidate.environment",
        &mut diagnostics,
    );
    check_digest(
        &candidate.contract_fingerprint,
        "transition.candidate.contractFingerprint",
        &mut diagnostics,
    );
    check_source_revision(
        &candidate.source_revision,
        "transition.candidate.sourceRevision",
        &mut diagnostics,
    );
    if candidate.release.is_empty() || candidate.release.len() > 256 {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_106",
            "transition.candidate.release",
            "release must be between 1 and 256 characters",
        ));
    }
    check_digest(
        &candidate.artifact_fingerprint,
        "transition.candidate.artifactFingerprint",
        &mut diagnostics,
    );

    if parse_test_run_admission_id(&transition.evidence).is_none() {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_141",
            "transition.evidence",
            "transition evidence must be test-run: followed by 64 lowercase hexadecimal characters",
        ));
    }
    validate_accepted(transition, &mut diagnostics);
    validate_decision(transition, &mut diagnostics);

    diagnostics.sort_by(|left, right| {
        (&left.path, left.code, &left.message).cmp(&(&right.path, right.code, &right.message))
    });
    diagnostics
}

fn validate_accepted(transition: &TestRunTransition, diagnostics: &mut Vec<ContractDiagnostic>) {
    let accepted = &transition.accepted;
    if accepted.len() > TEST_RUN_TRANSITION_MAX_ACCEPTED {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_131",
            "transition.accepted",
            "transition must accept at most 4096 admission ids",
        ));
    }
    if accepted
        .iter()
        .any(|id| parse_test_run_admission_id(id).is_none())
    {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_141",
            "transition.accepted",
            "accepted admission ids must be test-run: followed by 64 lowercase hexadecimal characters",
        ));
    }
    if accepted.windows(2).any(|pair| pair[0] >= pair[1]) {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_154",
            "transition.accepted",
            "accepted admission ids must be sorted and unique",
        ));
    }
    if transition.decision.allowed && !accepted.contains(&transition.evidence) {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_156",
            "transition.accepted",
            "an allowed transition must accept its own evidence",
        ));
    }
}

fn validate_decision(transition: &TestRunTransition, diagnostics: &mut Vec<ContractDiagnostic>) {
    let decision = &transition.decision;
    if decision.allowed && (!decision.denial_codes.is_empty() || !decision.diagnostics.is_empty()) {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_155",
            "transition.decision",
            "an allowed decision must carry no diagnostics",
        ));
    }
    if !decision.allowed && decision.diagnostics.is_empty() {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_155",
            "transition.decision",
            "a denied decision must carry its diagnostics",
        ));
    }
    if decision.diagnostics.iter().any(|diagnostic| {
        !diagnostic
            .code
            .strip_prefix("VIZE_MARQUETTE_")
            .is_some_and(|digits| {
                digits.len() == 3 && digits.bytes().all(|byte| byte.is_ascii_digit())
            })
    }) {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_155",
            "transition.decision",
            "diagnostic codes must be stable VIZE_MARQUETTE codes",
        ));
    }
    let sorted = decision.diagnostics.windows(2).all(|pair| {
        (&pair[0].path, &pair[0].code, &pair[0].message)
            <= (&pair[1].path, &pair[1].code, &pair[1].message)
    });
    if !sorted {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_155",
            "transition.decision",
            "decision diagnostics must be sorted by path, code, and message",
        ));
    }
    let mut recomputed: Vec<TestRunDenialCode> = decision
        .diagnostics
        .iter()
        .map(|diagnostic| denial_code_for(&diagnostic.code, &diagnostic.path))
        .collect();
    recomputed.sort_unstable();
    recomputed.dedup();
    if decision.denial_codes != recomputed {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_155",
            "transition.decision",
            "denial codes must match the published diagnostic mapping",
        ));
    }
}

/// Verifies one release transition against the durable chain tip.
///
/// `previous` is the retained, already-verified predecessor — `None` only
/// when deciding the very first transition of a chain. The transition must
/// validate structurally, extend the predecessor's sequence, fingerprint,
/// scope, and decision time exactly, never re-accept evidence the
/// predecessor already accepted, and carry an accepted state equal to the
/// predecessor's state plus exactly the newly accepted evidence (unchanged
/// for a denial). Any diagnostic rejects the transition: a conforming host
/// must not persist it, and on recovery must discard a tip this function
/// rejects. Diagnostics, denial codes, and ordering are identical in every
/// host family, as pinned by the shared transition-decision fixtures.
pub fn verify_test_run_transition(
    transition: &TestRunTransition,
    previous: Option<&TestRunTransition>,
) -> TestRunAdmissionDecision {
    let mut diagnostics = validate_test_run_transition(transition);

    let mut prior_accepted: &[String] = &[];
    match previous {
        None => {
            if transition.sequence != 1 {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_MARQUETTE_157",
                    "transition.sequence",
                    "transition requires its predecessor to verify",
                ));
            }
        }
        Some(previous) => {
            prior_accepted = &previous.accepted;
            if transition.sequence != previous.sequence.saturating_add(1) {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_MARQUETTE_157",
                    "transition.sequence",
                    "transition must extend its predecessor's sequence",
                ));
            }
            match test_run_transition_fingerprint(previous) {
                Ok(fingerprint) => {
                    if transition.previous.as_deref() != Some(fingerprint.as_str()) {
                        diagnostics.push(ContractDiagnostic::error(
                            "VIZE_MARQUETTE_157",
                            "transition.previous",
                            "transition must name its predecessor's canonical fingerprint",
                        ));
                    }
                }
                Err(_) => diagnostics.push(ContractDiagnostic::error(
                    "VIZE_MARQUETTE_157",
                    "transition.previous",
                    "predecessor could not be canonically fingerprinted",
                )),
            }
            for (recorded, expected, path) in [
                (
                    &transition.candidate.application,
                    &previous.candidate.application,
                    "transition.candidate.application",
                ),
                (
                    &transition.candidate.environment,
                    &previous.candidate.environment,
                    "transition.candidate.environment",
                ),
            ] {
                if recorded != expected {
                    diagnostics.push(ContractDiagnostic::error(
                        "VIZE_MARQUETTE_157",
                        path,
                        "transition must stay within its predecessor's scope",
                    ));
                }
            }
            if is_strict_timestamp_value(&transition.decided_at)
                && is_strict_timestamp_value(&previous.decided_at)
                && transition.decided_at < previous.decided_at
            {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_MARQUETTE_157",
                    "transition.decidedAt",
                    "transition must not predate its predecessor",
                ));
            }
            if transition.decision.allowed && previous.accepted.contains(&transition.evidence) {
                diagnostics.push(ContractDiagnostic::error(
                    "VIZE_MARQUETTE_158",
                    "transition.evidence",
                    "accepted evidence must not be accepted again",
                ));
            }
        }
    }

    let mut expected: Vec<String> = prior_accepted.to_vec();
    if transition.decision.allowed {
        expected.push(transition.evidence.clone());
    }
    expected.sort_unstable();
    expected.dedup();
    if transition.accepted != expected {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_159",
            "transition.accepted",
            "accepted state must equal its predecessor's state plus exactly the newly accepted evidence",
        ));
    }

    diagnostics.sort_by(|left, right| {
        (&left.path, left.code, &left.message).cmp(&(&right.path, right.code, &right.message))
    });
    decision_from_diagnostics(diagnostics)
}
