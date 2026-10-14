//! Retained editor buffers belong to the owner, not to one native snapshot.

use super::{CorsaProjectClient, materialize_session_document, uri_document_identifier};
use crate::file_uri::file_uri_to_path;
use corsa::{
    api::{FileChangeSummary, FileChanges},
    runtime::block_on,
};
use vize_l0::{String, cstr};

impl CorsaProjectClient {
    pub(in crate::lsp_client) fn synchronize_materialized_project_documents(
        &mut self,
        documents: &[(&str, &str)],
    ) -> Result<(), String> {
        // Startup must see the complete batch, including a typed overlay API
        // fallback while replay temporarily owns the retained document map.
        for (uri, content) in documents {
            let previous = self.document_texts.insert((*uri).into(), (*content).into());
            if previous.as_deref() != Some(*content) {
                self.editor_lsp_documents_dirty = true;
            }
        }
        if self.synchronize_existing_canon_project_documents(documents)? {
            return Ok(());
        }
        self.activate_materialized_project_session()?;
        let mut summary = FileChangeSummary::default();
        for (uri, content) in documents {
            self.clear_document_state(uri);
            let document_uri = self.materialized_session_document_uri(uri).ok_or_else(|| {
                cstr!("Failed to derive materialized Corsa overlay path for {uri}")
            })?;
            if let Some(FileChanges::Summary(delta)) =
                materialize_session_document(uri, document_uri.as_str(), content)
            {
                summary.changed.extend(delta.changed);
                summary.created.extend(delta.created);
            }
        }
        // All new members now exist before one owner reload; content-only
        // batches use one refresh instead of refreshing each document.
        self.refresh_materialized_overlay_snapshot(Some(FileChanges::Summary(summary)))
    }

    fn synchronize_existing_canon_project_documents(
        &mut self,
        documents: &[(&str, &str)],
    ) -> Result<bool, String> {
        // These files already belong to Canon's complete generated project.
        // A second overlay root would change their identity and detach closed
        // incoming scripts from the configured native answer.
        if self.materialized_project_session
            || !self.project_root.join("__vize_vue_modules.d.ts").is_file()
        {
            return Ok(false);
        }
        let Some(paths) = documents
            .iter()
            .map(|(uri, _)| file_uri_to_path(uri))
            .collect::<Option<Vec<_>>>()
        else {
            return Ok(false);
        };
        if !paths
            .iter()
            .all(|path| path.starts_with(&self.project_root) && path.is_file())
        {
            return Ok(false);
        }
        let mut summary = FileChangeSummary::default();
        for ((uri, content), path) in documents.iter().zip(paths) {
            self.clear_document_state(uri);
            if std::fs::read_to_string(&path).ok().as_deref() != Some(*content) {
                std::fs::write(&path, content).map_err(|error| {
                    cstr!(
                        "Failed to update Canon document {}: {error}",
                        path.display()
                    )
                })?;
                summary.changed.push(uri_document_identifier(uri));
            }
            self.remember_session_document_uri(uri, (*uri).into());
        }
        if !summary.changed.is_empty() {
            self.refresh_materialized_overlay_snapshot(Some(FileChanges::Summary(summary)))?;
        }
        Ok(true)
    }

    pub(in crate::lsp_client) fn refresh_materialized_overlay_snapshot(
        &mut self,
        file_changes: Option<FileChanges>,
    ) -> Result<(), String> {
        if !self.has_project_session() {
            return Ok(());
        }
        if matches!(&file_changes, Some(FileChanges::Summary(summary)) if !summary.created.is_empty())
        {
            return self.reload_materialized_project_transport();
        }
        block_on(self.project_session_mut()?.refresh(file_changes))
            .map_err(|error| cstr!("Failed to refresh Corsa snapshot: {error}"))
    }

    pub(in crate::lsp_client) fn reload_materialized_project_transport(
        &mut self,
    ) -> Result<(), String> {
        // A native summary refresh does not discover a new include member.
        // Reap the old physical owner before opening the complete disk project;
        // content-only edits continue to refresh its existing snapshot.
        let root = self.editor_lsp_project_root();
        let config = self
            .explicit_project_config
            .clone()
            .unwrap_or_else(|| crate::lsp_client::lifecycle_setup::workspace_config_path(&root));
        self.retire_project_session()?;
        self.replace_project_transport(&config, &root)
    }

    pub(in crate::lsp_client) fn ensure_project_transport(&mut self) -> Result<(), String> {
        if !self.closed && self.shared_editor_transport && !self.has_project_session() {
            crate::corsa_bridge::native_operation::checkpoint()?;
            self.ensure_editor_transport()?;
        }
        Ok(())
    }

    pub(in crate::lsp_client) fn replay_retained_project_documents(
        &mut self,
    ) -> Result<(), String> {
        // The new native snapshot has never received these bytes. Temporarily
        // take the retained map so the ordinary batch writer cannot treat an
        // unchanged buffer as an already-synchronized native overlay. Reuse its
        // single-refresh and typed materialization fallback, preserving every
        // buffer even when synchronization fails partway through the batch.
        let retained = std::mem::take(&mut self.document_texts);
        let documents = retained
            .iter()
            .map(|(uri, content)| (uri.as_str(), content.as_str()))
            .collect::<Vec<_>>();
        let result = self.did_open_batch_fast(&documents);
        for (uri, content) in retained {
            self.document_texts.entry(uri).or_insert(content);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::CorsaProjectClient;

    fn materialized_batch_fixture() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "../../../../../tests/_fixtures/differential/config/materialized-two-buffer-topology-8476.json"
        ))
        .expect("materialized two-buffer fixture must be valid JSON")
    }

    #[test]
    fn materialized_batch_retains_every_input_before_a_preparation_failure() {
        let root = tempfile::tempdir().unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        let fixture = materialized_batch_fixture();
        let mut documents = fixture["documents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|document| {
                (
                    document["uri"].as_str().unwrap(),
                    document["unsaved"].as_str().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        documents.push((
            fixture["preparationFailure"]["uri"].as_str().unwrap(),
            fixture["preparationFailure"]["text"].as_str().unwrap(),
        ));
        client.editor_lsp_documents_dirty = false;
        assert!(
            client
                .synchronize_materialized_project_documents(&documents)
                .is_err()
        );
        assert_eq!(client.document_texts.len(), documents.len());
        for (uri, source) in documents {
            assert_eq!(client.document_texts.get(uri).unwrap(), source);
        }
        assert!(client.editor_lsp_documents_dirty);
        assert!(!client.materialized_project_session);
        assert!(client.session.is_none());
        assert!(client.editor_lsp.is_none());
    }

    #[test]
    fn unchanged_materialized_batch_preserves_clean_editor_state_without_a_transport() {
        let root = tempfile::tempdir().unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        let fixture = materialized_batch_fixture();
        let documents = fixture["documents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|document| {
                (
                    document["uri"].as_str().unwrap(),
                    document["unsaved"].as_str().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        client
            .synchronize_materialized_project_documents(&documents)
            .unwrap();
        assert_eq!(client.document_texts.len(), documents.len());
        for (uri, source) in &documents {
            assert_eq!(client.document_texts.get(*uri).unwrap(), *source);
        }
        let retained = client.document_texts.clone();
        let mappings = client.session_document_uris.clone();
        client.editor_lsp_documents_dirty = false;
        client
            .synchronize_materialized_project_documents(&documents)
            .unwrap();
        assert!(!client.editor_lsp_documents_dirty);
        assert_eq!(client.document_texts, retained);
        assert_eq!(client.session_document_uris, mappings);
        assert!(client.materialized_project_session);
        assert!(client.session.is_none());
        assert!(client.editor_lsp.is_none());
    }

    #[test]
    fn editor_only_replay_retains_every_complete_buffer_without_starting_a_transport() {
        let root = tempfile::tempdir().unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        let buffers = [
            (
                "file:///workspace/first.ts",
                "export const first = 'unsaved';\n",
            ),
            ("file:///workspace/second.ts", "export const second = 42;\n"),
        ];
        client.did_open_batch_fast(&buffers).unwrap();
        let original = client.document_texts.clone();
        client.replay_retained_project_documents().unwrap();
        assert_eq!(client.document_texts, original);
        assert!(client.session.is_none());
        assert!(client.editor_lsp.is_none());
        assert!(client.editor_lsp_documents_dirty);
    }

    #[test]
    fn failed_replay_restores_every_complete_retained_buffer() {
        let root = tempfile::tempdir().unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        client
            .document_texts
            .insert("invalid-uri".into(), "first\n".into());
        client
            .document_texts
            .insert("another-invalid-uri".into(), "second\n".into());
        let original = client.document_texts.clone();
        client.materialized_project_session = true;
        assert!(client.replay_retained_project_documents().is_err());
        assert_eq!(client.document_texts, original);
        assert!(client.session.is_none());
        assert!(client.editor_lsp.is_none());
    }
}
