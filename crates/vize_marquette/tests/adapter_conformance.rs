use std::{fs, path::PathBuf};

use serde::Deserialize;
use serde_json::Value;
use vize_carton::{String, ToCompactString};
use vize_marquette::{
    ADAPTER_CAPABILITY_MANIFEST_JSON_SCHEMA, AdapterCapabilityManifest, ApplicationContract,
    NATIVE_ENGINE_CAPABILITY_IDS, NATIVE_ENGINE_CAPABILITY_VERSION, compare_adapter_capabilities,
    contract_fingerprint, native_engine_capability_definitions, native_engine_capability_profile,
    negotiate_adapter_capabilities, validate_adapter_capability_manifest,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/marquette")
        .join(name)
}

fn read<T: serde::de::DeserializeOwned>(name: &str) -> T {
    serde_json::from_slice(&fs::read(fixture(name)).unwrap()).unwrap()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NegotiationFixture {
    contract: ApplicationContract,
    cases: Vec<NegotiationCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NegotiationCase {
    name: String,
    required: Vec<String>,
    manifest: AdapterCapabilityManifest,
    expected: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CompatibilityCase {
    name: String,
    previous: AdapterCapabilityManifest,
    next: AdapterCapabilityManifest,
    expected: Value,
}

#[test]
fn native_engine_profile_matches_the_shared_contract_and_negotiates_fail_closed() {
    let expected: AdapterCapabilityManifest = read("native-engine-capability-profile.json");
    let profile = native_engine_capability_profile();
    let actual = AdapterCapabilityManifest {
        format_version: 1,
        adapter: "fixture.native".into(),
        capabilities: profile.clone(),
    };

    assert_eq!(actual, expected);
    assert!(validate_adapter_capability_manifest(&actual).is_empty());
    assert_eq!(profile.len(), NATIVE_ENGINE_CAPABILITY_IDS.len());
    assert!(profile.iter().all(|support| {
        support.min_version == NATIVE_ENGINE_CAPABILITY_VERSION
            && support.max_version == NATIVE_ENGINE_CAPABILITY_VERSION
    }));

    let definitions = native_engine_capability_definitions();
    let definition_ids = definitions
        .iter()
        .map(|definition| definition.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(definition_ids, NATIVE_ENGINE_CAPABILITY_IDS.to_vec());
    assert!(definitions.iter().all(|definition| {
        definition.version == NATIVE_ENGINE_CAPABILITY_VERSION
            && !definition.description.trim().is_empty()
    }));

    let mut contract = ApplicationContract::new("native-profile");
    for definition in definitions {
        contract
            .capabilities
            .insert(definition.id.clone(), definition);
    }
    let compatible =
        negotiate_adapter_capabilities(&contract, NATIVE_ENGINE_CAPABILITY_IDS, &actual);
    assert!(compatible.compatible);

    let mut missing_animation = actual;
    missing_animation
        .capabilities
        .retain(|support| support.id.as_str() != "native.animation");
    let incompatible =
        negotiate_adapter_capabilities(&contract, NATIVE_ENGINE_CAPABILITY_IDS, &missing_animation);
    assert!(!incompatible.compatible);
    assert_eq!(incompatible.mismatches.len(), 1);
    assert_eq!(incompatible.mismatches[0].capability, "native.animation");
    assert_eq!(
        incompatible.mismatches[0].path,
        "capabilities.native.animation"
    );
    assert_eq!(
        incompatible.mismatches[0].message,
        "adapter does not support the required capability"
    );
}

#[test]
fn matches_shared_negotiation_fixtures_and_input_permutations() {
    let fixture: NegotiationFixture = read("adapter-negotiation.json");
    let mut results = Vec::new();
    for case in fixture.cases {
        let actual = negotiate_adapter_capabilities(
            &fixture.contract,
            case.required.iter().map(String::as_str),
            &case.manifest,
        );
        let actual = serde_json::to_value(actual).unwrap();
        assert_eq!(actual, case.expected, "{}", case.name);
        results.push(actual);
    }

    assert_eq!(results[0], results[1], "input order must not affect output");
    assert_eq!(results[2]["compatible"], true, "inclusive bounds must pass");
    assert_eq!(
        results[3]["mismatches"][0]["code"], "unknown-requirement",
        "undeclared requirements must fail even when the adapter offers them"
    );
    assert_eq!(
        results[3]["mismatches"][0]["path"], "capabilities.unknown.capability",
        "undeclared requirement diagnostics must use stable application capability paths"
    );
}

#[test]
fn matches_shared_semantic_validation_diagnostics() {
    let manifest: AdapterCapabilityManifest = read("adapter-manifest-invalid.json");
    let expected: Value = read("adapter-manifest-invalid.expected.json");

    assert_eq!(
        serde_json::to_value(validate_adapter_capability_manifest(&manifest)).unwrap(),
        expected
    );
}

#[test]
fn rejects_unknown_fields_before_negotiation() {
    let error = serde_json::from_slice::<AdapterCapabilityManifest>(
        &fs::read(fixture("adapter-manifest-unknown-field.json")).unwrap(),
    )
    .unwrap_err();

    assert!(
        error
            .to_compact_string()
            .contains("unknown field `zUnexpected`")
    );
}

#[test]
fn schema_rejects_unknown_fields_and_non_positive_bounds() {
    let schema: Value = serde_json::from_str(ADAPTER_CAPABILITY_MANIFEST_JSON_SCHEMA).unwrap();
    let manifest = &schema["$defs"]["manifest"];
    let support = &schema["$defs"]["support"];

    assert_eq!(manifest["additionalProperties"], false);
    assert_eq!(support["additionalProperties"], false);
    assert_eq!(support["properties"]["minVersion"]["minimum"], 1);
    assert_eq!(support["properties"]["maxVersion"]["minimum"], 1);
    assert_eq!(
        support["required"],
        serde_json::json!(["id", "minVersion", "maxVersion"])
    );
}

#[test]
fn matches_shared_adapter_compatibility_matrix() {
    let cases: Vec<CompatibilityCase> = read("adapter-compatibility.json");
    for case in cases {
        let actual = compare_adapter_capabilities(&case.previous, &case.next);
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            case.expected,
            "{}",
            case.name
        );
    }
}

#[test]
fn negotiation_does_not_change_the_application_fingerprint() {
    let contract: ApplicationContract = read("valid.json");
    let manifest = AdapterCapabilityManifest {
        format_version: 1,
        adapter: "fixture.adapter".into(),
        capabilities: vec![vize_marquette::AdapterCapabilitySupport {
            id: "auth.session".into(),
            min_version: 1,
            max_version: 1,
        }],
    };
    let before = contract_fingerprint(&contract).unwrap();

    let result = negotiate_adapter_capabilities(&contract, ["auth.session"], &manifest);

    assert!(result.compatible);
    assert_eq!(contract_fingerprint(&contract).unwrap(), before);
    assert_eq!(
        before.as_str(),
        fs::read_to_string(fixture("valid.sha256")).unwrap().trim()
    );
}
