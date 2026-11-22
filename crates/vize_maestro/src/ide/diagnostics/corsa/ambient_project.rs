//! Explicit native Vue diagnostics for an editor snapshot and its workspace declarations.
#![expect(
    clippy::disallowed_types,
    reason = "retain this editor owner across native requests"
)]
use super::collect_virtual::{CorsaDocument, assemble_corsa_diagnostics, finished_from_phased_lsp};
use crate::{
    server::ServerState,
    source_project::{SnapshotRefusal, SourceSnapshot},
};
use futures::lock::Mutex;
use std::{io, path::PathBuf, sync::Arc};
use tower_lsp::lsp_types::Diagnostic;
use vize_canon::{CorsaBridge, CorsaBridgeConfig, CorsaBridgeError};

/// An explicitly started native session. Its requests serialize only with each other.
pub struct AmbientComponentProject {
    state: Arc<ServerState>,
    root: PathBuf,
    bridge: CorsaBridge,
    transaction: Mutex<()>,
}

#[derive(Debug)]
pub enum AmbientProjectError {
    Snapshot(SnapshotRefusal),
    Workspace,
    UnsupportedDocument,
    Declaration(io::Error),
    Native(CorsaBridgeError),
}

impl AmbientComponentProject {
    pub async fn start(
        state: Arc<ServerState>,
        mut config: CorsaBridgeConfig,
    ) -> Result<Self, AmbientProjectError> {
        let root = state
            .get_workspace_root()
            .ok_or(AmbientProjectError::Workspace)?;
        let root = vize_l0::path::physical_path(&root);
        if config
            .working_dir
            .as_ref()
            .is_some_and(|path| vize_l0::path::physical_path(path) != root)
        {
            return Err(AmbientProjectError::Workspace);
        }
        config.working_dir = Some(root.clone());
        let bridge = CorsaBridge::with_ambient_declaration_project(config);
        bridge.spawn().await.map_err(AmbientProjectError::Native)?;
        Ok(Self {
            state,
            root,
            bridge,
            transaction: Mutex::new(()),
        })
    }

    /// Return the native checker's complete diagnostic projection for this open
    /// Vue module. A revision change invalidates the whole answer.
    pub async fn diagnostics(
        &self,
        snapshot: &SourceSnapshot,
    ) -> Result<Vec<Diagnostic>, AmbientProjectError> {
        let _transaction = self.transaction.lock().await;
        let state = &self.state;
        let stamp = state.corsa_request_stamp();
        let current = || {
            snapshot
                .check_current(&state.documents)
                .map_err(AmbientProjectError::Snapshot)?;
            if !stamp.is_current(state) {
                return Err(AmbientProjectError::Snapshot(SnapshotRefusal::Superseded));
            }
            Ok(())
        };
        current()?;
        if !state
            .get_workspace_root()
            .is_some_and(|p| vize_l0::path::physical_path(&p) == self.root)
        {
            return Err(AmbientProjectError::Workspace);
        }
        let path = snapshot
            .uri()
            .to_file_path()
            .map_err(|()| AmbientProjectError::UnsupportedDocument)?;
        let path = vize_l0::path::physical_path(&path);
        if snapshot.language_id() != "vue"
            || path.extension().is_none_or(|e| e != "vue")
            || snapshot.uri().path().ends_with(".art.vue")
            || !path.starts_with(&self.root)
        {
            return Err(AmbientProjectError::UnsupportedDocument);
        }
        let overlays = state.corsa_overlays();
        let (options, declarations) = state
            .ambient_declaration_inputs(&self.root, &overlays)
            .await
            .map_err(AmbientProjectError::Declaration)?;
        current()?;
        let document = self
            .bridge
            .open_vue_virtual_workspace_document(
                &path,
                snapshot.source(),
                state.canonical_vue_document_options(),
                &borrowed(&overlays),
                &options,
                &borrowed(&declarations),
            )
            .await
            .map_err(AmbientProjectError::Native)?;
        let raw = self
            .bridge
            .get_diagnostics_with_provenance(&document.request_uri)
            .await
            .map_err(AmbientProjectError::Native)?;
        current()?;
        let projection = CorsaDocument {
            code: document.code,
            mapping: document.mapping,
            import_source_map: document.import_source_map,
        };
        let finished = raw
            .into_iter()
            .filter_map(|row| finished_from_phased_lsp(&projection.code, row, 0))
            .collect();
        let response = assemble_corsa_diagnostics(snapshot.source(), &[projection], finished)
            .into_iter()
            .map(|row| row.diagnostic)
            .collect();
        current()?;
        Ok(response)
    }

    pub async fn shutdown(self) -> Result<(), CorsaBridgeError> {
        self.bridge.shutdown().await
    }
}

fn borrowed(entries: &[(PathBuf, Arc<String>)]) -> Vec<(PathBuf, &str)> {
    entries
        .iter()
        .map(|(path, text)| (path.clone(), text.as_str()))
        .collect()
}

#[cfg(test)]
mod tests;
