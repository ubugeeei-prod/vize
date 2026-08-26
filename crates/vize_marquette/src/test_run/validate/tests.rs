use crate::test_run::model_tests::{example_evidence, filled};
use crate::{TestRunEvidence, TestRunSuiteOutcome, TestRunVerificationOutcome, validate_test_run};

fn codes(evidence: &TestRunEvidence) -> Vec<&'static str> {
    validate_test_run(evidence)
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

#[test]
fn a_complete_record_validates_cleanly() {
    assert_eq!(validate_test_run(&example_evidence()), Vec::new());
}

#[test]
fn format_marker_and_version_are_pinned() {
    let mut evidence = example_evidence();
    evidence.format = "vize.other.evidence".into();
    evidence.format_version = 2;
    assert_eq!(
        codes(&evidence),
        ["VIZE_MARQUETTE_101", "VIZE_MARQUETTE_102"]
    );
}

#[test]
fn bound_facts_must_use_exact_grammars() {
    let mut evidence = example_evidence();
    evidence.application = "Mixed.Case".into();
    evidence.contract_fingerprint = filled('x', 64);
    evidence.source_revision = filled('a', 12);
    evidence.release = "".into();
    evidence.artifact.size_bytes = 0;
    evidence.started_at = "2026-07-21T00:00:00Z".into();

    let reported = codes(&evidence);
    for expected in [
        "VIZE_MARQUETTE_103",
        "VIZE_MARQUETTE_104",
        "VIZE_MARQUETTE_105",
        "VIZE_MARQUETTE_106",
        "VIZE_MARQUETTE_110",
        "VIZE_MARQUETTE_107",
    ] {
        assert!(reported.contains(&expected), "{expected} in {reported:?}");
    }
}

#[test]
fn record_times_must_be_ordered() {
    let mut evidence = example_evidence();
    evidence.completed_at = "2026-07-20T00:00:00.000Z".into();
    let reported = codes(&evidence);
    assert!(reported.contains(&"VIZE_MARQUETTE_112"), "{reported:?}");

    let mut expired = example_evidence();
    expired.valid_until = expired.completed_at.clone();
    assert_eq!(codes(&expired), ["VIZE_MARQUETTE_113"]);

    let mut early_verification = example_evidence();
    early_verification.verification.completed_at = "2026-07-21T00:09:00.000Z".into();
    assert_eq!(codes(&early_verification), ["VIZE_MARQUETTE_114"]);
}

#[test]
fn mutable_evidence_references_are_rejected() {
    let mut unbound = example_evidence();
    unbound.runner.authentication_evidence.reference = {
        let mut reference = vize_s0::String::from("sha256:");
        reference.push_str(&filled('0', 64));
        reference
    };
    assert_eq!(codes(&unbound), ["VIZE_MARQUETTE_109"]);

    let mut mutable = example_evidence();
    mutable.suites[0].report.reference = "https://reports.example/latest".into();
    assert_eq!(codes(&mutable), ["VIZE_MARQUETTE_108"]);
}

#[test]
fn coverage_must_match_the_candidate_selection_exactly() {
    let mut undeclared = example_evidence();
    undeclared.selection.suite_ids.remove("e2e");
    assert_eq!(codes(&undeclared), ["VIZE_MARQUETTE_121"]);

    let mut omitted = example_evidence();
    omitted.suites.pop();
    omitted.verification.suite_count = 1;
    omitted.verification.passed = 120;
    assert_eq!(codes(&omitted), ["VIZE_MARQUETTE_122"]);

    let mut unknown_target = example_evidence();
    unknown_target.suites[0].target_id = "missing".into();
    assert_eq!(codes(&unknown_target), ["VIZE_MARQUETTE_123"]);
}

#[test]
fn duplicate_executions_are_rejected() {
    let mut duplicate_target = example_evidence();
    let target = duplicate_target.targets[0].clone();
    duplicate_target.targets.push(target);
    duplicate_target.verification.target_count = 2;
    assert_eq!(codes(&duplicate_target), ["VIZE_MARQUETTE_117"]);

    let mut duplicate_shard = example_evidence();
    let shard = duplicate_shard.suites[0].clone();
    duplicate_shard.suites.push(shard);
    duplicate_shard.verification.suite_count = 3;
    duplicate_shard.verification.passed += 120;
    assert_eq!(codes(&duplicate_shard), ["VIZE_MARQUETTE_120"]);
}

#[test]
fn shard_records_must_be_complete_and_consistent() {
    let mut sharded = example_evidence();
    sharded.suites[0].shard_count = 2;
    sharded.verification.suite_count = 2;
    assert_eq!(codes(&sharded), ["VIZE_MARQUETTE_125"]);

    let mut second = example_evidence();
    let mut shard = second.suites[0].clone();
    shard.shard_index = 2;
    shard.shard_count = 2;
    second.suites[0].shard_count = 2;
    second.suites.push(shard.clone());
    second.verification.suite_count = 3;
    second.verification.passed += shard.passed;
    assert_eq!(codes(&second), Vec::<&str>::new());

    let mut disagreeing = second.clone();
    disagreeing.suites[2].kind = crate::TestRunSuiteKind::Integration;
    assert_eq!(codes(&disagreeing), ["VIZE_MARQUETTE_126"]);

    let mut out_of_range = example_evidence();
    out_of_range.suites[0].shard_index = 2;
    assert_eq!(codes(&out_of_range), ["VIZE_MARQUETTE_124"]);
}

#[test]
fn hidden_retries_and_totals_are_rejected() {
    let mut hidden = example_evidence();
    hidden.suites[0].retries = 3;
    assert_eq!(codes(&hidden), ["VIZE_MARQUETTE_128"]);

    let mut declared = example_evidence();
    declared.suites[0].retries = 3;
    declared.verification.retries = 3;
    assert_eq!(codes(&declared), Vec::<&str>::new());

    let mut counts = example_evidence();
    counts.verification.passed = 1;
    assert_eq!(codes(&counts), ["VIZE_MARQUETTE_128"]);

    let mut sizes = example_evidence();
    sizes.verification.target_count = 3;
    assert_eq!(codes(&sizes), ["VIZE_MARQUETTE_127"]);
}

#[test]
fn acceptance_requires_a_fully_passing_run() {
    let mut failed = example_evidence();
    failed.suites[0].outcome = TestRunSuiteOutcome::Failed;
    failed.suites[0].failed = 2;
    failed.suites[0].passed = 118;
    failed.verification.failed = 2;
    failed.verification.passed = 142;
    assert_eq!(codes(&failed), ["VIZE_MARQUETTE_129"]);

    failed.verification.outcome = TestRunVerificationOutcome::Rejected;
    assert_eq!(codes(&failed), Vec::<&str>::new());

    let mut lying = example_evidence();
    lying.suites[0].failed = 1;
    lying.verification.failed = 1;
    let reported = codes(&lying);
    assert!(reported.contains(&"VIZE_MARQUETTE_129"), "{reported:?}");
    assert!(reported.contains(&"VIZE_MARQUETTE_130"), "{reported:?}");

    let mut empty = example_evidence();
    empty.suites[0].passed = 0;
    empty.verification.passed = 24;
    assert_eq!(codes(&empty), ["VIZE_MARQUETTE_130"]);
}

#[test]
fn impossible_calendar_dates_are_rejected() {
    let mut evidence = example_evidence();
    evidence.started_at = "2026-02-30T00:00:00.000Z".into();
    assert_eq!(codes(&evidence), ["VIZE_MARQUETTE_107"]);
}

#[test]
fn suite_id_scoped_diagnostics_appear_once_across_shards() {
    let mut evidence = example_evidence();
    let mut shard = evidence.suites[0].clone();
    shard.shard_index = 2;
    shard.shard_count = 2;
    let added_passed = shard.passed;
    evidence.suites[0].shard_count = 2;
    evidence.suites.push(shard);
    evidence.verification.suite_count += 1;
    evidence.verification.passed += added_passed;

    let suite_id = evidence.suites[0].id.clone();
    evidence.selection.suite_ids.remove(suite_id.as_str());

    // The membership error surfaces once for the suite, not once per shard.
    assert_eq!(codes(&evidence), ["VIZE_MARQUETTE_121"]);
}

#[test]
fn diagnostics_are_sorted_and_stable() {
    let mut evidence = example_evidence();
    evidence.format = "zzz".into();
    evidence.application = "Broken".into();
    let diagnostics = validate_test_run(&evidence);
    let mut sorted = diagnostics.clone();
    sorted.sort_by(|left, right| {
        (&left.path, left.code, &left.message).cmp(&(&right.path, right.code, &right.message))
    });
    assert_eq!(diagnostics, sorted);
}
