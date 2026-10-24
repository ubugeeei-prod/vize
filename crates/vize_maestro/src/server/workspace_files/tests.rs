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
    assert_eq!(
        options,
        serde_json::json!({"watchers":[
            {"globPattern":"**/*.d.{ts,mts,cts}"},
            {"globPattern":"**/*.{vue,ts,tsx,mts,cts,js,jsx,mjs,cjs}"},
            {"globPattern":"**/package.json"},
            {"globPattern":"**/{pnpm-lock.yaml,pnpm-workspace.yaml,yarn.lock,bun.lock,bun.lockb}"},
            {"globPattern":"**/vite.config.*"},
            {"globPattern":"**/*.json"},
            {"globPattern":"**/jsconfig.json"}
        ]})
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

#[test]
fn output_filter_keeps_authored_and_neighboring_paths() {
    for name in [
        "node_modules/.vize/rich-hover.json",
        "node_modules/.vize/completion.json",
        "node_modules/.vize/corsa-overlay/tsconfig.json",
        "node_modules/.vize",
        ".vize/policy.json",
        "node_modules/.vize-other/package.json",
        "node_modules/example/package.json",
    ] {
        let root = tempfile::tempdir().unwrap();
        let event = FileEvent {
            uri: Url::from_file_path(root.path().join(name)).unwrap(),
            typ: FileChangeType::CHANGED,
        };
        assert_eq!(
            user_watched_file_events(std::slice::from_ref(&event)),
            if name.starts_with("node_modules/.vize/") || name == "node_modules/.vize" {
                vec![]
            } else {
                vec![event]
            },
            "{name}"
        );
    }
}

#[cfg(unix)]
#[test]
fn output_filter_checks_authored_and_physical_symlink_paths_for_absent_children() {
    let root = tempfile::tempdir().unwrap();
    let cache = root.path().join("node_modules/.vize");
    std::fs::create_dir_all(&cache).unwrap();
    let alias = root.path().join("generated");
    std::os::unix::fs::symlink(&cache, &alias).unwrap();
    let authored = root.path().join(".vize");
    std::fs::create_dir(&authored).unwrap();
    // The authored path inside the owned output tree also remains excluded
    // when a symlink points from it to a directory outside that tree.
    std::os::unix::fs::symlink(&authored, cache.join("linked")).unwrap();
    for typ in [FileChangeType::CREATED, FileChangeType::DELETED] {
        for path in [
            alias.join("missing/output.json"),
            cache.join("linked/new.json"),
        ] {
            assert!(
                user_watched_file_events(&[FileEvent {
                    uri: Url::from_file_path(path).unwrap(),
                    typ,
                }])
                .is_empty()
            );
        }
        assert_eq!(
            user_watched_file_events(&[FileEvent {
                uri: Url::from_file_path(authored.join("new.json")).unwrap(),
                typ,
            }])
            .len(),
            1
        );
    }
}

fn watched_project() -> std::io::Result<(
    tempfile::TempDir,
    tower_lsp::LspService<super::MaestroServer>,
    Url,
)> {
    let root = tempfile::tempdir()?;
    let package = root.path().join("package");
    std::fs::create_dir_all(package.join(".vize"))?;
    for (name, content) in [
        ("package.json", "{}"),
        (
            "vite.config.mjs",
            "import vue from './policy.mjs'; export default { vue };",
        ),
        (
            "policy.mjs",
            "export default { languageServer: { hover: false, lint: false, ecosystem: false, typecheck: true } };",
        ),
        (
            "tsconfig.json",
            r#"{"extends":"./.vize/options.json","include":["*.vue"]}"#,
        ),
        (
            ".vize/options.json",
            r#"{"vueCompilerOptions":{"target":2.7}}"#,
        ),
        (
            "App.vue",
            "<script setup>const total = 1</script><template>{{ total }}</template>",
        ),
    ] {
        std::fs::write(package.join(name), content)?;
    }
    let (service, _socket) = tower_lsp::LspService::new(super::MaestroServer::new);
    service.inner().state.set_workspace_root(root.path().into());
    service
        .inner()
        .state
        .set_workspace_folders(vec![root.path().into()]);
    let uri = Url::from_file_path(package.join("App.vue"))
        .map_err(|()| std::io::Error::other("watched project path must form a file URI"))?;
    Ok((root, service, uri))
}

#[test]
fn owned_json_notification_preserves_pending_semantic_response_and_project_generation() {
    use futures::{FutureExt, channel::oneshot};
    use tower_lsp::LanguageServer;
    use tower_lsp::lsp_types::{DidChangeWatchedFilesParams, SemanticTokensParams};

    let (root, service, uri) = watched_project().unwrap();
    let server = service.inner();
    let state = server.state.document_project_state(&uri).unwrap();
    let source = std::fs::read_to_string(uri.to_file_path().unwrap()).unwrap();
    state
        .documents
        .open(uri.clone(), source.clone(), 7, "vue".into());
    state.update_virtual_docs(&uri, &source);
    let project = server.with_project_state(state.clone());
    let params: SemanticTokensParams = serde_json::from_value(serde_json::json!({
        "textDocument": {"uri": uri}
    }))
    .unwrap();
    let expected = crate::runtime::block_on(project.semantic_tokens_full(params)).unwrap();
    assert!(expected.is_some());
    let generation = server.state.project_routing_generation();
    let stamp = state.corsa_request_stamp();
    let (release, held) = oneshot::channel();
    let mut response = Box::pin(project.native_request(async {
        held.await.unwrap();
        Ok(expected.clone())
    }));
    assert!(response.as_mut().now_or_never().is_none());
    crate::runtime::block_on(
        server.did_change_watched_files(DidChangeWatchedFilesParams {
            changes: vec![FileEvent {
                uri: Url::from_file_path(root.path().join("node_modules/.vize/rich-hover.json"))
                    .unwrap(),
                typ: FileChangeType::CHANGED,
            }],
        }),
    );
    assert!(stamp.is_current(&state));
    assert_eq!(server.state.project_routing_generation(), generation);
    assert!(std::ptr::eq(
        state.as_ref(),
        server.state.document_project_state(&uri).unwrap().as_ref()
    ));
    assert_eq!(state.documents.version(&uri), Some(7));
    assert_eq!(state.documents.text(&uri).as_deref(), Some(source.as_str()));
    release.send(()).unwrap();
    assert_eq!(crate::runtime::block_on(response).unwrap(), expected);
}

#[test]
fn helper_tsconfig_and_authored_dot_vize_json_notifications_still_reconfigure() {
    use tower_lsp::LanguageServer;
    use tower_lsp::lsp_types::DidChangeWatchedFilesParams;

    for (name, content) in [
        (
            "policy.mjs",
            "export default { languageServer: { hover: true, lint: false, ecosystem: false, typecheck: true } };",
        ),
        (
            "tsconfig.json",
            r#"{"vueCompilerOptions":{"target":3.5},"include":["*.vue"]}"#,
        ),
        (
            ".vize/options.json",
            r#"{"vueCompilerOptions":{"target":3.5}}"#,
        ),
    ] {
        let (root, service, uri) = watched_project().unwrap();
        let server = service.inner();
        let previous = server.state.document_project_state(&uri).unwrap();
        let stamp = previous.corsa_request_stamp();
        let generation = server.state.project_routing_generation();
        let path = root.path().join("package").join(name);
        std::fs::write(&path, content).unwrap();
        crate::runtime::block_on(
            server.did_change_watched_files(DidChangeWatchedFilesParams {
                changes: vec![FileEvent {
                    uri: Url::from_file_path(path).unwrap(),
                    typ: FileChangeType::CHANGED,
                }],
            }),
        );
        let current = server.state.document_project_state(&uri).unwrap();
        assert!(!stamp.is_current(&previous), "{name}");
        assert!(!std::ptr::eq(previous.as_ref(), current.as_ref()), "{name}");
        assert!(
            server.state.project_routing_generation() > generation,
            "{name}"
        );
        if name == "policy.mjs" {
            assert!(current.lsp_features().hover);
        } else {
            assert_eq!(
                current.type_checker_vue_version(),
                vize_l0::config::VueVersion::V3
            );
        }
    }
}
