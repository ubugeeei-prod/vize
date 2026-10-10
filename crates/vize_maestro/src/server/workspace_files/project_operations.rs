//! File notifications invalidate current owners and move shared buffers once.
use super::{
    CreateFilesParams, DeleteFilesParams, FileRenameService, MaestroServer, RenameFilesParams,
    ServerState, affected_vue_source_paths, forget_corsa_vue_files,
    include_open_typecheck_documents, invalidate_corsa_disk_state, publish_versioned_dependents,
    record_created_files, record_deleted_files, versioned_open_typecheck_dependents,
};

pub(super) async fn did_create_files(server: &MaestroServer, params: &CreateFilesParams) {
    let states = server.state.current_project_states();
    let mut dependents = Vec::new();
    // The registered workspace inventory is a shared lightweight controller;
    // it remains valid across primary runtime retirement.
    for state in states {
        dependents.extend(versioned_open_typecheck_dependents(
            &state,
            params.files.iter().map(|file| file.uri.as_str()),
        ));
        // Structural changes can satisfy an unresolved import which has no
        // reverse edge yet. Keep the fallback bounded by editor-open files.
        dependents = include_open_typecheck_documents(&state, dependents);
        record_created_files(&state, params);
        invalidate_corsa_disk_state(&state);
    }
    record_created_files(&server.state, params);
    dependents.sort();
    dependents.dedup();
    publish_versioned_dependents(server, dependents).await;
}

pub(super) async fn did_delete_files(server: &MaestroServer, params: &DeleteFilesParams) {
    let states = server.state.current_project_states();
    let mut dependents = Vec::new();
    for state in states {
        let mut affected = versioned_open_typecheck_dependents(
            &state,
            params.files.iter().map(|file| file.uri.as_str()),
        );
        let deleted =
            affected_vue_source_paths(&state, params.files.iter().map(|file| file.uri.as_str()));
        affected = include_open_typecheck_documents(&state, affected);
        dependents.extend(affected);
        record_deleted_files(&state, params);
        forget_corsa_vue_files(&state, &deleted);
        invalidate_corsa_disk_state(&state);
    }
    record_deleted_files(&server.state, params);
    dependents.sort();
    dependents.dedup();
    publish_versioned_dependents(server, dependents).await;
}

pub(super) async fn did_rename_files(server: &MaestroServer, params: &RenameFilesParams) {
    let states = server.state.current_project_states();
    let mut dependents = Vec::new();
    for state in states {
        dependents.extend(versioned_open_typecheck_dependents(
            &state,
            params
                .files
                .iter()
                .flat_map(|file| [file.old_uri.as_str(), file.new_uri.as_str()]),
        ));
        let removed = affected_vue_source_paths(
            &state,
            params.files.iter().map(|file| file.old_uri.as_str()),
        );
        record_renames(&state, params);
        forget_corsa_vue_files(&state, &removed);
        invalidate_corsa_disk_state(&state);
    }
    record_renames(&server.state, params);
    let renamed = FileRenameService::did_rename_project_files(&server.state, params);
    dependents.sort();
    dependents.dedup();
    publish_versioned_dependents(server, dependents).await;
    for (old, new) in renamed {
        server.client.publish_diagnostics(old, vec![], None).await;
        if let Some(version) = server.state.documents.version(&new) {
            server
                .for_document(&new)
                .publish_diagnostics_if_version(&new, version)
                .await;
        }
    }
}

fn record_renames(state: &ServerState, params: &RenameFilesParams) {
    state.invalidate_global_component_references(
        params
            .files
            .iter()
            .flat_map(|file| [file.old_uri.as_str(), file.new_uri.as_str()]),
    );
    for file in &params.files {
        state.forget_workspace_vue_files(file.old_uri.as_str());
        state.track_workspace_vue_files(file.new_uri.as_str());
    }
    state.invalidate_batch_cache();
}
