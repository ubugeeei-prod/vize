use vize_s0::cstr;

use crate::validate::rules::contract_path;
use crate::{ContractDiagnostic, TestRunEvidence};

mod executions;
pub(crate) mod rules;
#[cfg(test)]
mod tests;

use executions::{validate_suites, validate_targets};
use rules::{
    check_digest, check_identifier, check_retained_evidence, check_safe_integer, check_timestamp,
};

use super::model::{
    TEST_RUN_EVIDENCE_FORMAT, TEST_RUN_EVIDENCE_FORMAT_VERSION, TestRunSuiteOutcome,
    TestRunVerificationOutcome,
};

/// Returns whether `value` is a millisecond-precision UTC instant.
pub(crate) fn is_strict_timestamp_value(value: &str) -> bool {
    rules::is_strict_timestamp(value.as_bytes())
}

/// Maximum recorded target executions and selected target identifiers.
pub const TEST_RUN_MAX_TARGETS: usize = 32;

/// Maximum recorded suite executions and selected suite identifiers.
pub const TEST_RUN_MAX_SUITES: usize = 512;

/// Maximum shard index and shard count for one suite execution.
pub const TEST_RUN_MAX_SHARDS: u32 = 1024;

/// Validates a complete test-run evidence record.
///
/// Diagnostics are deterministic and sorted by path, code, and message so the
/// same record produces stable CLI, promotion, test, and CI output. A record
/// with any error diagnostic must never satisfy a deployment check.
pub fn validate_test_run(evidence: &TestRunEvidence) -> Vec<ContractDiagnostic> {
    let mut diagnostics = Vec::new();

    if evidence.format.as_str() != TEST_RUN_EVIDENCE_FORMAT {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_101",
            "format",
            "unsupported test-run evidence format marker",
        ));
    }
    if evidence.format_version != TEST_RUN_EVIDENCE_FORMAT_VERSION {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_102",
            "formatVersion",
            "unsupported test-run evidence format version",
        ));
    }

    check_identifier(&evidence.id, "id", &mut diagnostics);
    check_identifier(&evidence.application, "application", &mut diagnostics);
    check_identifier(&evidence.environment, "environment", &mut diagnostics);
    check_digest(
        &evidence.contract_fingerprint,
        "contractFingerprint",
        &mut diagnostics,
    );
    rules::check_source_revision(
        &evidence.source_revision,
        "sourceRevision",
        &mut diagnostics,
    );
    if evidence.release.is_empty() || evidence.release.len() > 256 {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_106",
            "release",
            "release must be between 1 and 256 characters",
        ));
    }

    check_identifier(&evidence.artifact.id, "artifact.id", &mut diagnostics);
    check_digest(
        &evidence.artifact.fingerprint,
        "artifact.fingerprint",
        &mut diagnostics,
    );
    if evidence.artifact.size_bytes == 0 {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_110",
            "artifact.sizeBytes",
            "artifact size must be at least one byte",
        ));
    }
    check_safe_integer(
        evidence.artifact.size_bytes,
        "artifact.sizeBytes",
        &mut diagnostics,
    );

    check_timestamp(&evidence.started_at, "startedAt", &mut diagnostics);
    check_timestamp(&evidence.completed_at, "completedAt", &mut diagnostics);
    check_timestamp(&evidence.valid_until, "validUntil", &mut diagnostics);
    if evidence.completed_at < evidence.started_at {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_112",
            "completedAt",
            "run completion must not precede its start",
        ));
    }
    if evidence.valid_until <= evidence.completed_at {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_113",
            "validUntil",
            "record expiry must come after run completion",
        ));
    }

    validate_runner(evidence, &mut diagnostics);
    validate_selection(evidence, &mut diagnostics);
    validate_targets(evidence, &mut diagnostics);
    validate_suites(evidence, &mut diagnostics);
    validate_verification(evidence, &mut diagnostics);

    diagnostics.sort_by(|left, right| {
        (&left.path, left.code, &left.message).cmp(&(&right.path, right.code, &right.message))
    });
    diagnostics
}

fn validate_runner(evidence: &TestRunEvidence, diagnostics: &mut Vec<ContractDiagnostic>) {
    let runner = &evidence.runner;
    check_identifier(&runner.identity, "runner.identity", diagnostics);
    check_retained_evidence(
        &runner.authentication_evidence,
        "runner.authenticationEvidence",
        diagnostics,
    );
    check_digest(
        &runner.invocation_fingerprint,
        "runner.invocationFingerprint",
        diagnostics,
    );
    check_retained_evidence(
        &runner.environment_evidence,
        "runner.environmentEvidence",
        diagnostics,
    );
    check_digest(
        &runner.environment_fingerprint,
        "runner.environmentFingerprint",
        diagnostics,
    );
}

fn validate_selection(evidence: &TestRunEvidence, diagnostics: &mut Vec<ContractDiagnostic>) {
    let selection = &evidence.selection;
    if selection.target_ids.is_empty() || selection.target_ids.len() > TEST_RUN_MAX_TARGETS {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_115",
            "selection.targetIds",
            "selection must include between 1 and 32 targets",
        ));
    }
    if selection.suite_ids.is_empty() || selection.suite_ids.len() > TEST_RUN_MAX_SUITES {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_115",
            "selection.suiteIds",
            "selection must include between 1 and 512 suites",
        ));
    }
    for id in &selection.target_ids {
        check_identifier(id, &contract_path("selection.targetIds", id), diagnostics);
    }
    for id in &selection.suite_ids {
        check_identifier(id, &contract_path("selection.suiteIds", id), diagnostics);
    }
}

fn validate_verification(evidence: &TestRunEvidence, diagnostics: &mut Vec<ContractDiagnostic>) {
    let verification = &evidence.verification;
    check_identifier(&verification.verifier, "verification.verifier", diagnostics);
    check_timestamp(
        &verification.completed_at,
        "verification.completedAt",
        diagnostics,
    );
    check_retained_evidence(&verification.evidence, "verification.evidence", diagnostics);
    if verification.completed_at < evidence.completed_at {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_114",
            "verification.completedAt",
            "verification must complete after the run completes",
        ));
    }

    if verification.target_count as usize != evidence.targets.len() {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_127",
            "verification.targetCount",
            "verified target count must equal the recorded target executions",
        ));
    }
    if verification.suite_count as usize != evidence.suites.len() {
        diagnostics.push(ContractDiagnostic::error(
            "VIZE_MARQUETTE_127",
            "verification.suiteCount",
            "verified suite count must equal the recorded suite executions",
        ));
    }

    let mut totals = [0u64; 4];
    for suite in &evidence.suites {
        totals[0] += u64::from(suite.passed);
        totals[1] += u64::from(suite.failed);
        totals[2] += u64::from(suite.skipped);
        totals[3] += u64::from(suite.retries);
    }
    let recorded = [
        (u64::from(verification.passed), "verification.passed"),
        (u64::from(verification.failed), "verification.failed"),
        (u64::from(verification.skipped), "verification.skipped"),
        (u64::from(verification.retries), "verification.retries"),
    ];
    for ((total, (value, path)), name) in totals
        .iter()
        .zip(recorded)
        .zip(["passed", "failed", "skipped", "retried"])
    {
        if *total != value {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_128",
                path,
                cstr!("verified {name} total must equal the sum over suite executions"),
            ));
        }
    }

    if verification.outcome == TestRunVerificationOutcome::Accepted {
        let clean = evidence
            .suites
            .iter()
            .all(|suite| suite.outcome == TestRunSuiteOutcome::Passed && suite.failed == 0);
        if !clean || verification.failed > 0 {
            diagnostics.push(ContractDiagnostic::error(
                "VIZE_MARQUETTE_129",
                "verification.outcome",
                "verification cannot accept a run with failed or cancelled executions",
            ));
        }
    }
}

impl TestRunEvidence {
    /// Returns all structural and consistency diagnostics in stable order.
    pub fn validate(&self) -> Vec<ContractDiagnostic> {
        validate_test_run(self)
    }
}
