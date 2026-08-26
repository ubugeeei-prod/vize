use std::collections::BTreeSet;

use super::model::*;

pub(crate) fn filled(fill: char, length: usize) -> vize_s0::String {
    let mut output = vize_s0::String::with_capacity(length);
    for _ in 0..length {
        output.push(fill);
    }
    output
}

fn digest(fill: char) -> vize_s0::String {
    filled(fill, 64)
}

fn retained(fill: char) -> TestRunRetainedEvidence {
    let fingerprint = digest(fill);
    let mut reference = vize_s0::String::from("sha256:");
    reference.push_str(&fingerprint);
    TestRunRetainedEvidence::new(reference, fingerprint)
}

/// Returns a fully-populated record that satisfies every schema bound.
pub(crate) fn example_evidence() -> TestRunEvidence {
    TestRunEvidence {
        format: TEST_RUN_EVIDENCE_FORMAT.into(),
        format_version: TEST_RUN_EVIDENCE_FORMAT_VERSION,
        id: "run-2026-07-21.001".into(),
        application: "example".into(),
        environment: "production".into(),
        contract_fingerprint: digest('1'),
        source_revision: filled('a', 40),
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
            suite_ids: BTreeSet::from(["e2e".into(), "unit".into()]),
        },
        targets: vec![TestRunTargetExecution {
            id: "web".into(),
            kind: TestRunTargetKind::Web,
            environment: "production".into(),
        }],
        suites: vec![
            TestRunSuiteExecution {
                id: "unit".into(),
                target_id: "web".into(),
                kind: TestRunSuiteKind::Unit,
                shard_index: 1,
                shard_count: 1,
                outcome: TestRunSuiteOutcome::Passed,
                passed: 120,
                failed: 0,
                skipped: 0,
                retries: 0,
                duration_ms: 61_000,
                invocation_fingerprint: digest('7'),
                report: retained('8'),
                log: retained('9'),
            },
            TestRunSuiteExecution {
                id: "e2e".into(),
                target_id: "web".into(),
                kind: TestRunSuiteKind::EndToEnd,
                shard_index: 1,
                shard_count: 1,
                outcome: TestRunSuiteOutcome::Passed,
                passed: 24,
                failed: 0,
                skipped: 0,
                retries: 0,
                duration_ms: 180_000,
                invocation_fingerprint: digest('a'),
                report: retained('b'),
                log: retained('c'),
            },
        ],
        verification: TestRunVerification {
            verifier: "release.verifier".into(),
            completed_at: "2026-07-21T00:11:00.000Z".into(),
            outcome: TestRunVerificationOutcome::Accepted,
            target_count: 1,
            suite_count: 2,
            passed: 144,
            failed: 0,
            skipped: 0,
            retries: 0,
            evidence: retained('d'),
        },
    }
}

#[test]
fn serialization_round_trips_with_schema_field_names() {
    let evidence = example_evidence();
    let json = serde_json::to_value(&evidence).unwrap();

    assert_eq!(json["format"], TEST_RUN_EVIDENCE_FORMAT);
    assert_eq!(json["formatVersion"], 1);
    assert_eq!(json["artifact"]["sizeBytes"], 1024);
    assert_eq!(json["runner"]["isolation"], "ephemeral");
    assert_eq!(json["selection"]["targetIds"][0], "web");
    assert_eq!(json["suites"][1]["kind"], "end-to-end");
    assert_eq!(json["verification"]["outcome"], "accepted");

    let restored: TestRunEvidence = serde_json::from_value(json).unwrap();
    assert_eq!(restored, evidence);
}

#[test]
fn format_version_defaults_and_unknown_properties_are_rejected() {
    let mut json = serde_json::to_value(example_evidence()).unwrap();
    json.as_object_mut().unwrap().remove("formatVersion");
    let restored: TestRunEvidence = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(restored.format_version, TEST_RUN_EVIDENCE_FORMAT_VERSION);

    json.as_object_mut()
        .unwrap()
        .insert("unexpected".into(), serde_json::Value::Bool(true));
    assert!(serde_json::from_value::<TestRunEvidence>(json).is_err());

    let mut nested = serde_json::to_value(example_evidence()).unwrap();
    nested["runner"]
        .as_object_mut()
        .unwrap()
        .insert("unexpected".into(), serde_json::Value::Bool(true));
    assert!(serde_json::from_value::<TestRunEvidence>(nested).is_err());
}
