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
    control: Option<&crate::lsp_client::startup_control::StartupControl>,
) -> Option<SharedProject> {
    if !supports_shared_editor_project(executable, config) {
        return None;
    }
    match spawn_shared_editor_project(executable, cwd, root, config, control) {
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
    control: Option<&crate::lsp_client::startup_control::StartupControl>,
) -> Result<SharedProject, String> {
    let mut editor = EditorLspSession::spawn_with_startup_configuration(
        executable,
        cwd,
        root,
        config,
        control.cloned(),
    )?;
    let result = (|| {
        let api = editor
            .configured_api
            .as_ref()
            .ok_or_else(|| cstr!("Editor project has no retained API attachment"))?;
        let session = block_on(ProjectSession::open(
            api.client.clone(),
            config.to_string_lossy().as_ref(),
            None,
        ))
        .map_err(|error| cstr!("Cannot open attached editor project: {error}"))?;
        let capabilities = match block_on(session.describe_capabilities()) {
            Ok(capabilities) => capabilities,
            Err(error) => {
                crate::lsp_client::retirement::ownership_error(
                    &editor.client.process_ownership(),
                    cstr!("attached native capabilities: {error}").as_str(),
                );
                if let Some(control) = control
                    && let Err(closed) = control.check_admission()
                {
                    return Err(cstr!("{closed}; attached native capabilities: {error}"));
                }
                Arc::new(CapabilitiesResponse::default())
            }
        };
        Ok((session, capabilities))
    })();
    match result {
        Ok((session, capabilities)) => {
            editor.finish_startup()?;
            Ok((session, capabilities, editor))
        }
        Err(error) => {
            crate::lsp_client::retirement::ownership_error(
                &editor.client.process_ownership(),
                error.as_str(),
            );
            if let Err(cleanup) = editor.shutdown() {
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
            self.editor_lsp = Some(EditorLspSession::spawn_with_config_controlled(
                self.executable.as_str(),
                &self.cwd,
                &root,
                self.explicit_project_config.as_deref(),
                self.startup_control.clone(),
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
        if self.shared_editor_transport
            && let Some((session, capabilities, editor)) = try_shared_editor_project(
                self.executable.as_str(),
                &self.cwd,
                root,
                config,
                self.startup_control.as_ref(),
            )
        {
            self.session = Some(session);
            self.capabilities = capabilities;
            self.editor_lsp = Some(editor);
            self.project_session_attached_to_editor = true;
            return Ok(());
        }
        match crate::lsp_client::session::spawn_project_session_with_control(
            self.executable.as_str(),
            &self.cwd,
            config,
            self.startup_control.as_ref(),
        ) {
            Ok((session, capabilities)) => {
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
            // Graceful retirement releases the live snapshot before its owner.
            // ProjectSession::close must never close the shared socket first.
            block_on(session.snapshot().release())
                .map_err(|error| cstr!("Cannot release attached editor snapshot: {error}"))
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
        let result = match (snapshot_result, editor_result) {
            (Ok(()), result) | (result, Ok(())) => result,
            (Err(snapshot), Err(editor)) => {
                Err(cstr!("{snapshot}; editor owner shutdown: {editor}"))
            }
        };
        self.publish_closed_native_owners(&result);
        result
    }
}
