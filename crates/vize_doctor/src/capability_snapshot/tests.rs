use std::collections::BTreeMap;

use serde_json::json;
use vize_s0::ToCompactString;

use super::{
    CapabilityCacheIdentity, CapabilitySnapshot, CapabilitySnapshotError, ContentFingerprint,
    DOCTOR_CAPABILITY_SNAPSHOT_FORMAT_VERSION,
};
use crate::{
    AnalysisProvenance, DoctorCategory, DoctorFinding, FindingAssessment, FindingConfidence,
    FindingImpact, FindingSeverity, HealthPenalty, RuleCost, SourceLocation,
};

/// Non-ASCII authored path exercised by the pinned output fingerprint.
const UNICODE_PATH: &str = "src/画面 `#1.vue";

/// Pinned digest of the normalized findings produced by [`identity`].
const POPULATED_OUTPUT_FINGERPRINT: &str =
    "sha256:923e7365d781906a02e090dbacbf7414be8c3ede6c9387714c2342d7c2965571";

/// Pinned digest of a capability that produced no findings at all.
const EMPTY_OUTPUT_FINGERPRINT: &str =
    "sha256:d3a6e45a9aaf9b776a8f6add5725ac941d29c2afd424dcec25e02b612bac23f2";

fn fingerprint(value: &str) -> ContentFingerprint {
    ContentFingerprint::digest(value)
}

fn identity() -> CapabilityCacheIdentity {
    CapabilityCacheIdentity::from_fingerprints(
        "template-semantics",
        fingerprint("implementation"),
        fingerprint("configuration"),
        [
            ("src/A.vue", fingerprint("source-a")),
            ("src/B.vue", fingerprint("source-b")),
            (UNICODE_PATH, fingerprint("source-unicode")),
        ],
    )
    .unwrap()
}

fn finding(code: &str, path: &str, source: &str) -> DoctorFinding {
    finding_at(code, path, source, 0, 1)
}

/// Builds a finding whose source span may cover multi-byte offsets.
fn finding_at(code: &str, path: &str, source: &str, start: u32, end: u32) -> DoctorFinding {
    DoctorFinding::new(
        code,
        DoctorCategory::Correctness,
        FindingAssessment::new(
            FindingSeverity::Warning,
            FindingConfidence::Certain,
            FindingImpact::Medium,
            HealthPenalty::new(10, "test"),
        ),
        SourceLocation::new(path, start, end),
        "Test finding",
        "Test message",
        AnalysisProvenance::new("template-semantics", RuleCost::Low)
            .with_invalidation_fingerprints(BTreeMap::from([(path.into(), fingerprint(source))])),
    )
}

#[test]
fn snapshot_normalizes_findings_and_round_trips_exact_key() {
    let snapshot = CapabilitySnapshot::try_new(
        identity(),
        [
            finding("VIZE_B", "src/B.vue", "source-b"),
            finding("VIZE_A", "src/A.vue", "source-a"),
            // "東京🙂" spans ten UTF-8 bytes, so the span is multi-byte on both ends.
            finding_at("VIZE_U", UNICODE_PATH, "source-unicode", 3, 13),
        ],
    )
    .unwrap();

    assert_eq!(snapshot.format_version(), 1);
    assert_eq!(snapshot.cache_key(), snapshot.identity().cache_key());
    assert_ne!(snapshot.output_fingerprint(), fingerprint("source-a"));
    assert_eq!(
        snapshot.output_fingerprint().to_compact_string(),
        POPULATED_OUTPUT_FINGERPRINT
    );
    assert_eq!(snapshot.findings()[0].code, "VIZE_A");
    assert_eq!(snapshot.findings()[1].code, "VIZE_B");
    assert_eq!(snapshot.findings()[2].code, "VIZE_U");
    assert_eq!(snapshot.findings()[2].primary.path, UNICODE_PATH);
    assert_eq!(snapshot.findings()[2].primary.start, 3);
    assert_eq!(snapshot.findings()[2].primary.end, 13);

    let encoded = serde_json::to_string(&snapshot).unwrap();
    assert_eq!(
        serde_json::from_str::<CapabilitySnapshot>(&encoded).unwrap(),
        snapshot
    );
    let report = snapshot.into_report("workspace");
    assert_eq!(report.workspace(), "workspace");
    assert_eq!(report.findings().len(), 3);
}

#[test]
fn snapshot_rejects_capability_and_input_boundary_mismatches() {
    let mut wrong_capability = finding("VIZE_A", "src/A.vue", "source-a");
    wrong_capability.provenance.capability = "type-semantics".into();
    assert!(matches!(
        CapabilitySnapshot::try_new(identity(), [wrong_capability]),
        Err(CapabilitySnapshotError::CapabilityMismatch { .. })
    ));

    let mut missing = finding("VIZE_A", "src/A.vue", "source-a");
    missing.provenance.invalidation_fingerprints.clear();
    assert!(matches!(
        CapabilitySnapshot::try_new(identity(), [missing]),
        Err(CapabilitySnapshotError::MissingFingerprint { .. })
    ));

    let outside = finding("VIZE_C", "src/C.vue", "source-c");
    assert!(matches!(
        CapabilitySnapshot::try_new(identity(), [outside]),
        Err(CapabilitySnapshotError::UndeclaredIdentityInput { .. })
    ));

    let mismatch = finding("VIZE_A", "src/A.vue", "different-source");
    assert!(matches!(
        CapabilitySnapshot::try_new(identity(), [mismatch]),
        Err(CapabilitySnapshotError::FingerprintMismatch { .. })
    ));

    let mut orphan = finding("VIZE_A", "src/A.vue", "source-a");
    orphan.provenance.invalidation_inputs.clear();
    assert!(matches!(
        CapabilitySnapshot::try_new(identity(), [orphan]),
        Err(CapabilitySnapshotError::OrphanFingerprint { .. })
    ));
}

#[test]
fn wire_rejects_stale_keys_versions_unknown_fields_and_tampering() {
    let snapshot =
        CapabilitySnapshot::try_new(identity(), [finding("VIZE_A", "src/A.vue", "source-a")])
            .unwrap();
    let mut value = serde_json::to_value(snapshot).unwrap();

    value["formatVersion"] = json!(DOCTOR_CAPABILITY_SNAPSHOT_FORMAT_VERSION + 1);
    let error = serde_json::from_value::<CapabilitySnapshot>(value.clone()).unwrap_err();
    assert!(
        error
            .to_compact_string()
            .contains("unsupported capability snapshot version")
    );

    value["formatVersion"] = json!(DOCTOR_CAPABILITY_SNAPSHOT_FORMAT_VERSION);
    value["cacheKey"] = json!(
        CapabilityCacheIdentity::from_fingerprints(
            "type-semantics",
            fingerprint("implementation"),
            fingerprint("configuration"),
            [] as [(&str, ContentFingerprint); 0],
        )
        .unwrap()
        .cache_key()
    );
    let error = serde_json::from_value::<CapabilitySnapshot>(value.clone()).unwrap_err();
    assert!(
        error
            .to_compact_string()
            .contains("does not match derived key")
    );

    value["cacheKey"] = serde_json::to_value(identity().cache_key()).unwrap();
    value["findings"][0]["message"] = json!("Tampered message");
    let error = serde_json::from_value::<CapabilitySnapshot>(value.clone()).unwrap_err();
    assert!(error.to_compact_string().contains("output fingerprint"));

    value["findings"][0]["message"] = json!("Test message");
    value["findings"][0]["provenance"]["capability"] = json!("type-semantics");
    let error = serde_json::from_value::<CapabilitySnapshot>(value.clone()).unwrap_err();
    assert!(error.to_compact_string().contains("names capability"));

    value["findings"][0]["provenance"]["capability"] = json!("template-semantics");
    assert!(serde_json::from_value::<CapabilitySnapshot>(value.clone()).is_ok());

    value["unexpected"] = json!(true);
    let error = serde_json::from_value::<CapabilitySnapshot>(value).unwrap_err();
    assert!(error.to_compact_string().contains("unexpected"));
}

#[test]
fn empty_capability_output_is_cacheable_and_preserves_no_findings() {
    let snapshot = CapabilitySnapshot::try_new(identity(), []).unwrap();
    let key = snapshot.cache_key();

    assert!(snapshot.findings().is_empty());
    assert_eq!(snapshot.identity().cache_key(), key);
    assert_eq!(
        snapshot.output_fingerprint().to_compact_string(),
        EMPTY_OUTPUT_FINGERPRINT
    );
    assert_ne!(EMPTY_OUTPUT_FINGERPRINT, POPULATED_OUTPUT_FINGERPRINT);
    assert_ne!(
        snapshot.output_fingerprint(),
        ContentFingerprint::digest("[]")
    );
    assert!(snapshot.into_findings().is_empty());
}
