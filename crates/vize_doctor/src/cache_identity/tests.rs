use serde_json::json;
use vize_carton::{ToCompactString, cstr};

use super::{
    CAPABILITY_CACHE_KEY_PREFIX, CapabilityCacheIdentity, CapabilityCacheIdentityError,
    CapabilityCacheInput, CapabilityCacheKey, ContentFingerprint,
    DOCTOR_CAPABILITY_CACHE_IDENTITY_VERSION,
};

fn fingerprint(value: impl AsRef<[u8]>) -> ContentFingerprint {
    ContentFingerprint::digest(value)
}

fn identity(
    capability: &str,
    implementation: &str,
    configuration: &str,
    inputs: &[(&str, &str)],
) -> CapabilityCacheIdentity {
    CapabilityCacheIdentity::from_fingerprints(
        capability,
        fingerprint(implementation),
        fingerprint(configuration),
        inputs
            .iter()
            .map(|(id, content)| (*id, fingerprint(content))),
    )
    .unwrap()
}

#[test]
fn canonical_identity_is_order_independent_and_round_trips() {
    let forward = identity(
        "whole-project-diagnostic-graph",
        "analyzer-v3",
        "strict=true",
        &[("src/App.vue", "app"), ("設定/共有.ts", "shared")],
    );
    let reverse = identity(
        "whole-project-diagnostic-graph",
        "analyzer-v3",
        "strict=true",
        &[("設定/共有.ts", "shared"), ("src/App.vue", "app")],
    );

    assert_eq!(forward, reverse);
    assert_eq!(forward.contract_version(), 1);
    assert_eq!(forward.capability(), "whole-project-diagnostic-graph");
    assert_eq!(forward.inputs()[0].id(), "src/App.vue");
    assert_eq!(forward.inputs()[1].id(), "設定/共有.ts");
    assert_eq!(forward.inputs()[0].fingerprint(), fingerprint("app"));
    assert_eq!(forward.cache_key(), reverse.cache_key());

    let json = serde_json::to_string(&forward).unwrap();
    assert_eq!(
        serde_json::from_str::<CapabilityCacheIdentity>(&json).unwrap(),
        forward
    );
}

#[test]
fn cache_key_is_domain_separated_strict_and_stable() {
    let identity = identity(
        "template-semantics",
        "implementation",
        "configuration",
        &[("src/画面.vue", "<template />"), ("src/raw.bin", "\0|")],
    );
    let key = identity.cache_key();
    let text = key.to_compact_string();

    assert!(text.starts_with(CAPABILITY_CACHE_KEY_PREFIX));
    assert_eq!(
        text,
        "vize-doctor-capability-v1:sha256:2d3bd59c5ca7d1f7c37eb998e5b1009375dee5bb3456fec3eebccc3456b2cba1"
    );
    assert_eq!(text.parse::<CapabilityCacheKey>().unwrap(), key);
    assert_eq!(
        serde_json::from_str::<CapabilityCacheKey>(&serde_json::to_string(&key).unwrap()).unwrap(),
        key
    );
    assert_eq!(
        key.fingerprint().to_compact_string(),
        text.trim_start_matches(CAPABILITY_CACHE_KEY_PREFIX)
    );
    assert!(
        text.replacen("vize", "Vize", 1)
            .parse::<CapabilityCacheKey>()
            .is_err()
    );
    assert!(
        text.replacen("sha256:", "SHA256:", 1)
            .parse::<CapabilityCacheKey>()
            .is_err()
    );
}

#[test]
fn every_identity_boundary_changes_the_cache_key() {
    let baseline = identity("syntax", "v1", "strict", &[("a", "x"), ("bc", "y")]);
    let variants = [
        identity("types", "v1", "strict", &[("a", "x"), ("bc", "y")]),
        identity("syntax", "v2", "strict", &[("a", "x"), ("bc", "y")]),
        identity("syntax", "v1", "loose", &[("a", "x"), ("bc", "y")]),
        identity("syntax", "v1", "strict", &[("a", "changed"), ("bc", "y")]),
        identity("syntax", "v1", "strict", &[("ab", "x"), ("c", "y")]),
        identity("syntax", "v1", "strict", &[("a", "x")]),
    ];

    for variant in variants {
        assert_ne!(baseline.cache_key(), variant.cache_key());
    }
}

#[test]
fn invalidation_classifies_every_boundary_in_stable_order() {
    let previous = identity(
        "syntax",
        "v1",
        "strict",
        &[("a", "removed"), ("b", "same"), ("d", "old")],
    );
    let current = identity(
        "types",
        "v2",
        "loose",
        &[("b", "same"), ("c", "added"), ("d", "new")],
    );
    let invalidation = current.invalidation_from(&previous);

    assert!(!invalidation.is_reusable());
    assert!(invalidation.capability_changed());
    assert!(invalidation.implementation_changed());
    assert!(invalidation.configuration_changed());
    assert_eq!(invalidation.added_inputs(), ["c"]);
    assert_eq!(invalidation.removed_inputs(), ["a"]);
    assert_eq!(invalidation.changed_inputs(), ["d"]);
    assert_eq!(invalidation.telemetry().added_input_count(), 1);
    assert_eq!(invalidation.telemetry().removed_input_count(), 1);
    assert_eq!(invalidation.telemetry().changed_input_count(), 1);
    assert_eq!(
        serde_json::to_value(invalidation.telemetry()).unwrap(),
        json!({
            "reusable": false,
            "capabilityChanged": true,
            "implementationChanged": true,
            "configurationChanged": true,
            "addedInputCount": 1,
            "removedInputCount": 1,
            "changedInputCount": 1
        })
    );

    let reusable = current.invalidation_from(&current);
    assert!(reusable.is_reusable());
    assert!(reusable.telemetry().is_reusable());
    assert_eq!(
        serde_json::to_value(reusable).unwrap(),
        json!({
            "capabilityChanged": false,
            "implementationChanged": false,
            "configurationChanged": false,
            "addedInputs": [],
            "removedInputs": [],
            "changedInputs": []
        })
    );
}

#[test]
fn constructor_rejects_ambiguous_identifiers_and_duplicates() {
    for capability in ["", "Syntax", "syntax--graph", "syntax_graph", "syntax-"] {
        let error = CapabilityCacheIdentity::try_new(
            capability,
            fingerprint("implementation"),
            fingerprint("configuration"),
            [],
        )
        .unwrap_err();
        assert!(matches!(
            error,
            CapabilityCacheIdentityError::InvalidCapabilityId { .. }
        ));
    }

    for input in [
        "",
        " src/App.vue",
        "src\\App.vue",
        "/src/App.vue",
        "C:/src/App.vue",
        "src//App.vue",
        "src/./App.vue",
        "src/../App.vue",
        "src/App.vue\n",
    ] {
        assert!(matches!(
            CapabilityCacheInput::try_new(input, fingerprint("source")),
            Err(CapabilityCacheIdentityError::InvalidInputId { .. })
        ));
    }

    let duplicate = CapabilityCacheIdentity::from_fingerprints(
        "syntax",
        fingerprint("implementation"),
        fingerprint("configuration"),
        [
            ("src/App.vue", fingerprint("first")),
            ("src/App.vue", fingerprint("second")),
        ],
    )
    .unwrap_err();
    assert!(matches!(
        duplicate,
        CapabilityCacheIdentityError::DuplicateInput { .. }
    ));
}

#[test]
fn wire_rejects_noncanonical_order_versions_duplicates_and_unknown_fields() {
    let identity = identity(
        "syntax",
        "implementation",
        "configuration",
        &[("a", "first"), ("b", "second")],
    );
    let mut value = serde_json::to_value(identity).unwrap();

    value["contractVersion"] = json!(DOCTOR_CAPABILITY_CACHE_IDENTITY_VERSION + 1);
    let error = serde_json::from_value::<CapabilityCacheIdentity>(value.clone()).unwrap_err();
    assert!(
        error
            .to_compact_string()
            .contains("unsupported capability cache identity version")
    );

    value["contractVersion"] = json!(DOCTOR_CAPABILITY_CACHE_IDENTITY_VERSION);
    value["inputs"].as_array_mut().unwrap().reverse();
    let error = serde_json::from_value::<CapabilityCacheIdentity>(value.clone()).unwrap_err();
    assert!(
        error
            .to_compact_string()
            .contains("inputs are not canonical")
    );

    value["inputs"][0]["id"] = json!("a");
    value["inputs"][1]["id"] = json!("a");
    let error = serde_json::from_value::<CapabilityCacheIdentity>(value.clone()).unwrap_err();
    assert!(
        error
            .to_compact_string()
            .contains("duplicate capability cache input")
    );

    value["unexpected"] = json!(true);
    assert!(serde_json::from_value::<CapabilityCacheIdentity>(value).is_err());
}

#[test]
fn empty_input_capabilities_remain_exact_and_distinct() {
    let first = identity("configuration", "v1", "enabled", &[]);
    let second = identity("configuration", "v1", "disabled", &[]);

    assert!(first.inputs().is_empty());
    assert_ne!(first.cache_key(), second.cache_key());
    assert_eq!(
        first.cache_key().to_compact_string().len(),
        CAPABILITY_CACHE_KEY_PREFIX.len() + cstr!("sha256:").len() + 64
    );
}

#[test]
fn ten_thousand_input_comparison_reports_only_the_changed_boundary() {
    let inputs = (0_u32..10_000)
        .map(|index| {
            (
                cstr!("src/generated/{index:05}.vue"),
                fingerprint(index.to_be_bytes()),
            )
        })
        .collect::<Vec<_>>();
    let previous = CapabilityCacheIdentity::from_fingerprints(
        "whole-project-graph",
        fingerprint("implementation"),
        fingerprint("configuration"),
        inputs.clone(),
    )
    .unwrap();
    let mut changed = inputs;
    changed[5_000].1 = fingerprint("changed");
    let current = CapabilityCacheIdentity::from_fingerprints(
        "whole-project-graph",
        fingerprint("implementation"),
        fingerprint("configuration"),
        changed,
    )
    .unwrap();

    let invalidation = current.invalidation_from(&previous);
    assert_eq!(invalidation.changed_inputs(), ["src/generated/05000.vue"]);
    assert!(invalidation.added_inputs().is_empty());
    assert!(invalidation.removed_inputs().is_empty());
}
