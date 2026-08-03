use vize_doctor::{
    AnalysisProvenance, DEFAULT_UNAVAILABLE_FIX_REASON, DOCTOR_REPORT_FORMAT_VERSION,
    DOCTOR_SCORING_VERSION, DoctorCategory, DoctorFinding, DoctorReport, EvidenceKind,
    FindingAssessment, FindingConfidence, FindingEvidence, FindingImpact, FindingSeverity,
    FixSafety, HealthPenalty, RuleCost, SourceLocation,
};

fn finding(
    code: &str,
    category: DoctorCategory,
    severity: FindingSeverity,
    confidence: FindingConfidence,
    impact: FindingImpact,
    path: &str,
    penalty: u8,
) -> DoctorFinding {
    DoctorFinding::new(
        code,
        category,
        FindingAssessment::new(
            severity,
            confidence,
            impact,
            HealthPenalty::new(penalty, "Test penalty"),
        ),
        SourceLocation::new(path, 10, 20),
        "Test finding",
        "Test message",
        AnalysisProvenance::new("test-analysis", RuleCost::Low),
    )
}

#[test]
fn ranks_findings_independently_of_input_order() {
    let error = finding(
        "VIZE_DOCTOR_001",
        DoctorCategory::Correctness,
        FindingSeverity::Error,
        FindingConfidence::Certain,
        FindingImpact::High,
        "src/a.vue",
        30,
    );
    let warning = finding(
        "VIZE_DOCTOR_002",
        DoctorCategory::Performance,
        FindingSeverity::Warning,
        FindingConfidence::High,
        FindingImpact::Medium,
        "src/b.vue",
        10,
    );

    let first = DoctorReport::new("app", [warning.clone(), error.clone()]);
    let second = DoctorReport::new("app", [error, warning]);

    assert_eq!(first, second);
    assert_eq!(first.findings()[0].code, "VIZE_DOCTOR_001");
}

#[test]
fn normalizes_nested_evidence_and_invalidation_order() {
    let mut forward = finding(
        "VIZE_DOCTOR_001",
        DoctorCategory::Correctness,
        FindingSeverity::Warning,
        FindingConfidence::High,
        FindingImpact::Medium,
        "src/a.vue",
        10,
    );
    forward.evidence = vec![
        FindingEvidence::new(EvidenceKind::Type, "second"),
        FindingEvidence::new(EvidenceKind::Source, "first"),
    ];
    forward.provenance.invalidation_inputs =
        vec!["src/b.ts".into(), "src/a.ts".into(), "src/b.ts".into()];
    let mut reversed = forward.clone();
    reversed.evidence.reverse();
    reversed.provenance.invalidation_inputs.reverse();

    assert_eq!(
        DoctorReport::new("app", [forward]),
        DoctorReport::new("app", [reversed])
    );
}

#[test]
fn scores_categories_and_preserves_blocking_errors() {
    let report = DoctorReport::new(
        "app",
        [
            finding(
                "VIZE_DOCTOR_001",
                DoctorCategory::Correctness,
                FindingSeverity::Error,
                FindingConfidence::Certain,
                FindingImpact::Critical,
                "src/a.vue",
                35,
            ),
            finding(
                "VIZE_DOCTOR_002",
                DoctorCategory::Correctness,
                FindingSeverity::Warning,
                FindingConfidence::High,
                FindingImpact::Medium,
                "src/b.vue",
                15,
            ),
            finding(
                "VIZE_DOCTOR_003",
                DoctorCategory::Performance,
                FindingSeverity::Notice,
                FindingConfidence::Medium,
                FindingImpact::Low,
                "src/c.vue",
                10,
            ),
        ],
    );

    assert_eq!(report.summary().counts.total(), 3);
    assert_eq!(report.summary().counts.errors, 1);
    assert!(report.summary().has_blocking_errors);
    assert_eq!(
        report.summary().categories[&DoctorCategory::Correctness].score,
        50
    );
    assert_eq!(
        report.summary().categories[&DoctorCategory::Performance].score,
        90
    );
    assert_eq!(report.summary().overall_score, 90);
}

#[test]
fn low_confidence_errors_never_block_by_default() {
    let report = DoctorReport::new(
        "app",
        [finding(
            "VIZE_DOCTOR_001",
            DoctorCategory::Maintainability,
            FindingSeverity::Error,
            FindingConfidence::Low,
            FindingImpact::Low,
            "src/a.vue",
            5,
        )],
    );

    assert_eq!(report.summary().counts.errors, 1);
    assert!(!report.summary().has_blocking_errors);
}

#[test]
fn serializes_language_neutral_versioned_properties() {
    let report = DoctorReport::new(
        "app",
        [finding(
            "VIZE_DOCTOR_001",
            DoctorCategory::ProductionReadiness,
            FindingSeverity::Warning,
            FindingConfidence::High,
            FindingImpact::Medium,
            "src/server.ts",
            12,
        )],
    );
    let value = serde_json::to_value(report).unwrap();

    assert_eq!(value["formatVersion"], DOCTOR_REPORT_FORMAT_VERSION);
    assert_eq!(value["scoringVersion"], DOCTOR_SCORING_VERSION);
    assert_eq!(value["workspace"], "app");
    assert_eq!(value["findings"][0]["category"], "production-readiness");
    assert_eq!(value["findings"][0]["provenance"]["cost"], "low");
    assert_eq!(value["findings"][0]["fix"]["safety"], "unavailable");
    assert_eq!(
        value["findings"][0]["fix"]["title"],
        DEFAULT_UNAVAILABLE_FIX_REASON
    );
    assert_eq!(value["summary"]["hasBlockingErrors"], false);
}

#[test]
fn normalizes_legacy_missing_fix_to_explicit_unavailable() {
    let report = DoctorReport::new(
        "app",
        [finding(
            "VIZE_DOCTOR_001",
            DoctorCategory::Correctness,
            FindingSeverity::Warning,
            FindingConfidence::High,
            FindingImpact::Medium,
            "src/a.vue",
            10,
        )],
    );
    let mut legacy_wire = serde_json::to_value(report).unwrap();
    legacy_wire["findings"][0]
        .as_object_mut()
        .unwrap()
        .remove("fix");

    let report = serde_json::from_value::<DoctorReport>(legacy_wire).unwrap();
    let fix = report.findings()[0].fix.as_ref().unwrap();

    assert_eq!(fix.safety, FixSafety::Unavailable);
    assert_eq!(fix.title, DEFAULT_UNAVAILABLE_FIX_REASON);
    assert!(fix.edits.is_empty());
    assert!(fix.verification.is_empty());
}

#[test]
fn validates_versions_and_derived_summary_when_deserializing() {
    let report = DoctorReport::new(
        "app",
        [finding(
            "VIZE_DOCTOR_001",
            DoctorCategory::Security,
            FindingSeverity::Warning,
            FindingConfidence::High,
            FindingImpact::High,
            "src/server.ts",
            20,
        )],
    );
    let value = serde_json::to_value(&report).unwrap();
    assert_eq!(
        serde_json::from_value::<DoctorReport>(value.clone()).unwrap(),
        report
    );

    let mut wrong_version = value.clone();
    wrong_version["formatVersion"] = 2.into();
    assert!(serde_json::from_value::<DoctorReport>(wrong_version).is_err());

    let mut wrong_summary = value;
    wrong_summary["summary"]["overallScore"] = 100.into();
    assert!(serde_json::from_value::<DoctorReport>(wrong_summary).is_err());
}
