use tower_lsp::lsp_types::{ClientCapabilities, CreateFilesParams, DeleteFilesParams, Url};

use super::{
    ServerState, global_component_watcher_registration, record_created_files, record_deleted_files,
    record_watcher_support,
};

#[test]
fn declaration_watcher_tracks_create_change_and_delete_recursively() {
    let registration = global_component_watcher_registration();
    assert_eq!(registration.method, "workspace/didChangeWatchedFiles");
    let options = registration.register_options.unwrap();
    assert_eq!(options["watchers"][0]["globPattern"], "**/*.d.{ts,mts,cts}");
    assert!(
        options["watchers"][0].get("kind").is_none(),
        "omitted kind must request create, change, and delete events: {options}"
    );
}

#[test]
fn client_watcher_support_is_recorded_from_initialize_capabilities() {
    let capabilities: ClientCapabilities = serde_json::from_value(serde_json::json!({
        "workspace": {
            "didChangeWatchedFiles": { "dynamicRegistration": true }
        }
    }))
    .unwrap();
    let state = ServerState::new();

    record_watcher_support(&state, &capabilities);

    assert!(state.global_component_watcher_supported());
}

#[test]
fn vue_file_events_track_only_existing_created_files_and_forget_deletes() {
    let root = tempfile::tempdir().unwrap();
    let vue_path = root.path().join("DiskChild.vue");
    let declaration_path = root.path().join("components.d.ts");
    std::fs::write(&vue_path, "<template />\n").unwrap();
    std::fs::write(&declaration_path, "export {};\n").unwrap();
    let vue_uri = Url::from_file_path(&vue_path).unwrap();
    let declaration_uri = Url::from_file_path(&declaration_path).unwrap();
    let missing_uri = Url::from_file_path(root.path().join("Missing.vue")).unwrap();
    let state = ServerState::new();

    let created: CreateFilesParams = serde_json::from_value(serde_json::json!({
        "files": [
            { "uri": vue_uri.as_str() },
            { "uri": declaration_uri.as_str() },
            { "uri": missing_uri.as_str() }
        ]
    }))
    .unwrap();
    record_created_files(&state, &created);

    assert_eq!(state.workspace_vue_file_uris(), vec![vue_uri.clone()]);

    std::fs::remove_file(&vue_path).unwrap();
    let deleted: DeleteFilesParams = serde_json::from_value(serde_json::json!({
        "files": [{ "uri": vue_uri.as_str() }]
    }))
    .unwrap();
    record_deleted_files(&state, &deleted);

    assert!(state.workspace_vue_file_uris().is_empty());
}
