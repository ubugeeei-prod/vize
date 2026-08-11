//! Switching a long-lived client onto a materialized Canon project.

use std::path::Path;

use corsa::runtime::block_on;
use vize_carton::String;

use super::{
    CorsaProjectClient,
    lifecycle_setup::workspace_config_path,
    session::{ProjectSessionSpawnError, spawn_project_session},
};

impl CorsaProjectClient {
    pub(crate) fn synchronize_materialized_project(
        &mut self,
        project_root: &Path,
        changes: &crate::batch::virtual_project::MaterializedFileDelta,
    ) -> Result<(), String> {
        if changes.has_topology_changes() {
            self.reload_workspace_project(project_root)?;
        } else {
            self.activate_workspace_project(project_root)?;
            if !changes.is_empty() {
                self.refresh_materialized_files(
                    &changes.changed,
                    &changes.created,
                    &changes.deleted,
                )?;
            }
        }
        Ok(())
    }

    /// Move both native query transports to an already-materialized Canon
    /// project. The mirror's tsconfig is the authority for native condition
    /// selection; merely opening a file under its `node_modules` tree would
    /// otherwise create an inferred project with default compiler options.
    pub(crate) fn activate_workspace_project(&mut self, project_root: &Path) -> Result<(), String> {
        self.activate_workspace_project_with_reload(project_root, false)
    }

    /// Replace only the native project handle when package topology changes.
    /// Standard tsgo retains negative module-resolution state across a file
    /// summary refresh, while a new handle observes the already-materialized
    /// Canon snapshot without restarting the bridge process.
    pub(crate) fn reload_workspace_project(&mut self, project_root: &Path) -> Result<(), String> {
        self.activate_workspace_project_with_reload(project_root, true)
    }

    fn activate_workspace_project_with_reload(
        &mut self,
        project_root: &Path,
        reload: bool,
    ) -> Result<(), String> {
        let project_root = project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf());
        if !reload && self.project_root == project_root {
            return Ok(());
        }

        let config_path = workspace_config_path(&project_root);
        let (session, capabilities) =
            match spawn_project_session(self.executable.as_str(), &project_root, &config_path) {
                Ok((session, capabilities)) => (Some(session), capabilities),
                Err(ProjectSessionSpawnError::Unavailable(reason)) => {
                    tracing::debug!(
                        reason = reason.as_str(),
                        "using standard tsgo editor-only Canon mirror session"
                    );
                    (None, std::sync::Arc::new(Default::default()))
                }
                Err(ProjectSessionSpawnError::Failed(error)) => return Err(error),
            };
        let previous = std::mem::replace(&mut self.session, session);
        if let Some(previous) = previous {
            let _ = block_on(previous.close());
        }
        self.capabilities = capabilities;
        self.cwd = project_root.clone();
        self.project_root = project_root;
        self.materialized_project_session = false;
        self.session_document_uris.clear();
        self.external_document_uris.clear();
        self.diagnostics.clear();
        self.retire_editor_lsp();
        Ok(())
    }
}
