//! Forward disk revisions to the same native editor project as its overlays.

use std::{path::PathBuf, str::FromStr};

use lsp_types::{DidChangeWatchedFilesParams, FileChangeType, FileEvent, Uri};
use vize_s0::{String, cstr};

use super::EditorLspSession;

impl EditorLspSession {
    pub(in crate::lsp_client) fn refresh_materialized_files(
        &mut self,
        changed: &[PathBuf],
        created: &[PathBuf],
        deleted: &[PathBuf],
    ) -> Result<(), String> {
        let mut changes = Vec::new();
        for (paths, typ) in [
            (changed, FileChangeType::CHANGED),
            (created, FileChangeType::CREATED),
            (deleted, FileChangeType::DELETED),
        ] {
            for path in paths {
                let uri = crate::file_uri::path_to_file_uri(path);
                let uri = Uri::from_str(uri.as_str())
                    .map_err(|error| cstr!("Invalid materialized file URI: {error}"))?;
                changes.push(FileEvent { uri, typ });
            }
        }
        if changes.is_empty() {
            return Ok(());
        }
        // An updated live overlay will supply its own document readiness
        // request. Unopened disk inputs and topology changes need a query
        // barrier even when no overlay text changes.
        self.query_barrier_required |= changes.iter().any(|change| {
            change.typ != FileChangeType::CHANGED
                || !self.documents.contains_key(change.uri.as_str())
        });
        self.client
            .notify::<lsp_types::notification::DidChangeWatchedFiles>(DidChangeWatchedFilesParams {
                changes,
            })
            .map_err(|error| cstr!("Failed to update editor LSP disk revisions: {error}"))?;
        self.advance_document_generation();
        Ok(())
    }
}
