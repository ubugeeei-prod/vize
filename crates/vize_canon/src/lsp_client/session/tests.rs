use super::{
    ProjectSessionSpawnError, api_mode_for_executable, build_session_document_uri,
    classify_project_session_error, line_character_to_utf16_offset,
    overlay_changes_error_is_unsupported, should_retry_json_rpc, uri_document_identifier,
};
use crate::file_uri::path_to_file_uri;
use corsa::CorsaError;
use corsa::api::{ApiMode, DocumentIdentifier};

// Keep in-project virtual `.vue.ts` overlays at real paths so relative
// script imports resolve against the source tree.
#[test]
fn keeps_vue_virtual_overlay_at_real_path_inside_project() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let project = std::env::temp_dir().join(format!(
        "vize-canon-session-uri-{}-{nonce}",
        std::process::id()
    ));
    let components = project.join("src/components");
    std::fs::create_dir_all(&components).unwrap();
    let real = components.join("Button.vue");
    std::fs::write(&real, "<template><div /></template>").unwrap();

    let virtual_path = components.join("Button.vue.ts");
    let uri = path_to_file_uri(&virtual_path);
    let mapped = build_session_document_uri(&uri, &project, true);
    assert_eq!(mapped, uri, "in-project .vue.ts overlay must keep its path");

    let mapped_no_overlay = build_session_document_uri(&uri, &project, false);
    assert_eq!(
        mapped_no_overlay,
        path_to_file_uri(
            &project.join("node_modules/.vize/corsa-overlay/src/components/Button.vue.ts")
        ),
        "in-project overlays must preserve their project-relative path"
    );

    // A path outside the project is still remapped into the overlay tree.
    let outside = std::env::temp_dir().join(format!("vize-outside-{nonce}/Other.vue.ts"));
    let outside_uri = path_to_file_uri(&outside);
    let mapped_outside = build_session_document_uri(&outside_uri, &project, true);
    assert_ne!(mapped_outside, outside_uri);

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn uses_async_json_rpc_for_node_modules_bin_wrappers() {
    assert_eq!(
        api_mode_for_executable("/tmp/project/node_modules/.bin/tsgo"),
        ApiMode::AsyncJsonRpcStdio
    );
}

#[test]
fn uses_async_json_rpc_for_native_preview_js_entrypoints() {
    assert_eq!(
        api_mode_for_executable("/tmp/project/node_modules/@typescript/native-preview/bin/tsgo.js"),
        ApiMode::AsyncJsonRpcStdio
    );
}

#[test]
fn uses_async_json_rpc_for_typescript_seven_native_binaries() {
    for executable in ["tsc", "tsc.exe"] {
        let path = format!(
            "/tmp/project/node_modules/@typescript/typescript-darwin-arm64/lib/{executable}"
        );

        assert_eq!(
            api_mode_for_executable(&path),
            ApiMode::AsyncJsonRpcStdio,
            "{path}"
        );
    }
}

#[test]
fn keeps_native_binaries_on_sync_msgpack() {
    assert_eq!(
        api_mode_for_executable("/tmp/project/corsa-bind/.cache/tsgo"),
        ApiMode::SyncMsgpackStdio
    );
}

#[test]
fn retries_json_rpc_after_msgpack_shape_mismatch() {
    let error = CorsaError::Protocol("expected tuple marker, got 61".into());

    assert!(should_retry_json_rpc(ApiMode::SyncMsgpackStdio, &error));
    assert!(!should_retry_json_rpc(ApiMode::AsyncJsonRpcStdio, &error));
}

// Regression: the materialized-overlay fallback must be gated on the typed
// `CorsaError::Unsupported` variant that corsa-bind raises when a runtime
// rejects `updateSnapshot.overlayChanges`, not on sniffing the rendered
// error message. A runtime that fails an overlay write for an unrelated
// reason (e.g. a transport/protocol fault) must surface as an error rather
// than silently degrading to the slower materialized path.
#[test]
fn overlay_unsupported_uses_typed_capability_error() {
    assert!(overlay_changes_error_is_unsupported(
        &CorsaError::Unsupported("updateSnapshot.overlayChanges is not supported by this runtime",)
    ));

    // A look-alike message routed through a different (protocol) variant
    // must NOT be treated as an overlay-capability gap.
    assert!(!overlay_changes_error_is_unsupported(
        &CorsaError::Protocol("overlayChanges write failed: connection unsupported".into())
    ));
}

#[test]
fn utf16_offsets_clamp_to_line_boundaries() {
    assert_eq!(line_character_to_utf16_offset("alpha\nbeta", 0, 99), 5);
    assert_eq!(line_character_to_utf16_offset("a😀b", 0, 3), 3);
    assert_eq!(line_character_to_utf16_offset("a\nb", 9, 0), 3);
}

#[test]
fn api_queries_use_uri_document_identifiers() {
    assert!(matches!(
        uri_document_identifier("file:///workspace/App.vue.ts"),
        DocumentIdentifier::FileName(_)
    ));
    assert!(matches!(
        uri_document_identifier("corsa://overlay/App.vue.ts"),
        DocumentIdentifier::Uri { .. }
    ));
}

#[test]
fn recognizes_only_the_standard_runtime_project_session_gap() {
    let unavailable = classify_project_session_error(
        CorsaError::Protocol("project session did not resolve a project".into()),
        None,
    );
    assert!(matches!(
        unavailable,
        ProjectSessionSpawnError::Unavailable(_)
    ));

    for error in [
        CorsaError::Protocol("EOF while parsing project response".into()),
        CorsaError::Protocol("project session crashed".into()),
    ] {
        assert!(matches!(
            classify_project_session_error(error, None),
            ProjectSessionSpawnError::Failed(_)
        ));
    }
}
