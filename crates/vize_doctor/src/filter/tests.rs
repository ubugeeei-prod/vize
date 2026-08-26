use crate::{
    AnalysisProvenance, DoctorCategory, DoctorFinding, DoctorReport, FindingAssessment,
    FindingConfidence, FindingContext, FindingEvidence, FindingFix, FindingImpact, FindingSeverity,
    FixSafety, HealthPenalty, RelatedLocation, RuleCost, SourceLocation, TextEdit,
};
use vize_s0::{String, cstr};

use super::{DoctorFilterDimension, DoctorFilterSpec};

#[test]
fn empty_filter_accepts_every_finding_and_preserves_report() {
    let report = fixture_report();
    let filter = DoctorFilterSpec::default().compile().unwrap();

    assert!(
        report
            .findings()
            .iter()
            .all(|finding| filter.matches(finding))
    );
    assert_eq!(filter.apply(&report), report);
}

#[test]
fn dimensions_are_anded_and_values_within_dimensions_are_ored() {
    let report = fixture_report();
    let filter = DoctorFilterSpec {
        categories: vec![DoctorCategory::Security, DoctorCategory::Correctness],
        severities: vec![FindingSeverity::Error],
        confidences: vec![FindingConfidence::Certain, FindingConfidence::High],
        targets: vec!["w*".into()],
        rules: vec!["VIZE_DOCTOR_*_001".into()],
        paths: vec!["packages/**/src/*.vue".into()],
        routes: vec!["/account/**".into()],
        environments: vec!["prod*".into()],
        packages: vec!["@vize/app-*".into()],
        changed_files: vec!["packages/account/**".into()],
    }
    .compile()
    .unwrap();

    let selected = filter.apply(&report);

    assert_eq!(selected.findings().len(), 1);
    assert_eq!(selected.findings()[0].code, "VIZE_DOCTOR_SECURITY_001");
    assert_eq!(selected.summary().counts.errors, 1);
    assert!(selected.summary().has_blocking_errors);
}

#[test]
fn populated_context_dimension_rejects_absent_context() {
    let report = fixture_report();
    let filter = DoctorFilterSpec {
        targets: vec!["web".into()],
        ..DoctorFilterSpec::default()
    }
    .compile()
    .unwrap();

    assert!(filter.matches(&report.findings()[0]));
    assert!(!filter.matches(&report.findings()[1]));
}

#[test]
fn path_is_primary_only_while_changed_file_uses_complete_evidence_graph() {
    let report = fixture_report();
    let finding = &report.findings()[0];
    let primary_only = DoctorFilterSpec {
        paths: vec!["packages/shared/src/policy.ts".into()],
        ..DoctorFilterSpec::default()
    }
    .compile()
    .unwrap();
    let related_changed = DoctorFilterSpec {
        changed_files: vec!["packages/shared/src/policy.ts".into()],
        ..DoctorFilterSpec::default()
    }
    .compile()
    .unwrap();
    let edit_changed = DoctorFilterSpec {
        changed_files: vec!["packages/account/src/fix.ts".into()],
        ..DoctorFilterSpec::default()
    }
    .compile()
    .unwrap();
    let invalidation_changed = DoctorFilterSpec {
        changed_files: vec!["packages/account/src/generated.ts".into()],
        ..DoctorFilterSpec::default()
    }
    .compile()
    .unwrap();

    assert!(!primary_only.matches(finding));
    assert!(related_changed.matches(finding));
    assert!(edit_changed.matches(finding));
    assert!(invalidation_changed.matches(finding));
}

#[test]
fn windows_patterns_and_candidate_paths_are_slash_normalized() {
    let report = fixture_report();
    let filter = DoctorFilterSpec {
        paths: vec![r"packages\account\src\*.vue".into()],
        changed_files: vec![r"packages\shared\**".into()],
        ..DoctorFilterSpec::default()
    }
    .compile()
    .unwrap();

    assert!(filter.matches(&report.findings()[0]));
    assert_eq!(
        filter.spec().paths,
        [String::from("packages/account/src/*.vue")]
    );
}

#[test]
fn compiling_normalizes_deduplicates_and_serializes_the_spec() {
    let filter = DoctorFilterSpec {
        categories: vec![DoctorCategory::Security, DoctorCategory::Security],
        rules: vec![" VIZE_* ".into(), "VIZE_*".into()],
        ..DoctorFilterSpec::default()
    }
    .compile()
    .unwrap();

    assert_eq!(filter.spec().categories, [DoctorCategory::Security]);
    assert_eq!(filter.spec().rules, [String::from("VIZE_*")]);
    assert_eq!(
        serde_json::to_string(filter.spec()).unwrap(),
        r#"{"categories":["security"],"rules":["VIZE_*"]}"#
    );
}

#[test]
fn invalid_and_empty_patterns_fail_closed_with_dimension_context() {
    for (pattern, expected_reason) in [("[broken", "unclosed"), ("   ", "must not be empty")] {
        let error = DoctorFilterSpec {
            changed_files: vec![pattern.into()],
            ..DoctorFilterSpec::default()
        }
        .compile()
        .unwrap_err();

        assert_eq!(error.dimension(), DoctorFilterDimension::ChangedFile);
        assert!(error.reason().contains(expected_reason), "{error}");
        assert!(cstr!("{error}").contains("changed-file"));
    }
}

#[test]
fn filtered_reports_recompute_health_and_blocking_from_visible_findings() {
    let report = fixture_report();
    let filter = DoctorFilterSpec {
        severities: vec![FindingSeverity::Warning],
        ..DoctorFilterSpec::default()
    }
    .compile()
    .unwrap();

    let selected = filter.apply(&report);

    assert_eq!(selected.summary().counts.total(), 1);
    assert_eq!(selected.summary().counts.warnings, 1);
    assert!(!selected.summary().has_blocking_errors);
    assert!(selected.summary().overall_score > report.summary().overall_score);
}

fn fixture_report() -> DoctorReport {
    let security = DoctorFinding::new(
        "VIZE_DOCTOR_SECURITY_001",
        DoctorCategory::Security,
        assessment(
            FindingSeverity::Error,
            FindingConfidence::Certain,
            FindingImpact::Critical,
            30,
        ),
        SourceLocation::new("packages/account/src/Login.vue", 10, 20),
        "Authorization guard is missing",
        "Guard the reachable account route before loading private data.",
        AnalysisProvenance::new("authorization-graph", RuleCost::Moderate)
            .with_invalidation_inputs(["packages/account/src/generated.ts"]),
    )
    .with_context(FindingContext {
        target: Some("web".into()),
        environment: Some("production".into()),
        route: Some("/account/settings".into()),
        package: Some("@vize/app-account".into()),
        component: Some("Login".into()),
        capability: Some("account.read".into()),
        build_node: Some("account-client".into()),
    })
    .with_related(RelatedLocation::new(
        SourceLocation::new("packages/shared/src/policy.ts", 3, 8),
        "Policy declaration",
    ))
    .with_evidence(
        FindingEvidence::new(crate::EvidenceKind::Contract, "Guard edge is absent").with_location(
            SourceLocation::new("packages/account/src/router.ts", 30, 40),
        ),
    )
    .with_fix(
        FindingFix::new(FixSafety::ReviewRequired, "Add the account guard").with_edit(
            TextEdit::new(
                SourceLocation::new("packages/account/src/fix.ts", 0, 0),
                "requireAccount()",
            ),
        ),
    );

    let warning = DoctorFinding::new(
        "VIZE_DOCTOR_PERFORMANCE_002",
        DoctorCategory::Performance,
        assessment(
            FindingSeverity::Warning,
            FindingConfidence::Medium,
            FindingImpact::Medium,
            12,
        ),
        SourceLocation::new("src/Home.vue", 4, 12),
        "Avoidable render work",
        "Move the stable work out of the render path.",
        AnalysisProvenance::new("render-graph", RuleCost::Low),
    );

    DoctorReport::new("fixture", [warning, security])
}

fn assessment(
    severity: FindingSeverity,
    confidence: FindingConfidence,
    impact: FindingImpact,
    points: u8,
) -> FindingAssessment {
    FindingAssessment::new(
        severity,
        confidence,
        impact,
        HealthPenalty::new(points, "Filter fixture"),
    )
}
