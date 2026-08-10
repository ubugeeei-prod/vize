//! Workspace file-event handling used by LSP diagnostics and rename support.

use tower_lsp::lsp_types::{
    ClientCapabilities, CreateFilesParams, DeleteFilesParams, DidChangeWatchedFilesParams,
    MessageType, RenameFilesParams, Url, WorkspaceEdit,
};
use vize_carton::cstr;

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
        // A change to a discoverable global-component declaration invalidates
        // that cache — but any watched change can break open importers (a git
        // checkout rewriting a child SFC's props, a codegen run, a delete), so
        // the dependent refresh below is not gated on it (#3918).
        let global_components_invalidated = server.state.invalidate_global_component_references(
            params.changes.iter().map(|change| change.uri.as_str()),
        );

        refresh_dependents(
            server,
            params
                .changes
                .iter()
                .map(|change| change.uri.clone())
                .collect(),
            global_components_invalidated,
        )
        .await;
    }
    #[cfg(not(feature = "native"))]
    let _ = (server, params);
}

pub(super) async fn did_create_files(server: &MaestroServer, params: &CreateFilesParams) {
    #[cfg(feature = "native")]
    {
        let invalidated = register_created_files(&server.state, params);
        refresh_dependents(server, file_event_uris(params), invalidated).await;
    }
    #[cfg(not(feature = "native"))]
    let _ = (server, params);
}

pub(super) async fn did_delete_files(server: &MaestroServer, params: &DeleteFilesParams) {
    #[cfg(feature = "native")]
    {
        let invalidated = register_deleted_files(&server.state, params);
        let dependencies = file_event_uris(params);
        close_vue_dependency_documents(server, &dependencies).await;
        refresh_dependents(server, dependencies, invalidated).await;
    }
    #[cfg(not(feature = "native"))]
    let _ = (server, params);
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
        let invalidated = server.state.invalidate_global_component_references(
            params
                .files
                .iter()
                .flat_map(|file| [file.old_uri.as_str(), file.new_uri.as_str()]),
        );
        for file in &params.files {
            server
                .state
                .forget_workspace_vue_file(file.old_uri.as_str());
            server.state.track_workspace_vue_file(file.new_uri.as_str());
        }
        let dependencies = params
            .files
            .iter()
            .flat_map(|file| [&file.old_uri, &file.new_uri])
            .filter_map(|uri| Url::parse(uri).ok())
            .collect();
        close_vue_dependency_documents(
            server,
            &params
                .files
                .iter()
                .filter_map(|file| Url::parse(&file.old_uri).ok())
                .collect::<Vec<_>>(),
        )
        .await;
        refresh_dependents(server, dependencies, invalidated).await;
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

#[cfg(feature = "native")]
fn register_created_files(state: &ServerState, params: &CreateFilesParams) -> bool {
    let invalidated = state
        .invalidate_global_component_references(params.files.iter().map(|file| file.uri.as_str()));
    for file in &params.files {
        state.track_workspace_vue_file(file.uri.as_str());
    }
    invalidated
}

#[cfg(feature = "native")]
fn register_deleted_files(state: &ServerState, params: &DeleteFilesParams) -> bool {
    let invalidated = state
        .invalidate_global_component_references(params.files.iter().map(|file| file.uri.as_str()));
    for file in &params.files {
        state.forget_workspace_vue_file(file.uri.as_str());
    }
    invalidated
}

#[cfg(feature = "native")]
fn file_event_uris<T>(params: &T) -> Vec<Url>
where
    T: FileEventParams,
{
    params
        .uris()
        .filter_map(|uri| Url::parse(uri).ok())
        .collect()
}

#[cfg(feature = "native")]
trait FileEventParams {
    fn uris(&self) -> impl Iterator<Item = &str>;
}

#[cfg(feature = "native")]
impl FileEventParams for CreateFilesParams {
    fn uris(&self) -> impl Iterator<Item = &str> {
        self.files.iter().map(|file| file.uri.as_str())
    }
}

#[cfg(feature = "native")]
impl FileEventParams for DeleteFilesParams {
    fn uris(&self) -> impl Iterator<Item = &str> {
        self.files.iter().map(|file| file.uri.as_str())
    }
}

#[cfg(feature = "native")]
async fn refresh_dependents(
    server: &MaestroServer,
    dependencies: Vec<Url>,
    force_invalidate: bool,
) {
    let mut dependents = dependencies
        .iter()
        .flat_map(|dependency| {
            super::importers::open_typecheck_dependents(&server.state, dependency)
        })
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
    if dependents.is_empty() && !force_invalidate {
        return;
    }
    server.state.invalidate_batch_cache();
    for (dependent, version) in dependents {
        server
            .publish_diagnostics_if_version(&dependent, version)
            .await;
    }
}

#[cfg(feature = "native")]
async fn close_vue_dependency_documents(server: &MaestroServer, dependencies: &[Url]) {
    if !server.state.has_corsa_bridge() {
        return;
    }
    let Some(bridge) = server.state.get_corsa_bridge().await else {
        return;
    };
    for source_path in dependencies
        .iter()
        .filter_map(|uri| uri.to_file_path().ok())
    {
        if source_path
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("vue")
        {
            continue;
        }
        let Some(file_name) = source_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        for suffix in [".ts", ".tsx"] {
            let virtual_path = source_path.with_file_name(cstr!("{file_name}{suffix}"));
            if let Ok(uri) = Url::from_file_path(virtual_path) {
                let _ = bridge.close_virtual_document(uri.as_str()).await;
            }
        }
    }
}

#[cfg(all(test, feature = "native"))]
#[path = "workspace_files_tests.rs"]
mod tests;
