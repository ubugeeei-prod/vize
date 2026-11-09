use super::{NativeBuildProfile, NativeBuildProfileDiagnostic, validate_native_build_profile};

const VALID: &str =
    include_str!("../../../../tests/fixtures/marquette/native-build-profile.valid.json");
const INVALID: &str =
    include_str!("../../../../tests/fixtures/marquette/native-build-profile.invalid.json");
const INVALID_EXPECTED: &str =
    include_str!("../../../../tests/fixtures/marquette/native-build-profile.invalid.expected.json");

#[test]
fn accepts_the_shared_production_profile() {
    let profile: NativeBuildProfile = serde_json::from_str(VALID).unwrap();
    assert!(validate_native_build_profile(&profile).is_empty());
    assert_eq!(profile.targets.len(), 2);
    assert_eq!(profile.capabilities.len(), 2);
}

#[test]
fn matches_shared_semantic_diagnostics() {
    let profile: NativeBuildProfile = serde_json::from_str(INVALID).unwrap();
    let expected: Vec<NativeBuildProfileDiagnostic> =
        serde_json::from_str(INVALID_EXPECTED).unwrap();
    assert_eq!(validate_native_build_profile(&profile), expected);
}

#[test]
fn strict_parsing_rejects_executor_and_secret_fields() {
    let unknown_executor = VALID.replace(
        "\"profile\": \"production\",",
        "\"profile\": \"production\", \"executor\": \"hosted\",",
    );
    assert!(serde_json::from_str::<NativeBuildProfile>(&unknown_executor).is_err());

    let target_secret = VALID.replace(
        "\"signing\": \"automatic\"",
        "\"signing\": \"automatic\", \"secret\": \"value\"",
    );
    assert!(serde_json::from_str::<NativeBuildProfile>(&target_secret).is_err());
}
