//! One editor owner may attach its snapshot API to its own LSP process.

use super::super::EditorLspSession;
use crate::lsp_client::{CorsaProjectClient, lifecycle_setup::workspace_config_path};
use corsa::{
    api::{CapabilitiesResponse, ProjectSession},
    runtime::block_on,
};
use std::{path::Path, sync::Arc};
use vize_l0::{String, cstr};

type SharedProject = (ProjectSession, Arc<CapabilitiesResponse>, EditorLspSession);

fn supports_shared_editor_project(executable: &str, config: &Path) -> bool {
    cfg!(unix)
        && config.is_file()
        && crate::lsp_client::session::api_mode_for_executable(executable)
            == corsa::api::ApiMode::AsyncJsonRpcStdio
}

pub(in crate::lsp_client) fn try_shared_editor_project(
    executable: &str,
    cwd: &Path,
    root: &Path,
    config: &Path,
    storage: &crate::lsp_client::session_storage::SessionStorage,
    preferred: Option<&str>,
) -> Option<SharedProject> {
    if !supports_shared_editor_project(executable, config) {
        return None;
    }
    match spawn_shared_editor_project(executable, cwd, root, config, storage, preferred) {
        Ok(project) => Some(project),
        Err(error) => {
            tracing::debug!(
                error = error.as_str(),
                "using independent project API transport"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests;

fn spawn_shared_editor_project(
    executable: &str,
    cwd: &Path,
    root: &Path,
    config: &Path,
    storage: &crate::lsp_client::session_storage::SessionStorage,
    preferred: Option<&str>,
) -> Result<SharedProject, String> {
    let mut editor =
        EditorLspSession::spawn_with_startup_configuration(executable, cwd, root, config)
            .inspect_err(|_| {
                storage.retain_after_failed_shutdown();
            })?;
    let result = (|| {
        let api = editor
            .configured_api
            .as_ref()
            .ok_or_else(|| cstr!("Editor project has no retained API attachment"))?;
        let session = block_on(ProjectSession::open(
            api.client.clone(),
            config.to_string_lossy().as_ref(),
            preferred.map(crate::lsp_client::session::uri_document_identifier),
        ))
        .map_err(|error| cstr!("Cannot open attached editor project: {error}"))?;
        crate::lsp_client::session::validate_materialized_project_selection(
            session.project(),
            config,
            preferred,
        )?;
        let capabilities = block_on(session.describe_capabilities())
            .unwrap_or_else(|_| Arc::new(CapabilitiesResponse::default()));
        Ok((session, capabilities))
    })();
    match result {
        Ok((session, capabilities)) => Ok((session, capabilities, editor)),
        Err(error) => {
            let cleanup = editor.shutdown();
            storage.observe_retirement(&cleanup);
            if let Err(cleanup) = cleanup {
                tracing::warn!(
                    error = cleanup.as_str(),
                    "cannot retire failed editor attachment"
                );
            }
            Err(error)
        }
    }
}

impl CorsaProjectClient {
    pub(in crate::lsp_client) fn ensure_editor_transport(&mut self) -> Result<(), String> {
        if self.editor_lsp.is_some() {
            return Ok(());
        }
        let root = self.editor_lsp_project_root();
        let config = self
            .explicit_project_config
            .clone()
            .unwrap_or_else(|| workspace_config_path(&root));
        if self.shared_editor_transport {
            self.retire_project_session()?;
            self.replace_project_transport(&config, &root)?;
            if let Err(error) = self.replay_retained_project_documents() {
                if self.project_session_attached_to_editor {
                    let _ = self.retire_shared_editor_transport(true);
                } else {
                    let _ = self.retire_project_session();
                    let _ = self.discard_editor_lsp();
                }
                return Err(error);
            }
        }
        if self.editor_lsp.is_none() {
            self.editor_lsp = Some(EditorLspSession::spawn_with_config(
                self.executable.as_str(),
                &self.cwd,
                &root,
                self.explicit_project_config.as_deref(),
            )?);
        }
        self.editor_lsp_documents_dirty = true;
        Ok(())
    }

    pub(in crate::lsp_client) fn replace_project_transport(
        &mut self,
        config: &Path,
        root: &Path,
    ) -> Result<(), String> {
        let cwd = self.cwd.clone();
        if self.try_replace_shared_project_transport(config, root, &cwd)? {
            return Ok(());
        }
        let preferred = self.preferred_document_for_config(config);
        match crate::lsp_client::session::spawn_project_session_for_document(
            self.executable.as_str(),
            &self.cwd,
            config,
            preferred.as_deref(),
        ) {
            Ok((session, capabilities)) => {
                if let Err(error) =
                    crate::lsp_client::session::validate_materialized_project_selection(
                        session.project(),
                        config,
                        preferred.as_deref(),
                    )
                {
                    let cleanup = block_on(session.close()).map_err(|error| {
                        cstr!("Failed to retire refused materialized Corsa project: {error}")
                    });
                    self.overlay_storage.observe_retirement(&cleanup);
                    return Err(error);
                }
                self.session = Some(session);
                self.capabilities = capabilities;
            }
            Err(crate::lsp_client::session::ProjectSessionSpawnError::Unavailable(error))
                if self.shared_editor_transport =>
            {
                tracing::debug!(
                    error = error.as_str(),
                    "using editor-only replacement transport"
                );
                self.session = None;
                self.capabilities = Arc::new(CapabilitiesResponse::default());
            }
            Err(crate::lsp_client::session::ProjectSessionSpawnError::Unavailable(error))
            | Err(crate::lsp_client::session::ProjectSessionSpawnError::Failed(error)) => {
                return Err(error);
            }
        }
        self.shared_editor_transport = false;
        Ok(())
    }

    /// A refused attachment leaves the caller's existing independent-API
    /// fallback/error policy in charge. Requested ownership is not consumed.
    pub(in crate::lsp_client) fn try_replace_shared_project_transport(
        &mut self,
        config: &Path,
        root: &Path,
        cwd: &Path,
    ) -> Result<bool, String> {
        if !self.shared_editor_transport_requested {
            return Ok(false);
        }
        // An initial refusal may have spawned an independent editor later.
        // Reap it before a replacement can acquire the same project owner.
        self.retire_editor_lsp()?;
        let preferred = self.preferred_document_for_config(config);
        if let Some((session, capabilities, editor)) = try_shared_editor_project(
            self.executable.as_str(),
            cwd,
            root,
            config,
            &self.overlay_storage,
            preferred.as_deref(),
        ) {
            self.session = Some(session);
            self.capabilities = capabilities;
            self.editor_lsp = Some(editor);
            self.project_session_attached_to_editor = true;
            self.shared_editor_transport = true;
            return Ok(true);
        }
        Ok(false)
    }

    pub(in crate::lsp_client) fn retire_shared_editor_transport(
        &mut self,
        discard: bool,
    ) -> Result<(), String> {
        // Keep the snapshot alive until its owning process is reaped: a full
        // SDK release queue makes snapshot drop synchronously call its peer.
        let session = self.session.take();
        let snapshot_result = session.as_ref().map_or(Ok(()), |session| {
            if discard {
                return Ok(());
            }
            // A derived snapshot's lineage can retain its live head, while
            // replaced snapshots already have releases on the SDK worker.
            // Await those replies with both the API reader and LSP owner alive.
            // ProjectSession::close must never close the shared socket first.
            let released = block_on(session.snapshot().release())
                .map_err(|error| cstr!("Cannot release attached editor snapshot: {error}"));
            let drained = block_on(session.client().drain_snapshot_releases())
                .map_err(|error| cstr!("Cannot drain attached editor snapshot releases: {error}"));
            released.and(drained)
        });
        let editor_result = self.editor_lsp.take().map_or(Ok(()), |mut editor| {
            if discard {
                editor.discard()
            } else {
                editor.shutdown()
            }
        });
        drop(session);
        self.project_session_attached_to_editor = false;
        self.editor_lsp_documents_dirty = true;
        let result = snapshot_result.and(editor_result);
        self.overlay_storage.observe_retirement(&result);
        result
    }
}
