use std::collections::BTreeMap;

use super::normalize_findings;
use crate::{
    AnalysisProvenance, ContentFingerprint, DoctorCategory, DoctorFinding, EvidenceKind,
    FindingAssessment, FindingConfidence, FindingEvidence, FindingFix, FindingImpact,
    FindingSeverity, FixSafety, HealthPenalty, RelatedLocation, RuleCost, SourceLocation, TextEdit,
};

/// Builds findings that exercise every normalization step at once.
fn unnormalized_findings() -> Vec<DoctorFinding> {
    let mut warning = DoctorFinding::new(
        "VIZE_B",
        DoctorCategory::Correctness,
        FindingAssessment::new(
            FindingSeverity::Warning,
            FindingConfidence::Certain,
            FindingImpact::Medium,
            HealthPenalty::new(10, "test"),
        ),
        SourceLocation::new("src/B.vue", 0, 1),
        "Second finding",
        "Second message",
        AnalysisProvenance::new("template-semantics", RuleCost::Low),
    );
    warning.related = vec![
        RelatedLocation::new(SourceLocation::new("src/Z.vue", 4, 8), "later"),
        RelatedLocation::new(SourceLocation::new("src/A.vue", 0, 2), "earlier"),
    ];
    warning.evidence = vec![
        FindingEvidence::new(EvidenceKind::Type, "type evidence"),
        FindingEvidence::new(EvidenceKind::Reactivity, "reactivity evidence"),
    ];
    warning.provenance.invalidation_inputs =
        vec!["src/Z.vue".into(), "src/A.vue".into(), "src/A.vue".into()];
    warning.provenance.invalidation_fingerprints = BTreeMap::from([
        ("src/A.vue".into(), ContentFingerprint::digest("source-a")),
        // Orphan fingerprint for an input the finding never declares.
        ("src/Q.vue".into(), ContentFingerprint::digest("source-q")),
    ]);
    // Missing fix must be replaced by the explicit unavailable disposition.
    warning.fix = None;

    let mut error = DoctorFinding::new(
        "VIZE_A",
        DoctorCategory::Correctness,
        FindingAssessment::new(
            FindingSeverity::Error,
            FindingConfidence::Certain,
            FindingImpact::High,
            HealthPenalty::new(30, "test"),
        ),
        SourceLocation::new("src/A.vue", 0, 1),
        "First finding",
        "First message",
        AnalysisProvenance::new("template-semantics", RuleCost::Low),
    );
    error.fix = Some(
        FindingFix::new(FixSafety::Safe, "Apply fix")
            .with_edit(TextEdit::new(
                SourceLocation::new("src/Z.vue", 10, 12),
                "later",
            ))
            .with_edit(TextEdit::new(
                SourceLocation::new("src/A.vue", 0, 2),
                "earlier",
            ))
            .with_verification("vize check")
            .with_verification("vize check"),
    );

    vec![warning, error]
}

#[test]
fn normalizing_already_normalized_findings_is_idempotent() {
    let once = normalize_findings(unnormalized_findings());
    let twice = normalize_findings(once.clone());

    // Guards the capability snapshot output fingerprint: a snapshot normalizes
    // on construction and again on deserialization, so a second pass must not
    // change a single byte.
    assert_eq!(twice, once);
    assert_eq!(
        serde_json::to_string(&twice).unwrap(),
        serde_json::to_string(&once).unwrap()
    );

    // The first pass really did have work to do.
    assert_ne!(once, unnormalized_findings());
    assert_eq!(once[0].code, "VIZE_A");
    assert_eq!(once[1].code, "VIZE_B");
    assert_eq!(
        once[1].provenance.invalidation_inputs,
        ["src/A.vue", "src/Z.vue"]
    );
    assert_eq!(once[1].provenance.invalidation_fingerprints.len(), 1);
    assert_eq!(once[1].fix.as_ref().unwrap().safety, FixSafety::Unavailable);
    assert_eq!(once[0].fix.as_ref().unwrap().verification, ["vize check"]);
}
