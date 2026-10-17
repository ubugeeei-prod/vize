//! Watched files invalidate each initialized project without importing idle packages.

use super::{DidChangeWatchedFilesParams, MaestroServer};
#[cfg(feature = "native")]
use super::{
    FileChangeType, FileEvent, ServerState, Url, affected_vue_source_paths, forget_corsa_vue_files,
    include_open_typecheck_documents, invalidate_corsa_disk_state, publish_versioned_dependents,
    versioned_open_typecheck_dependents,
};

pub(super) async fn did_change_watched_files(
    server: &MaestroServer,
    params: &DidChangeWatchedFilesParams,
) {
    #[cfg(feature = "native")]
    super::super::native_requests::trace_watched_file_events(params);
    // Filter before routing, navigation, or native request stamps can move.
    // Output written under our cache must not feed back into authored inputs.
    #[cfg(feature = "native")]
    let params = &DidChangeWatchedFilesParams {
        changes: user_watched_file_events(&params.changes),
    };
    if params.changes.is_empty() {
        return;
    }
    #[cfg(feature = "native")]
    let reconfigured = server
        .state
        .observe_project_config_events_async(params.changes.clone())
        .await;
    #[cfg(feature = "native")]
    server.queue_configuration_watches();
    #[cfg(feature = "native")]
    for error in &reconfigured.errors {
        server
            .client
            .show_message(error.typ, error.message.clone())
            .await;
    }
    #[cfg(feature = "native")]
    let operation = server.with_configuration_admission(&reconfigured.checked_roots);
    #[cfg(feature = "native")]
    let server = &operation;
    #[cfg(all(feature = "native", feature = "experimental-source-navigation"))]
    if server.state.project_context_retired()
        && let Some(navigation) = &server.navigation
    {
        navigation.retire();
    }
    #[cfg(feature = "native")]
    let accepted = DidChangeWatchedFilesParams {
        changes: reconfigured.accepted_file_events(&server.state, &params.changes),
    };
    #[cfg(feature = "native")]
    did_change_in_context(server, &accepted).await;
    #[cfg(not(feature = "native"))]
    did_change_in_context(server, params).await;
    #[cfg(feature = "native")]
    for state in server.state.cached_project_states() {
        let project = server.with_project_state(state);
        let accepted = DidChangeWatchedFilesParams {
            changes: reconfigured.accepted_file_events(&project.state, &params.changes),
        };
        did_change_in_context(&project, &accepted).await;
    }
    #[cfg(feature = "native")]
    for (uri, version) in reconfigured.documents {
        let project = server.for_document_admitted(&uri).await;
        if project.state.documents.version(&uri) == Some(version)
            && let Some(source) = project.state.documents.text(&uri)
        {
            project.state.update_virtual_docs(&uri, &source);
        }
        project
            .publish_configuration_diagnostics_if_version(&uri, version)
            .await;
    }
}

async fn did_change_in_context(server: &MaestroServer, params: &DidChangeWatchedFilesParams) {
    // A rejected configuration event can leave one owner with no accepted
    // changes even when the original notification was nonempty.
    if params.changes.is_empty() {
        return;
    }
    #[cfg(feature = "experimental-source-navigation")]
    server
        .state
        .observe_module_target_file_events(!params.changes.is_empty());
    #[cfg(feature = "native")]
    {
        let changes = &params.changes;
        server.state.observe_workspace_project_file_events(changes);
        server.state.invalidate_component_interfaces();
        if changes_invalidate_disk_project_state(&server.state, changes) {
            invalidate_corsa_disk_state(&server.state);
        }
        // Any watched change can affect an open importer; declaration changes
        // additionally invalidate the discoverable global-component cache.
        let global_components_invalidated = server.state.invalidate_global_component_references(
            changes.iter().map(|change| change.uri.as_str()),
        );
        let mut dependents = versioned_open_typecheck_dependents(
            &server.state,
            changes.iter().map(|change| change.uri.as_str()),
        );
        let deleted_paths = affected_vue_source_paths(
            &server.state,
            changes
                .iter()
                .filter(|change| change.typ == FileChangeType::DELETED)
                .map(|change| change.uri.as_str()),
        );
        if !deleted_paths.is_empty() {
            dependents = include_open_typecheck_documents(&server.state, dependents);
        }
        if dependents.is_empty() && !global_components_invalidated && deleted_paths.is_empty() {
            return;
        }
        server.state.invalidate_batch_cache();
        forget_corsa_vue_files(&server.state, &deleted_paths);
        publish_versioned_dependents(server, dependents).await;
    }
    #[cfg(not(feature = "native"))]
    let _ = (server, params);
}

#[cfg(feature = "native")]
pub(super) fn user_watched_file_events(changes: &[FileEvent]) -> Vec<FileEvent> {
    changes
        .iter()
        .filter(|change| !is_internal_output_uri(&change.uri))
        .cloned()
        .collect()
}

#[cfg(feature = "native")]
fn is_internal_output_uri(uri: &Url) -> bool {
    let Ok(path) = uri.to_file_path() else {
        return false;
    };
    if is_internal_output_path(&path) {
        return true;
    }
    // Clients can watch a symlink alias, including a newly created or deleted
    // child. Resolve its nearest existing ancestor without requiring the child
    // itself to exist. Keep the authored spelling check above as well.
    for ancestor in path.ancestors() {
        if let Ok(physical) = ancestor.canonicalize()
            && let Ok(relative) = path.strip_prefix(ancestor)
        {
            return is_internal_output_path(&physical.join(relative));
        }
    }
    false
}

#[cfg(feature = "native")]
fn is_internal_output_path(path: &std::path::Path) -> bool {
    path.components()
        .zip(path.components().skip(1))
        .any(|(parent, child)| parent.as_os_str() == "node_modules" && child.as_os_str() == ".vize")
}

/// Whether watched changes moved project state the type checker only sees on disk.
///
/// Editing an open `.vue` source reaches the checker through its synchronized
/// virtual document, so treating those edits as disk changes would retire the
/// reusable editor session on every save. Changed closed `.vue` files are only
/// visible on disk and must invalidate cached project state just like
/// declaration, manifest, and configuration changes.
#[cfg(feature = "native")]
pub(super) fn changes_invalidate_disk_project_state(
    state: &ServerState,
    changes: &[FileEvent],
) -> bool {
    changes.iter().any(|change| {
        change.typ != FileChangeType::CHANGED
            || !change.uri.as_str().ends_with(".vue")
            || state.documents.version(&change.uri).is_none()
    })
}
