//! Preserve authored names when the native API cannot own the live revision.

use super::CorsaProjectClient;
use crate::file_uri::file_uri_to_path;
use std::path::Component;
use vize_l0::{FxHashMap, String, cstr};

impl CorsaProjectClient {
    pub(in crate::lsp_client) fn try_logical_workspace_overlays(&mut self) -> Result<bool, String> {
        if self.logical_workspace_overlays {
            self.retain_logical_workspace_revision()?;
            return Ok(true);
        }
        if !self.authored_workspace
            || self.materialized_project_session
            || self.project_root.join("__vize_vue_modules.d.ts").is_file()
            || (self.has_project_session()
                && self.supports_overlay_api()
                && self.can_use_attached_semantic_api())
        {
            return Ok(false);
        }
        // Retained invalid inputs stay owned for error reporting, but neither
        // the stale API nor the editor may publish them as an accepted view.
        self.logical_api_reset_pending = self.has_project_session();
        self.logical_workspace_overlays = true;
        self.retire_original_diagnosing_session()?;
        self.retain_logical_workspace_revision()?;
        Ok(true)
    }

    pub(in crate::lsp_client) fn activate_editor_overlay_fallback(&mut self) -> Result<(), String> {
        if self.try_logical_workspace_overlays()? {
            Ok(())
        } else {
            self.activate_materialized_project_session()
        }
    }

    pub(in crate::lsp_client) fn retain_logical_workspace_revision(
        &mut self,
    ) -> Result<(), String> {
        self.validate_logical_workspace_documents(&self.document_texts)?;
        let uris = self.document_texts.keys().cloned().collect::<Vec<_>>();
        for uri in uris {
            self.remember_session_document_uri(&uri, uri.clone());
        }
        self.editor_lsp_documents_dirty = true;
        self.diagnostics.clear();
        Ok(())
    }

    pub(in crate::lsp_client) fn validate_logical_workspace_documents(
        &self,
        documents: &FxHashMap<String, String>,
    ) -> Result<(), String> {
        if !self.logical_workspace_overlays {
            return Ok(());
        }
        // Validate the whole immutable input view before changing its mapping
        // or issuing any native notification. Outside-root imports remain valid
        // inputs: configured roots and semantic program membership differ.
        for uri in documents.keys() {
            let path = file_uri_to_path(uri)
                .filter(|path| path.is_absolute())
                .ok_or_else(|| cstr!("Logical Corsa workspace requires a file URI: {uri}"))?;
            if path
                .components()
                .any(|part| matches!(part, Component::ParentDir))
            {
                return Err(cstr!(
                    "Logical Corsa workspace path contains a parent component: {uri}"
                ));
            }
        }
        Ok(())
    }

    pub(in crate::lsp_client) fn close_logical_workspace_document(
        &mut self,
        uri: &str,
    ) -> Result<(), String> {
        self.document_texts.remove(uri);
        self.overlay_versions.remove(uri);
        if let Some(mapped) = self.session_document_uris.remove(uri) {
            self.external_document_uris.remove(mapped.as_str());
        }
        // The existing full synchronization sends didClose before the next
        // semantic request. Native readonly fallback then sees the real disk
        // file, or its real absence, without a materialized write or deletion.
        self.editor_lsp_documents_dirty = true;
        self.diagnostics.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_uri::path_to_file_uri;

    #[test]
    fn missing_closed_logical_queries_preserve_empty_answers_without_spawning() {
        let root = tempfile::tempdir().unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        client.logical_workspace_overlays = true;
        client.document_texts.insert(
            path_to_file_uri(&root.path().join("open.ts")),
            "export const value = 'dirty';".into(),
        );
        let before = client.document_texts.clone();
        let missing = path_to_file_uri(&root.path().join("missing.ts"));
        assert_eq!(client.hover_raw(&missing, 0, 0), Ok(None));
        assert_eq!(client.definition_raw(&missing, 0, 0), Ok(None));
        assert_eq!(client.references_raw(&missing, 0, 0, true), Ok(None));
        assert_eq!(client.document_texts, before);
        assert!(client.session.is_none());
        assert!(client.editor_lsp.is_none());
    }

    #[test]
    fn other_closed_logical_read_errors_retain_their_context_without_spawning() {
        let root = tempfile::tempdir().unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        client.logical_workspace_overlays = true;
        client.document_texts.insert(
            path_to_file_uri(&root.path().join("open.ts")),
            "export {};".into(),
        );
        let directory = path_to_file_uri(root.path());
        let read_error = std::fs::read_to_string(root.path()).unwrap_err();
        assert_eq!(
            client.hover_raw(&directory, 0, 0),
            Err(cstr!(
                "Failed to read closed logical Corsa query {directory}: {read_error}"
            ))
        );
        assert_eq!(client.document_texts.len(), 1);
        assert!(client.session.is_none());
        assert!(client.editor_lsp.is_none());
    }

    #[test]
    fn logical_fallback_retains_all_owned_names_and_close_uses_unchanged_disk() {
        let root = tempfile::tempdir().unwrap();
        let disk = root.path().join("support.ts");
        std::fs::write(&disk, "export const value = 1;").unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        client.authored_workspace = true;
        let main = path_to_file_uri(&root.path().join("App.vue.ts"));
        let support = path_to_file_uri(&disk);
        client
            .document_texts
            .insert(main.clone(), "import './support';".into());
        client
            .document_texts
            .insert(support.clone(), "export const value = 'dirty';".into());
        assert_eq!(client.try_logical_workspace_overlays(), Ok(true));
        assert_eq!(client.session_document_uri(&main), main);
        assert_eq!(client.session_document_uri(&support), support);
        assert!(!client.can_use_api_for_uri(&main));
        client.did_close(&support).unwrap();
        assert!(!client.can_use_api_for_uri(&main));
        assert_eq!(client.document_texts.len(), 1);
        assert_eq!(
            std::fs::read_to_string(disk).unwrap(),
            "export const value = 1;"
        );
        assert!(!root.path().join("App.vue.ts").exists());
        assert!(!client.overlay_storage.root().join("tsconfig.json").exists());
    }

    #[test]
    fn logical_fallback_refuses_unknown_schemes_before_mapping_any_document() {
        let root = tempfile::tempdir().unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        client.authored_workspace = true;
        client
            .document_texts
            .insert("untitled://App.vue.ts".into(), "export {};".into());
        assert_eq!(
            client.try_logical_workspace_overlays(),
            Err(cstr!(
                "Logical Corsa workspace requires a file URI: untitled://App.vue.ts"
            ))
        );
        assert!(client.logical_workspace_overlays);
        assert_eq!(
            client.validate_logical_workspace_documents(&client.document_texts),
            Err(cstr!(
                "Logical Corsa workspace requires a file URI: untitled://App.vue.ts"
            ))
        );
        assert!(client.session_document_uris.is_empty());
    }

    #[test]
    fn complete_owned_canon_projects_do_not_become_logical_workspace_fallbacks() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("__vize_vue_modules.d.ts"), "export {};").unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        client.authored_workspace = true;
        assert_eq!(client.try_logical_workspace_overlays(), Ok(false));
        assert!(!client.logical_workspace_overlays);
    }
    #[test]
    fn failed_readonly_api_reset_keeps_the_empty_revision_quarantined() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("tsconfig.json"), "{}").unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        client.executable = root
            .path()
            .join("missing-native-runtime")
            .to_string_lossy()
            .into();
        client.logical_workspace_overlays = true;
        client.logical_api_reset_pending = true;
        let uri = crate::file_uri::path_to_file_uri(&root.path().join("closed.ts"));
        let first = client.ensure_project_transport().unwrap_err();
        assert!(client.logical_api_reset_pending);
        assert!(client.logical_workspace_overlays);
        assert!(!client.can_use_api_for_uri(&uri));
        assert_eq!(client.ensure_project_transport(), Err(first));
        assert!(client.logical_api_reset_pending);
        assert!(client.logical_workspace_overlays);
    }
    #[cfg(windows)]
    #[test]
    fn canonical_root_and_file_uri_spelling_preserve_the_original_overlay_attempt() {
        // Keep the fixture alive independently of its canonical spelling.
        let work = tempfile::tempdir().unwrap();
        let root = work.path().canonicalize().unwrap();
        std::fs::write(root.join("App.vue"), "<template />").unwrap();
        let uri = path_to_file_uri(&root.join("App.vue.ts"));
        let storage = root.join("owned-overlay");
        assert_eq!(
            crate::lsp_client::session_paths::build_session_document_uri(
                &uri, &root, &storage, true
            ),
            uri
        );
        assert!(
            !crate::lsp_client::session_paths::is_within_project_namespace(
                &root.with_file_name("other-project").join("App.vue.ts"),
                &root
            )
        );
    }
    #[cfg(unix)]
    #[test]
    fn namespace_comparison_refuses_distinct_non_utf8_names_before_uri_encoding() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};
        let root = PathBuf::from(OsString::from_vec(b"/project/\x80".to_vec()));
        let path = PathBuf::from(OsString::from_vec(b"/project/\x81/App.vue.ts".to_vec()));
        assert!(!crate::lsp_client::session_paths::is_within_project_namespace(&path, &root));
        assert!(!crate::lsp_client::session_paths::is_within_project_namespace(&root, &root));
    }
}
