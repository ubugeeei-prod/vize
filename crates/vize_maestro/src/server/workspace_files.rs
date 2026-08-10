//! Workspace file-event handling used by LSP diagnostics and rename support.

#[cfg(feature = "native")]
use tower_lsp::lsp_types::FileChangeType;
use tower_lsp::lsp_types::{
    ClientCapabilities, CreateFilesParams, DeleteFilesParams, DidChangeWatchedFilesParams,
    MessageType, RenameFilesParams, WorkspaceEdit,
};

use super::{MaestroServer, ServerState};
use crate::ide::FileRenameService;

#[cfg(feature = "native")]
mod dependents;
#[cfg(feature = "native")]
use dependents::{
    affected_vue_source_paths, forget_corsa_vue_files, versioned_open_vue_dependents,
};

#[cfg(feature = "native")]
use tower_lsp::lsp_types::{
    DidChangeWatchedFilesRegistrationOptions, FileSystemWatcher, GlobPattern, Registration,
};

pub(super) fn record_watcher_support(state: &ServerState, capabilities: &ClientCapabilities) {
    #[cfg(feature = "native")]
    state.set_global_component_watcher_supported(
        capabilities
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.did_change_watched_files.as_ref())
            .and_then(|watched| watched.dynamic_registration)
            .unwrap_or(false),
    );
    #[cfg(not(feature = "native"))]
    let _ = (state, capabilities);
}

pub(super) async fn initialized(server: &MaestroServer) {
    register_global_component_watcher(server).await;
    server
        .client
        .log_message(MessageType::INFO, "vize_maestro LSP server initialized")
        .await;
}

async fn register_global_component_watcher(server: &MaestroServer) {
    #[cfg(feature = "native")]
    {
        if !server.state.is_lsp_typecheck_enabled()
            || !server.state.global_component_watcher_supported()
        {
            return;
        }
        if let Err(error) = server
            .client
            .register_capability(vec![global_component_watcher_registration()])
            .await
        {
            tracing::warn!("failed to register global component declaration watcher: {error}");
        }
    }
    #[cfg(not(feature = "native"))]
    let _ = server;
}

#[cfg(feature = "native")]
fn global_component_watcher_registration() -> Registration {
    let options = DidChangeWatchedFilesRegistrationOptions {
        watchers: vec![FileSystemWatcher {
            glob_pattern: GlobPattern::String("**/*.d.{ts,mts,cts}".into()),
            kind: None,
        }],
    };
    Registration {
        id: "vize-global-component-declarations".into(),
        method: "workspace/didChangeWatchedFiles".into(),
        register_options: serde_json::to_value(options).ok(),
    }
}

pub(super) async fn did_change_watched_files(
    server: &MaestroServer,
    params: &DidChangeWatchedFilesParams,
) {
    #[cfg(feature = "native")]
    {
        // A change to a discoverable global-component declaration invalidates
        // that cache — but any watched change can break open importers (a git
        // checkout rewriting a child SFC's props, a codegen run, a delete), so
        // the dependent refresh below is not gated on it (#3918).
        let global_components_invalidated = server.state.invalidate_global_component_references(
            params.changes.iter().map(|change| change.uri.as_str()),
        );

        let dependents = versioned_open_vue_dependents(
            &server.state,
            params.changes.iter().map(|change| change.uri.as_str()),
        );
        let deleted_paths = affected_vue_source_paths(
            &server.state,
            params
                .changes
                .iter()
                .filter(|change| change.typ == FileChangeType::DELETED)
                .map(|change| change.uri.as_str()),
        );
        if dependents.is_empty() && !global_components_invalidated && deleted_paths.is_empty() {
            return;
        }
        // Recomputation must read the changed files fresh from disk.
        server.state.invalidate_batch_cache();
        forget_corsa_vue_files(&server.state, &deleted_paths).await;
        for (dependent, version) in dependents {
            server
                .publish_diagnostics_if_version(&dependent, version)
                .await;
        }
    }
    #[cfg(not(feature = "native"))]
    let _ = (server, params);
}

pub(super) async fn did_create_files(server: &MaestroServer, params: &CreateFilesParams) {
    #[cfg(feature = "native")]
    {
        let dependents = versioned_open_vue_dependents(
            &server.state,
            params.files.iter().map(|file| file.uri.as_str()),
        );
        record_created_files(&server.state, params);
        for (dependent, version) in dependents {
            server
                .publish_diagnostics_if_version(&dependent, version)
                .await;
        }
    }
    #[cfg(not(feature = "native"))]
    let _ = (server, params);
}

#[cfg(any(test, feature = "native"))]
fn record_created_files(state: &ServerState, params: &CreateFilesParams) {
    #[cfg(feature = "native")]
    {
        state.invalidate_global_component_references(
            params.files.iter().map(|file| file.uri.as_str()),
        );
        for file in &params.files {
            state.track_workspace_vue_files(file.uri.as_str());
        }
        state.invalidate_batch_cache();
    }
    #[cfg(not(feature = "native"))]
    let _ = (state, params);
}

pub(super) async fn did_delete_files(server: &MaestroServer, params: &DeleteFilesParams) {
    #[cfg(feature = "native")]
    {
        let dependents = versioned_open_vue_dependents(
            &server.state,
            params.files.iter().map(|file| file.uri.as_str()),
        );
        record_deleted_files(&server.state, params);
        let deleted_paths = affected_vue_source_paths(
            &server.state,
            params.files.iter().map(|file| file.uri.as_str()),
        );
        forget_corsa_vue_files(&server.state, &deleted_paths).await;
        for (dependent, version) in dependents {
            server
                .publish_diagnostics_if_version(&dependent, version)
                .await;
        }
    }
    #[cfg(not(feature = "native"))]
    let _ = (server, params);
}

#[cfg(any(test, feature = "native"))]
fn record_deleted_files(state: &ServerState, params: &DeleteFilesParams) {
    #[cfg(feature = "native")]
    {
        state.invalidate_global_component_references(
            params.files.iter().map(|file| file.uri.as_str()),
        );
        for file in &params.files {
            state.forget_workspace_vue_files(file.uri.as_str());
        }
        state.invalidate_batch_cache();
    }
    #[cfg(not(feature = "native"))]
    let _ = (state, params);
}

pub(super) async fn will_rename_files(
    state: &ServerState,
    params: &RenameFilesParams,
) -> Option<WorkspaceEdit> {
    if !state.lsp_features().file_rename {
        return None;
    }
    FileRenameService::will_rename_files(state, params).await
}

pub(super) async fn did_rename_files(server: &MaestroServer, params: &RenameFilesParams) {
    #[cfg(feature = "native")]
    {
        let dependents = versioned_open_vue_dependents(
            &server.state,
            params
                .files
                .iter()
                .flat_map(|file| [file.old_uri.as_str(), file.new_uri.as_str()]),
        );
        server.state.invalidate_global_component_references(
            params
                .files
                .iter()
                .flat_map(|file| [file.old_uri.as_str(), file.new_uri.as_str()]),
        );
        for file in &params.files {
            server
                .state
                .forget_workspace_vue_files(file.old_uri.as_str());
            server
                .state
                .track_workspace_vue_files(file.new_uri.as_str());
        }
        server.state.invalidate_batch_cache();
        let renamed_paths = affected_vue_source_paths(
            &server.state,
            params.files.iter().map(|file| file.old_uri.as_str()),
        );
        forget_corsa_vue_files(&server.state, &renamed_paths).await;
        for (dependent, version) in dependents {
            server
                .publish_diagnostics_if_version(&dependent, version)
                .await;
        }
    }
    if !server.state.lsp_features().file_rename {
        return;
    }

    let renamed = FileRenameService::did_rename_files(&server.state, params).await;
    for (old_uri, new_uri) in renamed {
        server
            .client
            .publish_diagnostics(old_uri, vec![], None)
            .await;
        server.publish_diagnostics(&new_uri).await;
    }
}

#[cfg(all(test, feature = "native"))]
mod tests {
    use tower_lsp::lsp_types::{ClientCapabilities, CreateFilesParams, DeleteFilesParams, Url};

    use super::{
        ServerState, global_component_watcher_registration, record_created_files,
        record_deleted_files, record_watcher_support,
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
}
