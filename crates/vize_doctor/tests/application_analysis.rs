#![cfg(feature = "application-analysis")]

use vize_croquis_cf::{
    CrossFileAnalyzer, CrossFileDiagnostic, CrossFileDiagnosticKind, CrossFileOptions,
    CrossFileResult, DiagnosticSeverity, FileId,
};
use vize_doctor::{
    DoctorCategory, FindingConfidence, FindingImpact, FindingSeverity, FixSafety,
    application_analysis::{findings_from_application_graph, report_from_application_graph},
};

fn analyzer_with_files() -> (CrossFileAnalyzer, FileId, FileId) {
    let mut analyzer =
        CrossFileAnalyzer::with_project_root(CrossFileOptions::minimal(), "/workspace");
    let app = analyzer.add_file("src/App.ts", "export const app = 1");
    let state = analyzer.add_file("src/state.ts", "export const state = 1");
    (analyzer, app, state)
}

#[test]
fn preserves_graph_sources_evidence_fixes_and_invalidation() {
    let (analyzer, app, state) = analyzer_with_files();
    let diagnostic = CrossFileDiagnostic::with_span(
        CrossFileDiagnosticKind::CircularDependency {
            cycle: vec!["App".into(), "state".into()],
        },
        DiagnosticSeverity::Error,
        app,
        7,
        15,
        "Application modules form a cycle",
    )
    .with_related(state, 4, "Cycle returns through this module")
    .with_suggestion("Move shared state behind an acyclic boundary");
    let result = CrossFileResult {
        diagnostics: vec![diagnostic],
        ..CrossFileResult::default()
    };

    let report = report_from_application_graph("example", &analyzer, &result).unwrap();
    let finding = &report.findings()[0];

    assert_eq!(finding.code, "VIZE_DOCTOR_CF_CIRCULAR_DEP");
    assert_eq!(finding.category, DoctorCategory::Maintainability);
    assert_eq!(finding.assessment.severity, FindingSeverity::Error);
    assert_eq!(finding.assessment.confidence, FindingConfidence::Certain);
    assert_eq!(finding.assessment.impact, FindingImpact::High);
    assert_eq!(finding.primary.path, "src/App.ts");
    assert_eq!(finding.primary.start, 7);
    assert_eq!(finding.primary.end, 15);
    assert_eq!(finding.related[0].location.path, "src/state.ts");
    assert_eq!(
        finding.evidence[0].details["sourceDiagnostic"],
        "vize:croquis/cf/circular-dep"
    );
    assert_eq!(
        finding.provenance.invalidation_inputs,
        ["src/App.ts", "src/state.ts"]
    );
    assert_eq!(
        finding.fix.as_ref().unwrap().safety,
        FixSafety::ReviewRequired
    );
    assert_eq!(
        finding.fix.as_ref().unwrap().title,
        "Move shared state behind an acyclic boundary"
    );
    assert!(report.summary().has_blocking_errors);
}

#[test]
fn distinguishes_exact_review_suggestions_from_unavailable_fixes() {
    let (analyzer, app, _) = analyzer_with_files();
    let diagnostic = || {
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::UnregisteredComponent {
                component_name: "MissingCard".into(),
                template_offset: 4,
            },
            DiagnosticSeverity::Error,
            app,
            4,
            "Component cannot be resolved",
        )
    };
    let review_title = "  Register MissingCard in this component  ";
    let result = CrossFileResult {
        diagnostics: vec![
            diagnostic(),
            diagnostic().with_suggestion(" \t\n"),
            diagnostic().with_suggestion(review_title),
        ],
        ..CrossFileResult::default()
    };

    let findings = findings_from_application_graph(&analyzer, &result).unwrap();
    let unavailable_reason =
        "No automatic fix is available because the source diagnostic did not provide a suggestion.";

    for finding in &findings[..2] {
        let fix = finding.fix.as_ref().unwrap();
        assert_eq!(fix.safety, FixSafety::Unavailable);
        assert_eq!(fix.title, unavailable_reason);
        assert!(fix.edits.is_empty());
        assert!(fix.verification.is_empty());
        assert_eq!(
            serde_json::to_value(fix).unwrap(),
            serde_json::json!({
                "safety": "unavailable",
                "title": unavailable_reason,
            })
        );
    }

    let review = findings[2].fix.as_ref().unwrap();
    assert_eq!(review.safety, FixSafety::ReviewRequired);
    assert_eq!(review.title, review_title);
    assert!(review.edits.is_empty());
}

#[test]
fn classifies_every_health_family_without_inflating_confidence() {
    let (analyzer, app, _) = analyzer_with_files();
    let diagnostics = vec![
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::UnregisteredComponent {
                component_name: "MissingCard".into(),
                template_offset: 4,
            },
            DiagnosticSeverity::Error,
            app,
            4,
            "Component cannot be resolved",
        ),
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::DuplicateElementId {
                id: "email".into(),
                locations: vec![(app, 10)],
            },
            DiagnosticSeverity::Warning,
            app,
            10,
            "Element identifier is duplicated",
        ),
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::DeepImportChain {
                depth: 12,
                chain: vec!["App".into()],
            },
            DiagnosticSeverity::Warning,
            app,
            20,
            "Import chain exceeds the configured depth",
        ),
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::ReactiveStateExported {
                variable_name: "state".into(),
                export_type: "named".into(),
            },
            DiagnosticSeverity::Warning,
            app,
            30,
            "Mutable reactive state crosses its owner",
        ),
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::HydrationMismatchRisk {
                reason: "target-dependent output".into(),
            },
            DiagnosticSeverity::Warning,
            app,
            40,
            "Server and client output may diverge",
        ),
    ];
    let result = CrossFileResult {
        diagnostics,
        ..CrossFileResult::default()
    };
    let findings = findings_from_application_graph(&analyzer, &result).unwrap();

    assert_profile(
        &findings,
        "VIZE_DOCTOR_CF_UNREGISTERED_COMPONENT",
        DoctorCategory::Correctness,
        FindingConfidence::Certain,
    );
    assert_profile(
        &findings,
        "VIZE_DOCTOR_CF_DUPLICATE_ID",
        DoctorCategory::Accessibility,
        FindingConfidence::Certain,
    );
    assert_profile(
        &findings,
        "VIZE_DOCTOR_CF_DEEP_IMPORT",
        DoctorCategory::Performance,
        FindingConfidence::Medium,
    );
    assert_profile(
        &findings,
        "VIZE_DOCTOR_CF_REACTIVE_EXPORT",
        DoctorCategory::Maintainability,
        FindingConfidence::Certain,
    );
    assert_profile(
        &findings,
        "VIZE_DOCTOR_CF_HYDRATION_RISK",
        DoctorCategory::ProductionReadiness,
        FindingConfidence::Medium,
    );
}

#[test]
fn report_output_is_independent_of_source_diagnostic_order() {
    let (analyzer, app, _) = analyzer_with_files();
    let first = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::UnregisteredComponent {
            component_name: "MissingCard".into(),
            template_offset: 2,
        },
        DiagnosticSeverity::Error,
        app,
        2,
        "Component cannot be resolved",
    );
    let second = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::DeepImportChain {
            depth: 12,
            chain: vec!["App".into()],
        },
        DiagnosticSeverity::Warning,
        app,
        10,
        "Import chain exceeds the configured depth",
    );

    let forward = CrossFileResult {
        diagnostics: vec![first.clone(), second.clone()],
        ..CrossFileResult::default()
    };
    let reversed = CrossFileResult {
        diagnostics: vec![second, first],
        ..CrossFileResult::default()
    };

    assert_eq!(
        report_from_application_graph("example", &analyzer, &forward).unwrap(),
        report_from_application_graph("example", &analyzer, &reversed).unwrap()
    );
}

fn assert_profile(
    findings: &[vize_doctor::DoctorFinding],
    code: &str,
    category: DoctorCategory,
    confidence: FindingConfidence,
) {
    let finding = findings
        .iter()
        .find(|finding| finding.code == code)
        .unwrap();
    assert_eq!(finding.category, category);
    assert_eq!(finding.assessment.confidence, confidence);
}
