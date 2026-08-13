use tower_lsp::lsp_types::{
    ClientCapabilities, CreateFilesParams, DeleteFilesParams, FileChangeType, FileEvent, Url,
};

use super::{
    ServerState, changes_invalidate_disk_project_state, record_created_files, record_deleted_files,
    record_watcher_support, typecheck_dependency_watcher_registration,
};

#[test]
fn typecheck_watcher_tracks_declarations_vue_sources_and_manifests() {
    let registration = typecheck_dependency_watcher_registration();
    assert_eq!(registration.method, "workspace/didChangeWatchedFiles");
    let options = registration.register_options.unwrap();
    assert_eq!(options["watchers"][0]["globPattern"], "**/*.d.{ts,mts,cts}");
    assert_eq!(options["watchers"][1]["globPattern"], "**/*.vue");
    assert_eq!(options["watchers"][2]["globPattern"], "**/package.json");
    for watcher in options["watchers"].as_array().unwrap() {
        assert!(
            watcher.get("kind").is_none(),
            "omitted kind must request create, change, and delete events: {options}"
        );
    }
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

#[test]
fn only_vue_content_changes_keep_the_cached_disk_project_state() {
    let vue = "file:///workspace/src/App.vue";
    let declaration = "file:///workspace/components.d.ts";
    let changed = |uri: &str| FileEvent {
        uri: Url::parse(uri).unwrap(),
        typ: FileChangeType::CHANGED,
    };

    assert!(!changes_invalidate_disk_project_state(&[changed(vue)]));
    assert!(changes_invalidate_disk_project_state(&[changed(
        declaration
    )]));
    assert!(changes_invalidate_disk_project_state(&[FileEvent {
        uri: Url::parse(vue).unwrap(),
        typ: FileChangeType::CREATED,
    }]));
    assert!(changes_invalidate_disk_project_state(&[FileEvent {
        uri: Url::parse(vue).unwrap(),
        typ: FileChangeType::DELETED,
    }]));
    assert!(changes_invalidate_disk_project_state(&[
        changed(vue),
        changed(declaration)
    ]));
}
