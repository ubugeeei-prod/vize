//! Workspace file-event handling used by LSP diagnostics and rename support.

use tower_lsp::lsp_types::{
    ClientCapabilities, CreateFilesParams, DeleteFilesParams, DidChangeWatchedFilesParams,
    MessageType, RenameFilesParams, WorkspaceEdit,
};

use super::{MaestroServer, ServerState};
use crate::ide::FileRenameService;

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
        if !server.state.invalidate_global_component_references(
            params.changes.iter().map(|change| change.uri.as_str()),
        ) {
            return;
        }

        let mut dependents = params
            .changes
            .iter()
            .flat_map(|change| super::importers::open_vue_dependents(&server.state, &change.uri))
            .collect::<Vec<_>>();
        dependents.sort();
        dependents.dedup();
        let dependents = dependents
            .into_iter()
            .filter_map(|uri| {
                server
                    .state
                    .documents
                    .version(&uri)
                    .map(|version| (uri, version))
            })
            .collect::<Vec<_>>();
        for (dependent, version) in dependents {
            server
                .publish_diagnostics_if_version(&dependent, version)
                .await;
        }
    }
    #[cfg(not(feature = "native"))]
    let _ = (server, params);
}

pub(super) fn did_create_files(state: &ServerState, params: &CreateFilesParams) {
    #[cfg(feature = "native")]
    state.invalidate_global_component_references(params.files.iter().map(|file| file.uri.as_str()));
    #[cfg(not(feature = "native"))]
    let _ = (state, params);
}

pub(super) fn did_delete_files(state: &ServerState, params: &DeleteFilesParams) {
    #[cfg(feature = "native")]
    state.invalidate_global_component_references(params.files.iter().map(|file| file.uri.as_str()));
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
    server.state.invalidate_global_component_references(
        params
            .files
            .iter()
            .flat_map(|file| [file.old_uri.as_str(), file.new_uri.as_str()]),
    );
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
    use tower_lsp::lsp_types::ClientCapabilities;

    use super::{ServerState, global_component_watcher_registration, record_watcher_support};

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
}
