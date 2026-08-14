use tower_lsp::lsp_types::{
    ClientCapabilities, CreateFilesParams, DeleteFilesParams, FileChangeType, FileEvent, Url,
};

use super::{
    ServerState, changes_invalidate_disk_project_state, record_created_files, record_deleted_files,
    record_watcher_support, typecheck_dependency_watcher_registration, user_watched_file_events,
};

#[test]
fn typecheck_watcher_tracks_declarations_vue_sources_and_manifests() {
    let registration = typecheck_dependency_watcher_registration();
    assert_eq!(registration.method, "workspace/didChangeWatchedFiles");
    let options = registration.register_options.unwrap();
    assert_eq!(options["watchers"][0]["globPattern"], "**/*.d.{ts,mts,cts}");
    assert_eq!(options["watchers"][1]["globPattern"], "**/*.vue");
    assert_eq!(options["watchers"][2]["globPattern"], "**/package.json");
    assert_eq!(options["watchers"][3]["globPattern"], "**/tsconfig*.json");
    assert_eq!(options["watchers"][4]["globPattern"], "**/jsconfig.json");
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
    let tsconfig = "file:///workspace/tsconfig.app.json";
    let jsconfig = "file:///workspace/jsconfig.json";
    let changed = |uri: &str| FileEvent {
        uri: Url::parse(uri).unwrap(),
        typ: FileChangeType::CHANGED,
    };
    let state = ServerState::new();
    let vue_uri = Url::parse(vue).unwrap();
    state
        .documents
        .open(vue_uri.clone(), "<template />\n".into(), 1, "vue".into());

    assert!(!changes_invalidate_disk_project_state(
        &state,
        &[changed(vue)]
    ));
    assert!(changes_invalidate_disk_project_state(
        &state,
        &[changed(declaration)]
    ));
    assert!(changes_invalidate_disk_project_state(
        &state,
        &[changed(tsconfig)]
    ));
    assert!(changes_invalidate_disk_project_state(
        &state,
        &[changed(jsconfig)]
    ));
    assert!(changes_invalidate_disk_project_state(
        &state,
        &[FileEvent {
            uri: vue_uri.clone(),
            typ: FileChangeType::CREATED,
        }]
    ));
    assert!(changes_invalidate_disk_project_state(
        &state,
        &[FileEvent {
            uri: vue_uri,
            typ: FileChangeType::DELETED,
        }]
    ));
    assert!(changes_invalidate_disk_project_state(
        &state,
        &[changed(vue), changed(declaration)]
    ));
}

#[test]
fn internal_corsa_overlay_events_do_not_reenter_watched_file_handling() {
    let root = tempfile::tempdir().unwrap();
    let overlay_path = root
        .path()
        .join("node_modules/.vize/corsa-overlay/tsconfig.json");
    let project_tsconfig_path = root.path().join("tsconfig.json");
    let overlay_event = FileEvent {
        uri: Url::from_file_path(overlay_path).unwrap(),
        typ: FileChangeType::CREATED,
    };
    let project_event = FileEvent {
        uri: Url::from_file_path(project_tsconfig_path).unwrap(),
        typ: FileChangeType::CHANGED,
    };
    let state = ServerState::new();

    let ignored = user_watched_file_events(std::slice::from_ref(&overlay_event));
    assert!(ignored.is_empty());
    assert!(!changes_invalidate_disk_project_state(&state, &ignored));

    let kept = user_watched_file_events(&[overlay_event, project_event.clone()]);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].uri, project_event.uri);
    assert!(changes_invalidate_disk_project_state(&state, &kept));
}

#[test]
fn closed_vue_content_changes_invalidate_the_cached_disk_project_state() {
    let root = tempfile::tempdir().unwrap();
    let open_path = root.path().join("Open.vue");
    let closed_path = root.path().join("Closed.vue");
    std::fs::write(&open_path, "<template />\n").unwrap();
    std::fs::write(&closed_path, "<template />\n").unwrap();
    let open_uri = Url::from_file_path(&open_path).unwrap();
    let closed_uri = Url::from_file_path(&closed_path).unwrap();
    let changed = |uri: Url| FileEvent {
        uri,
        typ: FileChangeType::CHANGED,
    };
    let state = ServerState::new();
    state
        .documents
        .open(open_uri.clone(), "<template />\n".into(), 1, "vue".into());

    assert!(!changes_invalidate_disk_project_state(
        &state,
        &[changed(open_uri)]
    ));
    assert!(changes_invalidate_disk_project_state(
        &state,
        &[changed(closed_uri)]
    ));
}
