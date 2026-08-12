use super::{
    DiagnosticPresentation, DiagnosticPresentationError, DiagnosticPresentationKind, DiagnosticTone,
};

#[test]
fn structured_kinds_require_their_validating_constructors() {
    for kind in [
        DiagnosticPresentationKind::Score,
        DiagnosticPresentationKind::CodeLocation,
        DiagnosticPresentationKind::Evidence,
        DiagnosticPresentationKind::KeyHint,
    ] {
        assert!(matches!(
            DiagnosticPresentation::new(kind, "value", DiagnosticTone::Neutral),
            Err(DiagnosticPresentationError::InvalidStructure { .. })
        ));
    }
}

#[test]
fn wire_shape_is_canonical_and_rejects_unknown_fields() {
    let presentation = DiagnosticPresentation::evidence("型の証拠 🧭", 3, 9)
        .unwrap()
        .with_description("Combining mark: e\u{301}")
        .unwrap();
    let value = serde_json::to_value(&presentation).unwrap();
    assert_eq!(value["kind"], "evidence");
    assert_eq!(value["tone"], "informational");
    assert_eq!(value["setPosition"], serde_json::json!([3, 9]));
    assert!(value.get("set_position").is_none());

    let mut tampered = value.clone();
    tampered["vendorExtension"] = serde_json::json!(true);
    assert!(serde_json::from_value::<DiagnosticPresentation>(tampered).is_err());
    assert_eq!(
        serde_json::from_value::<DiagnosticPresentation>(value).unwrap(),
        presentation
    );
}

#[test]
fn wire_input_cannot_bypass_score_or_evidence_invariants() {
    let invalid_score = serde_json::json!({
        "kind": "score",
        "tone": "negative",
        "value": "101 / 100",
        "description": null,
        "score": [101, 100],
        "setPosition": null
    });
    assert!(serde_json::from_value::<DiagnosticPresentation>(invalid_score).is_err());

    let inconsistent_score = serde_json::json!({
        "kind": "score",
        "tone": "positive",
        "value": "99 / 100",
        "score": [92, 100]
    });
    assert!(serde_json::from_value::<DiagnosticPresentation>(inconsistent_score).is_err());

    let incomplete_evidence = serde_json::json!({
        "kind": "evidence",
        "tone": "informational",
        "value": "Related component"
    });
    assert!(serde_json::from_value::<DiagnosticPresentation>(incomplete_evidence).is_err());
}

#[test]
fn wire_input_revalidates_location_and_key_hint_text() {
    let invalid_location = serde_json::json!({
        "kind": "code-location",
        "tone": "neutral",
        "value": "src/App.vue:0:1"
    });
    assert!(serde_json::from_value::<DiagnosticPresentation>(invalid_location).is_err());

    let invalid_key_hint = serde_json::json!({
        "kind": "key-hint",
        "tone": "neutral",
        "value": "missing action separator"
    });
    assert!(serde_json::from_value::<DiagnosticPresentation>(invalid_key_hint).is_err());
}
