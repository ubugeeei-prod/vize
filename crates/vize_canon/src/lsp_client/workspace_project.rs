//! Switching a long-lived client onto a materialized Canon project.

use std::path::Path;

use corsa::runtime::block_on;
use vize_l0::String;

use super::{
    CorsaProjectClient,
    lifecycle_setup::workspace_config_path,
    session::{ProjectSessionSpawnError, spawn_project_session},
};

impl CorsaProjectClient {
    pub(crate) fn synchronize_materialized_project(
        &mut self,
        project_root: &Path,
        config_path: Option<&Path>,
        changes: &crate::batch::virtual_project::MaterializedFileDelta,
    ) -> Result<(), String> {
        if changes.has_topology_changes() {
            self.activate_workspace_project_with_reload(project_root, config_path, true)?;
            self.refresh_materialized_files(&changes.changed, &changes.created, &changes.deleted)?;
        } else {
            self.activate_workspace_project_with_reload(project_root, config_path, false)?;
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
    /// Replace only the native project handle when package topology changes.
    /// Standard tsgo retains negative module-resolution state across a file
    /// summary refresh, while a new handle observes the already-materialized
    /// Canon snapshot without restarting the bridge process.
    fn activate_workspace_project_with_reload(
        &mut self,
        project_root: &Path,
        config_path: Option<&Path>,
        reload: bool,
    ) -> Result<(), String> {
        let project_root = project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf());
        let root_changed = self.project_root != project_root;
        let config_changed = explicit_config_changed(
            self.explicit_project_config.as_deref(),
            config_path,
            &project_root,
        );
        if !reload && !root_changed && !config_changed {
            return Ok(());
        }

        self.retire_original_diagnosing_session()?;

        let config_path = config_path
            .map(Path::to_path_buf)
            .unwrap_or_else(|| workspace_config_path(&project_root));
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
        if self.explicit_project_config.is_some() {
            self.explicit_project_config = Some(config_path);
        }
        self.materialized_project_session = false;
        self.clear_workspace_project_overlays();
        self.session_document_uris.clear();
        self.external_document_uris.clear();
        self.diagnostics.clear();
        if root_changed || config_changed {
            self.retire_editor_lsp()?;
        }
        Ok(())
    }

    fn clear_workspace_project_overlays(&mut self) {
        self.document_texts.clear();
        self.overlay_versions.clear();
        self.editor_lsp_documents_dirty = true;
    }
}

fn explicit_config_changed(current: Option<&Path>, requested: Option<&Path>, root: &Path) -> bool {
    current.is_some_and(|current| {
        requested
            .map(Path::to_path_buf)
            .unwrap_or_else(|| workspace_config_path(root))
            != current
    })
}

#[cfg(test)]
mod tests {
    use super::{CorsaProjectClient, explicit_config_changed};
    use std::path::Path;

    #[test]
    fn explicit_selection_changes_are_observed_before_same_root_reuse() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("playground/tsconfig.json");
        assert!(!explicit_config_changed(None, Some(&nested), root.path()));
        assert!(!explicit_config_changed(
            Some(&nested),
            Some(&nested),
            root.path()
        ));
        assert!(explicit_config_changed(Some(&nested), None, root.path()));
        assert!(explicit_config_changed(
            Some(&nested),
            Some(Path::new("/other/config.json")),
            root.path(),
        ));
    }

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
