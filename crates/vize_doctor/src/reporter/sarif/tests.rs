use serde_json::Value;
use vize_s0::{ToCompactString, cstr};

use super::*;
use crate::{
    AnalysisProvenance, DoctorCategory, DoctorFinding, FindingAssessment, FindingConfidence,
    FindingContext, FindingEvidence, FindingFix, FindingImpact, FindingSeverity, FixSafety,
    HealthPenalty, RelatedLocation, ReporterErrorKind, ReporterFailure, RuleCost, SourceLocation,
    SuppressionPolicy, TextEdit, render_report,
};

const SOURCE: &str = "const greeting = \"東京🙂\";\nconst owner = greeting;\n";

fn finding() -> DoctorFinding {
    let start = SOURCE.find("東京🙂").unwrap() as u32;
    let end = start + "東京🙂".len() as u32;
    DoctorFinding::new(
        "VIZE_DOCTOR_I18N_001",
        DoctorCategory::Accessibility,
        FindingAssessment::new(
            FindingSeverity::Warning,
            FindingConfidence::High,
            FindingImpact::Medium,
            HealthPenalty::new(12, "Locale fallback can hide content"),
        ),
        SourceLocation::new("src/画面 #1.vue", start, end),
        "Localized name has no fallback",
        "Provide a stable authored fallback for the accessible name.",
        AnalysisProvenance::new("semantic-tree", RuleCost::Low)
            .with_invalidation_inputs(["src/画面 #1.vue"]),
    )
    .with_failure_scenario("The control becomes unnamed when the locale key is absent.")
    .with_documentation("/doctor/accessibility/localized-name")
    .with_related(RelatedLocation::new(
        SourceLocation::new(
            "src/画面 #1.vue",
            SOURCE.find("greeting").unwrap() as u32,
            SOURCE.find("greeting").unwrap() as u32 + "greeting".len() as u32,
        ),
        "Authored binding",
    ))
    .with_evidence(
        FindingEvidence::new(
            crate::EvidenceKind::Component,
            "Resolved accessible-name binding",
        )
        .with_location(SourceLocation::new("src/画面 #1.vue", 0, 5))
        .with_detail("binding", "greeting"),
    )
    .with_fix(
        FindingFix::new(FixSafety::Safe, "Add the fallback")
            .with_edit(TextEdit::new(
                SourceLocation::new("src/画面 #1.vue", start, end),
                "東京 (Tokyo)🙂",
            ))
            .with_verification("vize doctor --format sarif"),
    )
    .with_context(FindingContext {
        target: Some("web".into()),
        environment: Some("client".into()),
        component: Some("LocalizedControl".into()),
        ..FindingContext::default()
    })
    .with_suppression(SuppressionPolicy::ReasonRequired)
}

fn render_value(
    reporter: &SarifReporter<'_>,
    report: &crate::DoctorReport,
) -> Result<Value, ReporterFailure> {
    let mut bytes = Vec::new();
    render_report(reporter, report, &mut bytes)?;
    Ok(serde_json::from_slice(&bytes).unwrap())
}

#[test]
fn empty_log_declares_the_oasis_contract_and_vize_versions() {
    let report = crate::DoctorReport::new("workspace", []);
    let value = render_value(&SarifReporter::new(), &report).unwrap();

    assert_eq!(value["version"], "2.1.0");
    assert_eq!(
        value["$schema"],
        "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json"
    );
    assert_eq!(value["runs"][0]["tool"]["driver"]["name"], "Vize Doctor");
    assert_eq!(value["runs"][0]["columnKind"], "unicodeCodePoints");
    assert_eq!(value["runs"][0]["results"], serde_json::json!([]));
    assert_eq!(
        value["runs"][0]["properties"]["vizeReportFormatVersion"],
        crate::DOCTOR_REPORT_FORMAT_VERSION
    );
}

#[test]
fn result_preserves_unicode_regions_evidence_policy_and_text_fixes() {
    let report = crate::DoctorReport::new("workspace", [finding()]);
    let reporter = SarifReporter::new()
        .with_sources([SarifSource::new("src/画面 #1.vue", SOURCE)])
        .unwrap();
    let value = render_value(&reporter, &report).unwrap();
    let result = &value["runs"][0]["results"][0];
    let region = &result["locations"][0]["physicalLocation"]["region"];

    assert_eq!(result["ruleId"], "VIZE_DOCTOR_I18N_001");
    assert_eq!(result["ruleIndex"], 0);
    assert_eq!(result["level"], "warning");
    assert_eq!(
        result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
        "src/%E7%94%BB%E9%9D%A2%20%231.vue"
    );
    assert_eq!(region["startLine"], 1);
    assert_eq!(region["startColumn"], 19);
    assert_eq!(region["endLine"], 1);
    assert_eq!(region["endColumn"], 22);
    assert_eq!(result["relatedLocations"].as_array().unwrap().len(), 2);
    assert_eq!(
        result["relatedLocations"][1]["properties"]["vizeEvidenceDetails"]["binding"],
        "greeting"
    );
    let expected_baseline_key = cstr!(
        "VIZE_DOCTOR_I18N_001:src/画面 #1.vue:{}:{}",
        SOURCE.find("東京🙂").unwrap(),
        SOURCE.find("東京🙂").unwrap() + "東京🙂".len()
    );
    assert_eq!(
        result["partialFingerprints"]["vizeBaselineKey/v1"],
        expected_baseline_key.as_str()
    );
    assert_eq!(result["fixes"][0]["properties"]["vizeSafety"], "safe");
    assert_eq!(
        result["fixes"][0]["artifactChanges"][0]["replacements"][0]["insertedContent"]["text"],
        "東京 (Tokyo)🙂"
    );
    assert_eq!(
        result["properties"]["vizeProvenance"]["capability"],
        "semantic-tree"
    );
}

#[test]
fn missing_sources_fail_before_any_output_by_default() {
    let report = crate::DoctorReport::new("workspace", [finding()]);
    let mut bytes = Vec::new();
    let failure = render_report(&SarifReporter::new(), &report, &mut bytes).unwrap_err();

    assert!(bytes.is_empty());
    assert_eq!(failure.bytes_written(), 0);
    let ReporterFailure::Rendering { error, .. } = failure else {
        panic!("expected a rendering failure");
    };
    assert_eq!(error.kind(), ReporterErrorKind::InvalidData);
    assert!(error.message().contains("is required"));
}

#[test]
fn artifact_only_policy_marks_intentionally_imprecise_results() {
    let report = crate::DoctorReport::new("workspace", [finding()]);
    let reporter =
        SarifReporter::new().with_missing_source_policy(SarifMissingSourcePolicy::ArtifactOnly);
    let value = render_value(&reporter, &report).unwrap();
    let result = &value["runs"][0]["results"][0];

    assert!(
        result["locations"][0]["physicalLocation"]
            .get("region")
            .is_none()
    );
    assert_eq!(result["properties"]["vizeSourceRegionsOmitted"], 3);
    assert_eq!(result["properties"]["vizeFixEditsOmitted"], 1);
    assert!(result.get("fixes").is_none());
}

#[test]
fn stale_and_non_utf8_boundaries_are_rejected() {
    for location in [
        SourceLocation::new("src/画面 #1.vue", 0, SOURCE.len() as u32 + 1),
        SourceLocation::new("src/画面 #1.vue", 19, 20),
    ] {
        let mut invalid = finding();
        invalid.primary = location;
        let report = crate::DoctorReport::new("workspace", [invalid]);
        let reporter = SarifReporter::new()
            .with_sources([SarifSource::new("src/画面 #1.vue", SOURCE)])
            .unwrap();
        let error = render_value(&reporter, &report).unwrap_err();
        assert_eq!(error.bytes_written(), 0);
    }
}

#[test]
fn source_configuration_rejects_duplicates_and_non_normalized_paths() {
    let duplicate = SarifReporter::new()
        .with_sources([
            SarifSource::new("src/App.vue", "a"),
            SarifSource::new("src/App.vue", "a"),
        ])
        .err()
        .unwrap();
    let parent = SarifReporter::new()
        .with_sources([SarifSource::new("src/../App.vue", "a")])
        .err()
        .unwrap();

    assert_eq!(duplicate.path(), "src/App.vue");
    assert!(duplicate.reason().contains("more than once"));
    assert!(parent.reason().contains("normalized"));
}

#[test]
fn output_is_deterministic_across_source_order_and_pretty_mode() {
    let mut second = finding();
    second.primary.path = "src/Other.vue".into();
    second.code = "VIZE_DOCTOR_I18N_002".into();
    second.fix = Some(FindingFix::unavailable("No deterministic edit"));
    let report = crate::DoctorReport::new("workspace", [second, finding()]);
    let sources = [
        SarifSource::new("src/画面 #1.vue", SOURCE),
        SarifSource::new("src/Other.vue", SOURCE),
    ];
    let first = SarifReporter::new()
        .with_pretty(false)
        .with_sources(sources)
        .unwrap();
    let second = SarifReporter::new()
        .with_pretty(false)
        .with_sources(sources.into_iter().rev())
        .unwrap();
    let mut first_bytes = Vec::new();
    let mut second_bytes = Vec::new();

    render_report(&first, &report, &mut first_bytes).unwrap();
    render_report(&second, &report, &mut second_bytes).unwrap();

    assert_eq!(first_bytes, second_bytes);
    assert_eq!(first_bytes.last(), Some(&b'\n'));
}

#[test]
fn overlapping_fix_edits_fail_closed() {
    let mut invalid = finding();
    let fix = invalid.fix.as_mut().unwrap();
    fix.edits.push(TextEdit::new(
        SourceLocation::new(
            "src/画面 #1.vue",
            fix.edits[0].location.start,
            fix.edits[0].location.end,
        ),
        "duplicate",
    ));
    let report = crate::DoctorReport::new("workspace", [invalid]);
    let reporter = SarifReporter::new()
        .with_sources([SarifSource::new("src/画面 #1.vue", SOURCE)])
        .unwrap();
    let error = render_value(&reporter, &report).unwrap_err();

    assert_eq!(error.bytes_written(), 0);
    assert!(error.to_compact_string().contains("overlapping edits"));
}
