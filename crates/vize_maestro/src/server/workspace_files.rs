//! Workspace file-event handling used by LSP diagnostics and rename support.

use tower_lsp::lsp_types::{
    ClientCapabilities, CreateFilesParams, DeleteFilesParams, DidChangeWatchedFilesParams,
    MessageType, RenameFilesParams, WorkspaceEdit,
};
#[cfg(feature = "native")]
use tower_lsp::lsp_types::{FileChangeType, FileEvent, Url};

use super::{MaestroServer, ServerState};
use crate::ide::FileRenameService;

#[cfg(feature = "native")]
mod dependents;
#[cfg(feature = "native")]
mod project_operations;
mod watched;
#[cfg(feature = "native")]
use dependents::{
    affected_vue_source_paths, forget_corsa_vue_files, include_open_typecheck_documents,
    invalidate_corsa_disk_state, versioned_open_typecheck_dependents,
};
#[cfg(all(test, feature = "native"))]
use watched::{changes_invalidate_disk_project_state, user_watched_file_events};

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
    register_typecheck_dependency_watcher(server).await;
    server
        .client
        .log_message(MessageType::INFO, "vize_maestro LSP server initialized")
        .await;
}
async fn register_typecheck_dependency_watcher(server: &MaestroServer) {
    #[cfg(feature = "native")]
    {
        if !server.state.project_source_watcher_enabled()
            || !server.state.global_component_watcher_supported()
        {
            return;
        }
        if let Err(error) = server
            .client
            .register_capability(vec![typecheck_dependency_watcher_registration()])
            .await
        {
            tracing::warn!("failed to register typecheck dependency watcher: {error}");
        }
    }
    #[cfg(not(feature = "native"))]
    let _ = server;
}

#[cfg(feature = "native")]
fn typecheck_dependency_watcher_registration() -> Registration {
    let options = DidChangeWatchedFilesRegistrationOptions {
        watchers: [
            "**/*.d.{ts,mts,cts}",
            "**/*.{vue,ts,tsx,mts,cts,js,jsx,mjs,cjs}",
            "**/package.json",
            "**/vize.config.*",
            "**/vite.config.*",
            "**/tsconfig*.json",
            "**/jsconfig.json",
        ]
        .into_iter()
        .map(|pattern| FileSystemWatcher {
            glob_pattern: GlobPattern::String(pattern.into()),
            kind: None,
        })
        .collect(),
    };
    Registration {
        id: "vize-typecheck-dependencies".into(),
        method: "workspace/didChangeWatchedFiles".into(),
        register_options: serde_json::to_value(options).ok(),
    }
}

pub(super) async fn did_change_watched_files(
    server: &MaestroServer,
    params: &DidChangeWatchedFilesParams,
) {
    watched::did_change_watched_files(server, params).await;
}

#[cfg(feature = "native")]
pub(super) async fn invalidate_changed_document_disk_project_state(
    server: &MaestroServer,
    uri: &tower_lsp::lsp_types::Url,
) {
    if watched::changes_invalidate_disk_project_state(
        &server.state,
        &[FileEvent {
            uri: uri.clone(),
            typ: FileChangeType::CHANGED,
        }],
    ) {
        invalidate_corsa_disk_state(&server.state);
    }
}

pub(super) async fn did_create_files(server: &MaestroServer, params: &CreateFilesParams) {
    #[cfg(feature = "experimental-source-navigation")]
    server
        .state
        .observe_module_target_file_events(!params.files.is_empty());
    #[cfg(feature = "native")]
    project_operations::did_create_files(server, params).await;
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
    #[cfg(feature = "experimental-source-navigation")]
    server
        .state
        .observe_module_target_file_events(!params.files.is_empty());
    #[cfg(feature = "native")]
    project_operations::did_delete_files(server, params).await;
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
    server: &MaestroServer,
    params: &RenameFilesParams,
) -> tower_lsp::jsonrpc::Result<Option<WorkspaceEdit>> {
    #[cfg(feature = "native")]
    return server
        .project_request(FileRenameService::will_rename_project_files(
            &server.state,
            params,
        ))
        .await;
    #[cfg(not(feature = "native"))]
    server
        .native_request(async {
            if !server.state.lsp_features().file_rename {
                return Ok(None);
            }
            Ok(FileRenameService::will_rename_files(&server.state, params).await)
        })
        .await
}

pub(super) async fn did_rename_files(server: &MaestroServer, params: &RenameFilesParams) {
    #[cfg(feature = "experimental-source-navigation")]
    server
        .state
        .observe_module_target_file_events(!params.files.is_empty());
    #[cfg(feature = "native")]
    project_operations::did_rename_files(server, params).await;
    #[cfg(not(feature = "native"))]
    {
        if !server.state.lsp_features().file_rename {
            return;
        }
        for (old, new) in FileRenameService::did_rename_files(&server.state, params).await {
            server.client.publish_diagnostics(old, vec![], None).await;
            server.publish_diagnostics(&new).await;
        }
    }
}

#[cfg(feature = "native")]
async fn publish_versioned_dependents(
    server: &MaestroServer,
    dependents: Vec<(tower_lsp::lsp_types::Url, i32)>,
) {
    for (dependent, version) in dependents {
        server
            .for_document(&dependent)
            .await
            .publish_diagnostics_if_version(&dependent, version)
            .await;
    }
}

#[cfg(all(test, feature = "native"))]
#[path = "workspace_files_tests.rs"]
mod tests;
