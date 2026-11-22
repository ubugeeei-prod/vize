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
use vize_canon::{CorsaBridge, CorsaBridgeConfig, CorsaBridgeError, CorsaVueWorkspaceRequest};

/// An explicitly started native session. Its requests serialize only with each other.
pub struct AmbientComponentProject {
    state: Arc<ServerState>,
    root: PathBuf,
    bridge: CorsaBridge,
    transaction: Mutex<Option<vize_canon::lsp_client::AmbientDiagnosticSnapshot>>,
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
    /// Create an independent editor owner. Its first native request starts
    /// directly on the captured Vue project whenever the backend supports it.
    pub fn new(
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
        Ok(Self {
            state,
            root,
            bridge,
            transaction: Mutex::new(None),
        })
    }

    /// Return the native checker's complete diagnostic projection for this open
    /// Vue module. A revision change invalidates the whole answer.
    pub async fn diagnostics(
        &self,
        snapshot: &SourceSnapshot,
    ) -> Result<Vec<Diagnostic>, AmbientProjectError> {
        let mut previous = self.transaction.lock().await;
        let state = &self.state;
        let stamp = state.corsa_request_stamp();
        let declaration_revision = state.ambient_declaration_revision();
        let current = || {
            snapshot
                .check_current(&state.documents)
                .map_err(AmbientProjectError::Snapshot)?;
            if !stamp.is_current(state)
                || state.ambient_declaration_revision() != declaration_revision
            {
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
        let document_options = state.canonical_vue_document_options();
        if !self.bridge.is_initialized() {
            let seed = self
                .bridge
                .prepare_initial_vue_project(CorsaVueWorkspaceRequest {
                    source_path: path.clone(),
                    content: snapshot.source().into(),
                    options: document_options,
                    virtual_ts_options: options.clone(),
                    overlays: owned(&overlays),
                    requested_sources: owned(&declarations),
                })
                .map_err(AmbientProjectError::Native)?;
            current()?;
            if let Some(seed) = seed {
                let owner = Arc::clone(state);
                // The subsequent read always synchronizes the entire current
                // graph, even when inputs changed during the initial handshake.
                self.bridge
                    .spawn_with_initial_project(seed, move || {
                        stamp.is_current(&owner)
                            && owner.ambient_declaration_revision() == declaration_revision
                    })
                    .await
                    .map_err(AmbientProjectError::Native)?;
            } else {
                self.bridge
                    .spawn()
                    .await
                    .map_err(AmbientProjectError::Native)?;
            }
        }
        current()?;
        let document = self
            .bridge
            .read_ambient_vue_diagnostics(
                &path,
                snapshot.source(),
                document_options,
                &borrowed(&overlays),
                &options,
                &borrowed(&declarations),
                previous.take(),
            )
            .await
            .map_err(AmbientProjectError::Native)?;
        current()?;
        let projection = CorsaDocument {
            code: document.code,
            mapping: document.mapping,
            import_source_map: document.import_source_map,
        };
        let finished = document
            .diagnostics
            .into_iter()
            .filter_map(|row| finished_from_phased_lsp(&projection.code, row, 0))
            .collect();
        let response = assemble_corsa_diagnostics(snapshot.source(), &[projection], finished)
            .into_iter()
            .map(|row| row.diagnostic)
            .collect();
        current()?;
        *previous = document.snapshot;
        Ok(response)
    }

    pub async fn shutdown(self) -> Result<(), CorsaBridgeError> {
        drop(self.transaction.into_inner());
        self.bridge.shutdown().await
    }
}

fn borrowed(entries: &[(PathBuf, Arc<String>)]) -> Vec<(PathBuf, &str)> {
    entries
        .iter()
        .map(|(path, text)| (path.clone(), text.as_str()))
        .collect()
}

fn owned(entries: &[(PathBuf, Arc<String>)]) -> Vec<(PathBuf, Arc<str>)> {
    entries
        .iter()
        .map(|(path, text)| (path.clone(), Arc::from(text.as_str())))
        .collect()
}

#[cfg(test)]
mod tests;
