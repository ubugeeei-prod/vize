//! Pure source-boundary controls. These never create a native client/session.

use super::*;
use serde_json::json;

fn project() -> ProjectResponse {
    ProjectResponse {
        id: "project".into(),
        config_file_name: "/p/tsconfig.json".into(),
        compiler_options: json!({"strict":true,"paths":{"vue":["./selected-vue"]}}),
        root_files: vec!["/p/generated.ts".into()],
    }
}

fn anchor() -> Anchor {
    Anchor {
        start: 10,
        end: 14,
        position: 10,
        text: "host".into(),
    }
}

fn expected() -> SourceExpectation {
    SourceExpectation {
        file: "/p/generated.ts".into(),
        path: "/p/generated.ts".into(),
        text: "😀\r\nconst host = 1;".into(),
    }
}

fn request() -> UsageRequest {
    UsageRequest {
        usage_id: "original27:constructor-public-props-discrepancy:9".into(),
        authored_file: "/p/Tiny.vue".into(),
        authored_source: "<WitnessChild :value=\"1\" />\r\n".into(),
        authored_usage: Anchor {
            start: 1,
            end: 13,
            position: 1,
            text: "WitnessChild".into(),
        },
        config_file: "/p/tsconfig.json".into(),
        generated: expected(),
        component_source: expected(),
        component_reference: anchor(),
        vue_module: anchor(),
        hosts: [anchor(), anchor()],
    }
}

fn native_source() -> EncodedSourceFile {
    EncodedSourceFile {
        protocol_version: 9,
        file_name: "/p/generated.ts".into(),
        path: "/p/generated.ts".into(),
        text: "😀\r\nconst host = 1;".into(),
        original_text: "<WitnessChild :value=\"1\" />\r\n".into(),
        language_variant: 0,
        script_kind: 3,
        content_mapping: None,
    }
}

#[test]
fn selected_project_requires_the_whole_same_snapshot_descriptor() {
    let selected = project();
    assert!(project_matches(
        std::slice::from_ref(&selected),
        &selected,
        "/p/tsconfig.json"
    ));
    assert!(!project_matches(&[], &selected, "/p/tsconfig.json"));
    assert!(!project_matches(
        &[selected.clone(), selected.clone()],
        &selected,
        "/p/tsconfig.json"
    ));
    assert!(!project_matches(
        std::slice::from_ref(&selected),
        &selected,
        "/other/tsconfig.json"
    ));
    let mut foreign = selected.clone();
    foreign.compiler_options = json!({"strict":false});
    assert!(!project_matches(&[foreign], &selected, "/p/tsconfig.json"));
    let mut foreign = selected.clone();
    foreign.root_files.push("/foreign/input.ts".into());
    assert!(!project_matches(&[foreign], &selected, "/p/tsconfig.json"));
    let mut other = selected.clone();
    other.id = "other-project".into();
    assert!(project_matches(
        &[other, selected.clone()],
        &selected,
        "/p/tsconfig.json"
    ));
}

#[test]
fn source_joins_require_exact_names_paths_and_whole_unicode_crlf_text() {
    let expected = expected();
    assert_eq!(source_matches(&expected, &native_source()), None);
    let mut changed = native_source();
    changed.file_name = "/foreign/generated.ts".into();
    assert_eq!(
        source_matches(&expected, &changed),
        Some(Unavailable::SourceIdentity(expected.file.clone()))
    );
    let mut changed = native_source();
    changed.path = "/p/../p/generated.ts".into();
    assert_eq!(
        source_matches(&expected, &changed),
        Some(Unavailable::SourceIdentity(expected.file.clone()))
    );
    for text in [
        "😀\nconst host = 1;",
        "😀\r\nconst host = 2;",
        "😀\r\nconst host = 1; ",
    ] {
        let mut changed = native_source();
        changed.text = text.into();
        assert_eq!(
            source_matches(&expected, &changed),
            Some(Unavailable::SourceText(expected.file.clone()))
        );
    }
    // A mapped source's authored text is retained separately, never replaced
    // by or confused with the exact generated text being queried.
    assert_ne!(native_source().original_text, native_source().text);
}

#[test]
fn anchors_use_complete_utf16_ranges_without_splitting_astral_characters() {
    let source = expected().text;
    assert!(anchor_matches(&source, &anchor()));
    for (start, end, position) in [
        (1, 14, 10),
        (10, 13, 10),
        (10, 14, 14),
        (14, 10, 10),
        (10, 100, 10),
    ] {
        let changed = Anchor {
            start,
            end,
            position,
            ..anchor()
        };
        assert!(!anchor_matches(&source, &changed));
    }
    let mut changed = anchor();
    changed.text = "post".into();
    assert!(!anchor_matches(&source, &changed));
    assert!(!anchor_matches("😀\nconst host = 1;", &anchor()));
    let usage = request();
    assert!(anchor_matches(
        &usage.authored_source,
        &usage.authored_usage
    ));
    assert!(!anchor_matches("<DifferentChild />", &usage.authored_usage));
}

#[test]
fn source_membership_refuses_missing_duplicate_foreign_and_malformed_names() {
    let file = "/p/generated.ts";
    assert!(source_member(Some(&json!([file, "/p/component.ts"])), file));
    for names in [
        json!([]),
        json!([file, file]),
        json!(["/foreign/generated.ts"]),
        json!([file, 42]),
        json!({"files":[file]}),
        json!(null),
    ] {
        assert!(!source_member(Some(&names), file));
    }
    assert!(!source_member(None, file));
}

#[test]
fn sdk_missing_generic_metadata_is_not_proof_of_a_nongeneric_signature() {
    let absent: SignatureResponse = serde_json::from_value(json!({
        "id":"signature","flags":0,"parameters":[]
    }))
    .unwrap();
    let empty: SignatureResponse = serde_json::from_value(json!({
        "id":"signature","flags":0,"parameters":[],"typeParameters":[]
    }))
    .unwrap();
    assert_eq!(
        absent, empty,
        "the actual SDK DTO cannot distinguish these inputs"
    );
    let data = ObservationData::new(request(), "snapshot".into(), project());
    assert!(
        data.unavailable
            .contains(&Unavailable::GenericMetadataPresence)
    );
    assert!(!data.native_admission);
}

#[test]
fn compact_or_malformed_declarations_cannot_supply_native_call_ranges() {
    let full = NodeHandle::from("1.5.176./p/component.ts");
    assert_eq!(full.parse().unwrap().path.as_str(), "/p/component.ts");
    let compact = NodeHandle::from("42.176./p/component.ts");
    assert_eq!(compact.declaring_path().as_deref(), Some("/p/component.ts"));
    assert!(
        compact.parse().is_err(),
        "a native filename is not a native call range"
    );
    for node in ["42.176./p/component.ts", "1.5.176.", "bad"] {
        assert!(NodeHandle::from(node).parse().is_err());
    }
    let data = ObservationData::new(request(), "snapshot".into(), project());
    assert!(
        data.unavailable
            .contains(&Unavailable::SelectedSignatureRequiresNativeCallNode)
    );
    assert!(!data.native_admission);
}

#[test]
fn matching_property_declarations_and_headers_never_grant_vue_admission() {
    let props: SymbolResponse = serde_json::from_value(json!({
        "id":"same-symbol","name":"$props","flags":4,"checkFlags":123,
        "declarations":["1.5.176./p/selected-vue.d.ts"],
        "valueDeclaration":"1.5.176./p/selected-vue.d.ts"
    }))
    .unwrap();
    let mut data = ObservationData::new(request(), "snapshot".into(), project());
    data.vue.props = Some(props.clone());
    data.hosts.push(HostObservation {
        anchor: anchor(),
        pair: None,
        with_arguments: None,
        instance: None,
        instance_symbol: None,
        props: Some(props),
        props_type: None,
    });
    data.sources.push(NativeSource {
        requested: "/p/generated.ts".into(),
        encoded: Some(vec![9, 0, 0]),
        decoded: Some(native_source()),
    });
    let value = serde_json::to_value(&data).unwrap();
    assert_eq!(value["hosts"][0]["props"], value["vue"]["props"]);
    assert_eq!(value["hosts"][0]["props"]["checkFlags"], 123);
    assert_eq!(
        value["sources"][0]["decoded"]["originalText"],
        "<WitnessChild :value=\"1\" />\r\n"
    );
    assert_eq!(value["sources"][0]["encoded"], json!([9, 0, 0]));
    assert_eq!(value["nativeAdmission"], false);
    assert!(
        data.unavailable
            .contains(&Unavailable::VueReturnOriginNotAdmitted)
    );
}
