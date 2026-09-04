//! File rename support for workspace import updates.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

mod alias;
#[cfg(all(test, feature = "native"))]
mod alias_tests;
#[cfg(all(test, feature = "native"))]
mod declaration_index_tests;
mod manual;
#[cfg(all(test, feature = "native"))]
mod manual_tests;
#[cfg(test)]
mod merge_tests;
mod resources;
#[cfg(all(test, feature = "native"))]
mod source_alias_tests;

use tower_lsp::lsp_types::{
    DocumentChangeOperation, DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier,
    RenameFilesParams, TextDocumentEdit, TextEdit, Url, WorkspaceEdit,
};

use crate::server::ServerState;

/// File rename service for import-path updates and open document state moves.
pub struct FileRenameService;

impl FileRenameService {
    /// Compute workspace edits before a file rename so clients like Neovim can apply them.
    pub async fn will_rename_files(
        state: &ServerState,
        params: &RenameFilesParams,
    ) -> Option<WorkspaceEdit> {
        let corsa_edit = Self::corsa_workspace_edit(state, params).await;
        let manual_edit =
            manual::collect_import_rename_edits(state, &params.files, corsa_edit.is_some());

        merge_workspace_edits(corsa_edit, manual_edit)
    }

    /// Update in-memory state after files were renamed on disk.
    pub async fn did_rename_files(
        state: &ServerState,
        params: &RenameFilesParams,
    ) -> Vec<(Url, Url)> {
        let renamed = manual::rename_open_documents(state, &params.files);

        #[cfg(feature = "native")]
        {
            state.invalidate_batch_cache();

            if !renamed.is_empty()
                && state.has_corsa_bridge()
                && let Some(bridge) = state.get_corsa_bridge().await
            {
                for (old_uri, _) in &renamed {
                    for request_path in [
                        crate::ide::corsa_support::template_request_path(old_uri),
                        crate::ide::corsa_support::script_request_path(old_uri, false),
                        crate::ide::corsa_support::script_request_path(old_uri, true),
                    ] {
                        let request_uri =
                            crate::ide::corsa_support::request_file_uri(&request_path);
                        let _ = bridge.close_virtual_document(&request_uri).await;
                    }
                }
            }
        }

        renamed
    }

    #[cfg(feature = "native")]
    async fn corsa_workspace_edit(
        state: &ServerState,
        params: &RenameFilesParams,
    ) -> Option<WorkspaceEdit> {
        if !state.has_corsa_bridge() {
            return None;
        }

        let bridge = state.get_corsa_bridge().await?;
        let renames = params
            .files
            .iter()
            .map(|file| (file.old_uri.as_str(), file.new_uri.as_str()))
            .collect::<Vec<_>>();

        let edit = match bridge.will_rename_files(&renames).await {
            Ok(Some(edit)) => edit,
            Ok(None) => {
                tracing::debug!(
                    rename_count = renames.len(),
                    "native tsgo file-rename request unavailable; falling back to import scanner"
                );
                return None;
            }
            Err(error) => {
                tracing::warn!(
                    rename_count = renames.len(),
                    error = %error,
                    "native tsgo file-rename request failed; falling back to import scanner"
                );
                return None;
            }
        };

        serde_json::from_value(edit).ok()
    }

    #[cfg(not(feature = "native"))]
    async fn corsa_workspace_edit(
        _state: &ServerState,
        _params: &RenameFilesParams,
    ) -> Option<WorkspaceEdit> {
        None
    }
}

pub(super) fn merge_workspace_edits(
    base: Option<WorkspaceEdit>,
    overlay: Option<WorkspaceEdit>,
) -> Option<WorkspaceEdit> {
    match (base, overlay) {
        (None, None) => None,
        (Some(edit), None) | (None, Some(edit)) => Some(edit),
        (Some(mut base), Some(mut overlay)) => {
            merge_change_sets(&mut base, &mut overlay);

            if let Some(overlay_annotations) = overlay.change_annotations.take() {
                base.change_annotations
                    .get_or_insert_with(std::collections::HashMap::new)
                    .extend(overlay_annotations);
            }

            Some(base)
        }
    }
}

fn merge_change_sets(base: &mut WorkspaceEdit, overlay: &mut WorkspaceEdit) {
    let base_document_changes = base.document_changes.take();
    let base_changes = base.changes.take();
    let overlay_document_changes = overlay.document_changes.take();
    let overlay_changes = overlay.changes.take();

    if base_document_changes.is_some() || overlay_document_changes.is_some() {
        let mut merged = merge_document_edits(
            base_document_changes,
            base_changes.map(changes_to_document_edits),
        );
        merged = merge_document_change_sets(merged, overlay_document_changes);
        merged = merge_document_edits(merged, overlay_changes.map(changes_to_document_edits));

        base.document_changes = merged;
        base.changes = None;
        return;
    }

    let mut merged_changes = base_changes.unwrap_or_default();
    if let Some(overlay_changes) = overlay_changes {
        for (uri, edits) in overlay_changes {
            merged_changes.entry(uri).or_default().extend(edits);
        }
    }

    base.changes = if merged_changes.is_empty() {
        None
    } else {
        Some(merged_changes)
    };
}

fn merge_document_change_sets(
    current: Option<DocumentChanges>,
    additional: Option<DocumentChanges>,
) -> Option<DocumentChanges> {
    let Some(additional) = additional else {
        return current;
    };

    match additional {
        DocumentChanges::Edits(edits) => merge_document_edits(current, Some(edits)),
        DocumentChanges::Operations(mut operations) => match current {
            Some(DocumentChanges::Operations(mut current_operations)) => {
                current_operations.extend(operations);
                Some(DocumentChanges::Operations(current_operations))
            }
            Some(DocumentChanges::Edits(current_edits)) => {
                insert_edit_operations(&mut operations, current_edits);
                Some(DocumentChanges::Operations(operations))
            }
            None => Some(DocumentChanges::Operations(operations)),
        },
    }
}

fn merge_document_edits(
    current: Option<DocumentChanges>,
    additional_edits: Option<Vec<TextDocumentEdit>>,
) -> Option<DocumentChanges> {
    let Some(additional_edits) = additional_edits else {
        return current;
    };

    if additional_edits.is_empty() {
        return current;
    }

    match current {
        Some(DocumentChanges::Edits(mut edits)) => {
            edits.extend(additional_edits);
            Some(DocumentChanges::Edits(edits))
        }
        Some(DocumentChanges::Operations(mut operations)) => {
            insert_edit_operations(&mut operations, additional_edits);
            Some(DocumentChanges::Operations(operations))
        }
        None => Some(DocumentChanges::Edits(additional_edits)),
    }
}

fn insert_edit_operations(
    operations: &mut Vec<DocumentChangeOperation>,
    edits: Vec<TextDocumentEdit>,
) {
    let insert_at = operations
        .iter()
        .position(|operation| matches!(operation, DocumentChangeOperation::Op(_)))
        .unwrap_or(operations.len());

    operations.splice(
        insert_at..insert_at,
        edits.into_iter().map(DocumentChangeOperation::Edit),
    );
}

fn changes_to_document_edits(
    changes: std::collections::HashMap<Url, Vec<TextEdit>>,
) -> Vec<TextDocumentEdit> {
    changes
        .into_iter()
        .map(|(uri, edits)| TextDocumentEdit {
            text_document: OptionalVersionedTextDocumentIdentifier { uri, version: None },
            edits: edits.into_iter().map(OneOf::Left).collect(),
        })
        .collect()
}
