//! One authored importer scan resolves each source with its current project.
#![expect(
    clippy::disallowed_types,
    reason = "bulk rename owns cached project views and LSP edit maps"
)]
mod deduplicate;

use super::{FileRenameService, ServerState, manual, merge_workspace_edits};
use std::sync::Arc;
use tower_lsp::{
    jsonrpc::{Error, Result},
    lsp_types::{DocumentChangeOperation, DocumentChanges, RenameFilesParams, Url, WorkspaceEdit},
};

impl FileRenameService {
    pub(crate) async fn will_rename_project_files(
        owner: &Arc<ServerState>,
        params: &RenameFilesParams,
    ) -> Result<Option<WorkspaceEdit>> {
        let mut states = owner.current_project_states();
        states.sort_by_key(|state| state.get_workspace_root());
        let native = futures::future::try_join_all(states.into_iter().map(|state| async move {
            if !state.lsp_features().file_rename {
                return Ok(None);
            }
            let scope = state.corsa_request_scope().await;
            let mut native = Self::corsa_workspace_edit(&state, params).await;
            if !scope.is_current() {
                return Err(Error::content_modified());
            }
            if let Some(native) = &mut native {
                retain_owned_edits(native, |uri| owns_uri(owner, &state, uri));
            }
            Ok(native)
        }))
        .await?;
        let mut edit = None;
        for native in native {
            edit = merge_workspace_edits(edit, native);
        }
        let manual = manual::collect_project_import_rename_edits(owner, &params.files);
        let mut edit = merge_workspace_edits(edit, manual);
        if let Some(edit) = &mut edit {
            deduplicate::edits(edit);
        }
        Ok(edit)
    }

    pub(crate) fn did_rename_project_files(
        owner: &Arc<ServerState>,
        params: &RenameFilesParams,
    ) -> Vec<(Url, Url)> {
        manual::rename_project_open_documents(owner, &params.files)
    }
}

fn owns_uri(owner: &Arc<ServerState>, state: &Arc<ServerState>, uri: &Url) -> bool {
    let current = owner
        .document_project_state(uri)
        .unwrap_or_else(|| owner.current_primary_project_state());
    current.lsp_features().file_rename && Arc::ptr_eq(&current, state)
}

fn retain_owned_edits(edit: &mut WorkspaceEdit, mut owns: impl FnMut(&Url) -> bool) {
    if let Some(changes) = &mut edit.changes {
        changes.retain(|uri, _| owns(uri));
    }
    match &mut edit.document_changes {
        Some(DocumentChanges::Edits(edits)) => edits.retain(|edit| owns(&edit.text_document.uri)),
        Some(DocumentChanges::Operations(operations)) => {
            operations.retain(|operation| match operation {
                DocumentChangeOperation::Edit(edit) => owns(&edit.text_document.uri),
                DocumentChangeOperation::Op(_) => false,
            })
        }
        None => {}
    }
}
