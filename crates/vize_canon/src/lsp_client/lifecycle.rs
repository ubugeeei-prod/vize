use super::{
    CorsaProjectClient,
    bootstrap::resolve_corsa_executable,
    lifecycle_setup::{
        cleanup_stale_sessions, install_node_modules_link, write_session_meta,
        write_shared_helper_decls, write_temp_tsconfig, write_vue_module_stubs,
    },
    paths::resolve_temp_dir_base,
    session::{materialize_session_document, uri_document_identifier},
    virtual_overlay,
};
use corsa::{
    CorsaError,
    api::{FileChangeSummary, FileChanges, OverlayChanges, OverlayUpdate},
    runtime::block_on,
};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};
use vize_carton::{String, cstr};

impl CorsaProjectClient {
    /// Start a Corsa project session rooted at an isolated scratch workspace.
    pub fn new(corsa_path: Option<&str>, working_dir: Option<&str>) -> Result<Self, String> {
        let executable = resolve_corsa_executable(corsa_path, working_dir)?;

        let project_root = working_dir
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .and_then(|path| path.canonicalize().ok());

        static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(0);

        let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
        let temp_dir_base = resolve_temp_dir_base(project_root.as_deref());
        let temp_dir_path = temp_dir_base.join(&*cstr!("{}-{}", std::process::id(), client_id));

        cleanup_stale_sessions(&temp_dir_base);
        let _ = std::fs::remove_dir_all(&temp_dir_path);
        std::fs::create_dir_all(&temp_dir_path)
            .map_err(|e| cstr!("Failed to create Corsa session directory: {e}"))?;

        write_session_meta(&temp_dir_path)?;
        install_node_modules_link(project_root.as_deref(), &temp_dir_path);
        write_vue_module_stubs(&temp_dir_path)?;
        write_shared_helper_decls(&temp_dir_path)?;
        write_temp_tsconfig(&temp_dir_path)?;

        let temp_root = temp_dir_path.canonicalize().ok();
        Self::spawn_initialized_client(
            executable.as_str(),
            temp_dir_path,
            temp_root,
            Some(temp_dir_base.join(&*cstr!("{}-{}", std::process::id(), client_id))),
        )
    }

    /// Start a Corsa project session rooted at an on-disk workspace.
    pub fn new_for_workspace(
        corsa_path: Option<&str>,
        workspace_root: &Path,
    ) -> Result<Self, String> {
        let workspace_root = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());
        let working_dir = workspace_root.to_string_lossy();
        let executable = resolve_corsa_executable(corsa_path, Some(working_dir.as_ref()))?;

        Self::spawn_initialized_client(
            executable.as_str(),
            workspace_root.clone(),
            Some(workspace_root),
            None,
        )
    }

    /// Shutdown the project session.
    pub fn shutdown(&mut self) -> Result<(), String> {
        if self.closed {
            return Ok(());
        }

        let _ = corsa::runtime::block_on(self.session.close());
        self.document_texts.clear();
        self.diagnostics.clear();
        self.overlay_versions.clear();
        self.closed = true;
        Ok(())
    }

    /// Open a virtual document.
    pub fn did_open(&mut self, uri: &str, content: &str) -> Result<(), String> {
        self.did_open_fast(uri, content)
    }

    /// Open or replace a virtual document overlay.
    pub fn did_open_fast(&mut self, uri: &str, content: &str) -> Result<(), String> {
        self.clear_document_state(uri);
        self.sync_overlay_document(uri, content)
    }

    /// Open many virtual document overlays with a single snapshot refresh when possible.
    pub fn did_open_batch_fast(&mut self, documents: &[(&str, &str)]) -> Result<(), String> {
        if documents.is_empty() {
            return Ok(());
        }

        if !self.supports_overlay_api() {
            for (uri, content) in documents {
                self.clear_document_state(uri);
                self.sync_overlay_document(uri, content)?;
            }
            return Ok(());
        }

        let mut summary = FileChangeSummary::default();
        let mut overlay_upserts = Vec::new();
        let mut changed = false;
        for (uri, content) in documents {
            self.clear_document_state(uri);
            let previous = self.document_texts.insert((*uri).into(), (*content).into());

            let document_uri = self.session_document_uri(uri);
            if previous.as_deref() == Some(*content) {
                continue;
            }
            changed = true;
            merge_materialized_file_changes(
                &mut summary,
                materialize_session_document(uri, document_uri.as_str(), content).or_else(|| {
                    virtual_overlay::upsert_file_changes(
                        uri,
                        document_uri.as_str(),
                        &self.project_root,
                        previous.is_some(),
                    )
                }),
            );
            if document_uri == *uri {
                let version = self
                    .overlay_versions
                    .get(*uri)
                    .copied()
                    .unwrap_or(0)
                    .saturating_add(1);
                self.overlay_versions.insert((*uri).into(), version);
                overlay_upserts.push(OverlayUpdate {
                    document: uri_document_identifier(document_uri.as_str()),
                    text: (*content).into(),
                    version: Some(version),
                    language_id: Some("typescript".into()),
                });
            }
        }

        if !changed {
            return Ok(());
        }

        let file_changes = if summary.changed.is_empty()
            && summary.created.is_empty()
            && summary.deleted.is_empty()
        {
            None
        } else {
            Some(FileChanges::Summary(summary))
        };

        if overlay_upserts.is_empty() {
            return block_on(self.session.refresh(file_changes))
                .map_err(|error| cstr!("Failed to refresh Corsa snapshot: {error}"));
        }

        match block_on(self.session.refresh_with_overlay_changes(
            file_changes,
            Some(OverlayChanges {
                upsert: overlay_upserts,
                delete: Vec::new(),
            }),
        )) {
            Ok(()) => Ok(()),
            Err(CorsaError::Unsupported(_)) => {
                self.overlay_api_disabled = true;
                for (uri, _) in documents {
                    self.document_texts.remove(*uri);
                    self.overlay_versions.remove(*uri);
                }
                for (uri, content) in documents {
                    self.clear_document_state(uri);
                    self.sync_overlay_document(uri, content)?;
                }
                Ok(())
            }
            Err(error) => Err(cstr!("Failed to refresh Corsa snapshot: {error}")),
        }
    }

    /// Update an already-open virtual document overlay.
    pub fn did_change(&mut self, uri: &str, content: &str) -> Result<(), String> {
        self.clear_document_state(uri);
        self.sync_overlay_document(uri, content)
    }

    /// Close a virtual document overlay.
    pub fn did_close(&mut self, uri: &str) -> Result<(), String> {
        self.delete_overlay_document(uri)?;
        self.clear_document_state(uri);
        Ok(())
    }

    pub(crate) fn diagnostics_cache_len(&self) -> usize {
        self.diagnostics.len()
    }

    pub(crate) fn clear_diagnostics_cache(&mut self) {
        self.diagnostics.clear();
    }

    /// Compatibility no-op for older call sites that expected publishDiagnostics.
    pub fn wait_for_diagnostics(&mut self, _expected_documents: usize) {}

    pub(super) fn clear_document_state(&mut self, uri: &str) {
        self.diagnostics.remove(uri);
    }
}

impl Drop for CorsaProjectClient {
    fn drop(&mut self) {
        let _ = self.shutdown();
        if let Some(ref dir) = self.temp_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

fn merge_materialized_file_changes(
    summary: &mut FileChangeSummary,
    file_changes: Option<FileChanges>,
) {
    let Some(FileChanges::Summary(file_changes)) = file_changes else {
        return;
    };

    summary.changed.extend(file_changes.changed);
    summary.created.extend(file_changes.created);
    summary.deleted.extend(file_changes.deleted);
}

#[cfg(test)]
mod tests {
    use super::merge_materialized_file_changes;
    use corsa::api::{DocumentIdentifier, FileChangeSummary, FileChanges};

    #[test]
    fn merges_materialized_file_change_summaries() {
        let mut summary = FileChangeSummary::default();
        merge_materialized_file_changes(
            &mut summary,
            Some(FileChanges::Summary(FileChangeSummary {
                changed: vec![DocumentIdentifier::from("/workspace/a.ts")],
                created: vec![DocumentIdentifier::from("/workspace/b.ts")],
                deleted: Vec::new(),
            })),
        );
        merge_materialized_file_changes(
            &mut summary,
            Some(FileChanges::Summary(FileChangeSummary {
                changed: vec![DocumentIdentifier::from("/workspace/c.ts")],
                created: Vec::new(),
                deleted: vec![DocumentIdentifier::from("/workspace/d.ts")],
            })),
        );

        assert_eq!(summary.changed.len(), 2);
        assert_eq!(summary.created.len(), 1);
        assert_eq!(summary.deleted.len(), 1);
    }
}
