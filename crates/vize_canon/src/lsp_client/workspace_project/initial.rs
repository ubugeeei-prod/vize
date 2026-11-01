//! First-owner completion without replaying an already-materialized topology.

use super::CorsaProjectClient;
use crate::{batch::virtual_project::MaterializedFileDelta, file_uri::path_to_file_uri};
use vize_l0::{String, cstr};

impl CorsaProjectClient {
    /// Reject every delta spelling the fresh-owner refresh would reject,
    /// before any native child is created. This performs no native I/O.
    pub(crate) fn validate_initial_editor_project_delta(
        changes: &MaterializedFileDelta,
    ) -> Result<(), String> {
        // Keep the existing FileChanges validation order: grouped, sorted,
        // deduplicated UTF-8 paths, then the fresh editor's original URI order.
        for paths in [&changes.changed, &changes.created, &changes.deleted] {
            let mut paths = paths.to_vec();
            paths.sort();
            paths.dedup();
            for path in paths {
                path.to_str().ok_or_else(|| {
                    cstr!(
                        "Corsa cannot represent non-UTF-8 materialized path {:?}",
                        path.as_os_str()
                    )
                })?;
            }
        }
        for path in changes
            .changed
            .iter()
            .chain(&changes.created)
            .chain(&changes.deleted)
        {
            path_to_file_uri(path)
                .parse::<lsp_types::Uri>()
                .map_err(|error| cstr!("Invalid materialized file URI: {error}"))?;
        }
        Ok(())
    }

    pub(in crate::lsp_client) fn finish_initial_materialized_project(
        &mut self,
        changes: &MaterializedFileDelta,
    ) -> Result<(), String> {
        // Match an authored editor constructor followed by its first successful
        // project activation, including its implicit configuration selection.
        self.explicit_project_config = None;
        self.materialized_project_session = false;
        self.authored_workspace = false;
        self.logical_workspace_overlays = false;
        self.logical_api_reset_pending = false;
        self.clear_workspace_project_overlays();
        self.session_document_uris.clear();
        self.external_document_uris.clear();
        self.diagnostics.clear();
        self.refresh_after_project_replacement(
            false,
            &changes.changed,
            &changes.created,
            &changes.deleted,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{CorsaProjectClient, MaterializedFileDelta};

    #[test]
    fn first_project_completion_matches_activation_state_without_topology_reload() {
        let root = tempfile::tempdir().unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        client.explicit_project_config = Some(root.path().join("tsconfig.json"));
        client.materialized_project_session = true;
        client.authored_workspace = true;
        client.logical_workspace_overlays = true;
        client.logical_api_reset_pending = true;
        client.editor_lsp_documents_dirty = false;
        client
            .document_texts
            .insert("file:///old.ts".into(), "old".into());
        client.overlay_versions.insert("file:///old.ts".into(), 7);
        client
            .session_document_uris
            .insert("old".into(), "mapped".into());
        client
            .external_document_uris
            .insert("mapped".into(), "old".into());
        client.diagnostics.insert("old".into(), Vec::new());
        let storage = client.overlay_storage.clone();
        let changes = MaterializedFileDelta {
            created: vec![root.path().join("package.json")],
            deleted: vec![root.path().join("node_modules/retired")],
            topology_changed: true,
            ..Default::default()
        };
        assert!(changes.has_topology_changes());
        client
            .finish_initial_materialized_project(&changes)
            .unwrap();
        assert!(std::sync::Arc::ptr_eq(&storage, &client.overlay_storage));
        assert_eq!(client.project_root, root.path());
        assert_eq!(client.cwd, root.path());
        assert!(client.explicit_project_config.is_none());
        assert!(!client.materialized_project_session);
        assert!(!client.authored_workspace);
        assert!(!client.logical_workspace_overlays);
        assert!(!client.logical_api_reset_pending);
        assert!(client.editor_lsp_documents_dirty);
        assert!(client.document_texts.is_empty());
        assert!(client.overlay_versions.is_empty());
        assert!(client.session_document_uris.is_empty());
        assert!(client.external_document_uris.is_empty());
        assert!(client.diagnostics.is_empty());
        assert!(client.session.is_none());
        assert!(client.editor_lsp.is_none());
        assert!(!client.closed);
    }

    #[test]
    fn first_delta_preserves_grouping_duplicates_unicode_and_empty_topology() {
        let root = tempfile::tempdir().unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        let path = root.path().join("雪 # [id].vue.ts");
        for changes in [
            MaterializedFileDelta::default(),
            MaterializedFileDelta {
                topology_changed: true,
                ..Default::default()
            },
            MaterializedFileDelta {
                changed: vec![path.clone(), path.clone()],
                created: vec![path.clone()],
                deleted: vec![path],
                topology_changed: true,
            },
        ] {
            let before = changes.clone();
            assert_eq!(
                CorsaProjectClient::validate_initial_editor_project_delta(&changes),
                client.refresh_after_project_replacement(
                    false,
                    &changes.changed,
                    &changes.created,
                    &changes.deleted
                )
            );
            client
                .finish_initial_materialized_project(&changes)
                .unwrap();
            assert_eq!(changes, before);
        }
    }

    #[cfg(unix)]
    #[test]
    fn invalid_delta_is_rejected_before_provider_resolution_with_the_old_error() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};
        let root = tempfile::tempdir().unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        let invalid = PathBuf::from(OsString::from_vec(b"/mirror/\x80.ts".to_vec()));
        for changes in [
            MaterializedFileDelta {
                changed: vec![invalid.clone()],
                ..Default::default()
            },
            MaterializedFileDelta {
                created: vec![invalid.clone()],
                ..Default::default()
            },
            MaterializedFileDelta {
                deleted: vec![invalid],
                ..Default::default()
            },
        ] {
            let old = client
                .refresh_after_project_replacement(
                    false,
                    &changes.changed,
                    &changes.created,
                    &changes.deleted,
                )
                .unwrap_err();
            assert_eq!(
                CorsaProjectClient::validate_initial_editor_project_delta(&changes).unwrap_err(),
                old
            );
            let missing = root.path().join("missing-tsgo");
            assert_eq!(
                CorsaProjectClient::new_for_editor_materialized_workspace(
                    missing.to_str(),
                    root.path(),
                    root.path(),
                    &root.path().join("tsconfig.json"),
                    &changes
                )
                .err()
                .unwrap(),
                old
            );
        }
    }

    #[test]
    fn missing_explicit_provider_keeps_the_original_error_without_fallback() {
        let root = tempfile::tempdir().unwrap();
        let canon = tempfile::tempdir().unwrap();
        let old = CorsaProjectClient::new_for_editor_workspace(Some("missing-tsgo"), root.path())
            .err()
            .unwrap();
        let actual = CorsaProjectClient::new_for_editor_materialized_workspace(
            Some("missing-tsgo"),
            root.path(),
            canon.path(),
            &canon.path().join("selected.json"),
            &MaterializedFileDelta::default(),
        )
        .err()
        .unwrap();
        assert_eq!(actual, old);
    }
}
