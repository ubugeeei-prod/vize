//! End-to-end tests for the consolidated vize-marquette CLI.

use std::collections::BTreeSet;
use std::process::Command;

use vize_marquette::{
    TEST_RUN_EVIDENCE_FORMAT, TestRunArtifact, TestRunEvidence, TestRunIsolation,
    TestRunRetainedEvidence, TestRunRunner, TestRunSelection, TestRunSuiteExecution,
    TestRunSuiteKind, TestRunSuiteOutcome, TestRunTargetExecution, TestRunTargetKind,
    TestRunVerification, TestRunVerificationOutcome, test_run_fingerprint,
};

fn digest(fill: char) -> vize_s0::String {
    let mut output = vize_s0::String::with_capacity(64);
    for _ in 0..64 {
        output.push(fill);
    }
    output
}

fn retained(fill: char) -> TestRunRetainedEvidence {
    let fingerprint = digest(fill);
    let mut reference = vize_s0::String::from("sha256:");
    reference.push_str(&fingerprint);
    TestRunRetainedEvidence::new(reference, fingerprint)
}

fn example_evidence() -> TestRunEvidence {
    let mut revision = vize_s0::String::with_capacity(40);
    for _ in 0..40 {
        revision.push('a');
    }
    TestRunEvidence {
        format: TEST_RUN_EVIDENCE_FORMAT.into(),
        format_version: 1,
        id: "run-1".into(),
        application: "example".into(),
        environment: "production".into(),
        contract_fingerprint: digest('1'),
        source_revision: revision,
        release: "0.298.0".into(),
        artifact: TestRunArtifact {
            id: "web-bundle".into(),
            fingerprint: digest('2'),
            size_bytes: 1024,
        },
        started_at: "2026-07-21T00:00:00.000Z".into(),
        completed_at: "2026-07-21T00:10:00.000Z".into(),
        valid_until: "2026-07-28T00:10:00.000Z".into(),
        runner: TestRunRunner {
            identity: "ci.runner-1".into(),
            authentication_evidence: retained('3'),
            isolation: TestRunIsolation::Ephemeral,
            invocation_fingerprint: digest('4'),
            environment_evidence: retained('5'),
            environment_fingerprint: digest('6'),
        },
        selection: TestRunSelection {
            target_ids: BTreeSet::from(["web".into()]),
            suite_ids: BTreeSet::from(["unit".into()]),
        },
        targets: vec![TestRunTargetExecution {
            id: "web".into(),
            kind: TestRunTargetKind::Web,
            environment: "production".into(),
        }],
        suites: vec![TestRunSuiteExecution {
            id: "unit".into(),
            target_id: "web".into(),
            kind: TestRunSuiteKind::Unit,
            shard_index: 1,
            shard_count: 1,
            outcome: TestRunSuiteOutcome::Passed,
            passed: 12,
            failed: 0,
            skipped: 0,
            retries: 0,
            duration_ms: 61_000,
            invocation_fingerprint: digest('7'),
            report: retained('8'),
            log: retained('9'),
        }],
        verification: TestRunVerification {
            verifier: "release.verifier".into(),
            completed_at: "2026-07-21T00:11:00.000Z".into(),
            outcome: TestRunVerificationOutcome::Accepted,
            target_count: 1,
            suite_count: 1,
            passed: 12,
            failed: 0,
            skipped: 0,
            retries: 0,
            evidence: retained('d'),
        },
    }
}

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vize-marquette"))
}

fn write_example(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(name);
    std::fs::write(&path, serde_json::to_vec(&example_evidence()).unwrap()).unwrap();
    path
}

#[test]
fn schema_commands_print_the_embedded_schemas() {
    let output = cli().args(["test-run", "schema"]).output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        vize_marquette::TEST_RUN_EVIDENCE_JSON_SCHEMA
    );

    let output = cli().arg("schema").output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        vize_marquette::APPLICATION_CONTRACT_JSON_SCHEMA
    );
}

#[test]
fn test_run_validate_reports_diagnostics_and_exit_codes() {
    let path = write_example("vize-marquette-cli-valid.json");
    let output = cli()
        .args(["test-run", "validate"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(std::str::from_utf8(&output.stdout).unwrap().trim(), "[]");

    let mut broken = example_evidence();
    broken.verification.passed = 1;
    let broken_path = std::env::temp_dir().join("vize-marquette-cli-broken.json");
    std::fs::write(&broken_path, serde_json::to_vec(&broken).unwrap()).unwrap();
    let output = cli()
        .args(["test-run", "validate"])
        .arg(&broken_path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        std::str::from_utf8(&output.stdout)
            .unwrap()
            .contains("VIZE_MARQUETTE_128")
    );
}

#[test]
fn test_run_fingerprint_matches_the_library() {
    let path = write_example("vize-marquette-cli-fingerprint.json");
    let output = cli()
        .args(["test-run", "fingerprint"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap().trim(),
        test_run_fingerprint(&example_evidence()).unwrap().as_str()
    );
}

#[test]
fn test_run_canonical_streams_sorted_bytes() {
    let path = write_example("vize-marquette-cli-canonical.json");
    let output = cli()
        .args(["test-run", "canonical"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        vize_marquette::canonical_test_run_json(&example_evidence()).unwrap()
    );
}
