//! Switching a long-lived client onto a materialized Canon project.

use std::path::Path;

use corsa::runtime::block_on;
use vize_s0::String;

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
        self.clear_workspace_project_overlays();
        self.session_document_uris.clear();
        self.external_document_uris.clear();
        self.diagnostics.clear();
        let _ = self.retire_editor_lsp();
        Ok(())
    }

    fn clear_workspace_project_overlays(&mut self) {
        self.document_texts.clear();
        self.overlay_versions.clear();
        self.editor_lsp_documents_dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::CorsaProjectClient;

    #[test]
    fn project_reload_drops_overlays_before_the_editor_fallback_can_reopen_them() {
        let root = tempfile::tempdir().unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        client.document_texts.insert(
            "file:///mirror/deleted.ts".into(),
            "export const stale = true;".into(),
        );
        client
            .overlay_versions
            .insert("file:///mirror/deleted.ts".into(), 3);

        client.clear_workspace_project_overlays();

        assert!(client.document_texts.is_empty());
        assert!(client.overlay_versions.is_empty());
        assert!(client.editor_lsp_documents_dirty);
    }
}
