use std::{fs, path::PathBuf};

use serde::Deserialize;
use serde_json::{Value, json};
use vize_l0::String;
use vize_marquette::{
    ADAPTER_CAPABILITY_MANIFEST_JSON_SCHEMA, APPLICATION_CONTRACT_JSON_SCHEMA,
    AdapterCapabilityManifest, ApplicationContract, negotiate_adapter_capabilities,
    validate_adapter_capability_manifest, validate_contract,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionBoundary {
    name: String,
    version: Value,
    parsed: bool,
    valid: bool,
}

#[test]
fn shared_capability_versions_match_native_parsing_validation_and_negotiation() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/marquette/capability-version-boundaries.json");
    let cases: Vec<VersionBoundary> = serde_json::from_slice(&fs::read(fixture).unwrap()).unwrap();
    for case in cases {
        let manifest = serde_json::from_value::<AdapterCapabilityManifest>(json!({
            "adapter": "native.fixture",
            "capabilities": [{
                "id": "native.feature", "minVersion": case.version, "maxVersion": case.version,
            }],
        }));
        let application = serde_json::from_value::<ApplicationContract>(json!({
            "application": "numeric-boundary",
            "capabilities": {
                "native.feature": {
                    "id": "native.feature", "description": "A native feature", "version": case.version,
                },
            },
        }));
        assert_eq!(manifest.is_ok(), case.parsed, "{}", case.name);
        assert_eq!(application.is_ok(), case.parsed, "{}", case.name);
        if let (Ok(manifest), Ok(application)) = (manifest, application) {
            assert_eq!(
                validate_adapter_capability_manifest(&manifest).is_empty(),
                case.valid,
                "{}",
                case.name,
            );
            assert_eq!(
                validate_contract(&application).is_empty(),
                case.valid,
                "{}",
                case.name
            );
            let result =
                negotiate_adapter_capabilities(&application, ["native.feature"], &manifest);
            assert_eq!(result.compatible, case.valid, "{}", case.name);
            if !case.valid {
                assert_eq!(
                    serde_json::to_value(&result.mismatches).unwrap(),
                    json!([{
                        "code": "invalid-requirement-version",
                        "capability": "native.feature",
                        "path": "capabilities.native.feature",
                        "message": "application capability version must be a positive unsigned 32-bit integer",
                    }]),
                );
            }
        }
    }
}

#[test]
fn published_version_schemas_match_the_native_integer_range() {
    let application: Value = serde_json::from_str(APPLICATION_CONTRACT_JSON_SCHEMA).unwrap();
    let adapter: Value = serde_json::from_str(ADAPTER_CAPABILITY_MANIFEST_JSON_SCHEMA).unwrap();
    for version in [
        &application["$defs"]["capabilityDefinition"]["properties"]["version"],
        &adapter["$defs"]["support"]["properties"]["minVersion"],
        &adapter["$defs"]["support"]["properties"]["maxVersion"],
    ] {
        assert_eq!(version["type"], "integer");
        assert_eq!(version["minimum"], 1);
        assert_eq!(version["maximum"], u32::MAX);
    }
}
