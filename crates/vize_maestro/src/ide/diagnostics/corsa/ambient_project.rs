//! Explicit native Vue diagnostics for an editor snapshot and its workspace declarations.
#![expect(
    clippy::disallowed_types,
    reason = "retain this editor owner across native requests"
)]
use super::collect_virtual::{CorsaDocument, assemble_corsa_diagnostics};
use crate::{
    server::ServerState,
    source_project::{SnapshotRefusal, SourceSnapshot},
};
use futures::lock::Mutex;
use std::{io, path::PathBuf, sync::Arc};
use tower_lsp::lsp_types::Diagnostic;
use vize_canon::{CorsaBridge, CorsaBridgeConfig, CorsaBridgeError, CorsaVueWorkspaceRequest};

mod positions;

/// An explicitly started native session. Its requests serialize only with each other.
pub struct AmbientComponentProject {
    state: Arc<ServerState>,
    root: PathBuf,
    bridge: CorsaBridge,
    transaction: Mutex<Option<PublishedDiagnostics>>,
}

struct PublishedDiagnostics {
    read: vize_canon::corsa_bridge::AmbientVueDiagnosticRead<Vec<Diagnostic>>,
    revision: u64,
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
        let prior = previous
            .take()
            .filter(|last| last.revision == snapshot.revision())
            .map(|last| last.read);
        let (options, mut declarations) = state
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
            .read_ambient_vue_diagnostic_batch(
                vize_canon::corsa_bridge::AmbientVueDiagnosticRequest {
                    source_path: &path,
                    content: snapshot.source(),
                    options: document_options,
                    overlays: &borrowed(&overlays),
                    virtual_ts_options: &options,
                    requested_sources: &take_paths_with_text(&mut declarations),
                },
                prior,
            )
            .await
            .map_err(AmbientProjectError::Native)?;
        current()?;
        let response = document.map(|host, rows| {
            let finished = positions::finish(&host.code, rows);
            // Empty compiler rows can still yield unused authored expectations.
            let projection = (!finished.is_empty()).then(|| CorsaDocument {
                code: host.code.clone(),
                mapping: host.mapping.clone(),
                import_source_map: host.import_source_map.clone(),
            });
            assemble_corsa_diagnostics(snapshot.source(), projection.as_slice(), finished)
                .into_iter()
                .map(|row| row.diagnostic)
                .collect()
        });
        current()?;
        if response.has_native_snapshot() {
            let returned = response.value().clone();
            *previous = Some(PublishedDiagnostics {
                read: response,
                revision: snapshot.revision(),
            });
            return Ok(returned);
        }
        Ok(response.into_value())
    }

    pub async fn shutdown(self) -> Result<(), CorsaBridgeError> {
        drop(self.transaction.into_inner());
        self.bridge.shutdown().await
    }
}

// The native read is the last consumer of these paths. The input buffers stay
// alive until the bridge has consumed all of the borrowed source texts.
fn take_paths_with_text(
    entries: &mut [(PathBuf, Arc<String>)],
) -> vize_l0::SmallVec<[(PathBuf, &str); 4]> {
    entries
        .iter_mut()
        .map(|(path, text)| (std::mem::take(path), text.as_str()))
        .collect()
}

fn borrowed(entries: &[(PathBuf, Arc<String>)]) -> vize_l0::SmallVec<[(PathBuf, &str); 4]> {
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
