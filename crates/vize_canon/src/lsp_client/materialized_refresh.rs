use super::CorsaProjectClient;
use corsa::{
    api::{DocumentIdentifier, FileChangeSummary, FileChanges},
    runtime::block_on,
};
use std::path::PathBuf;
use vize_s0::{String, cstr};

use crate::file_uri::path_to_file_uri;

impl CorsaProjectClient {
    /// Refresh an on-disk project session after materialized files change.
    ///
    /// Paths are forwarded as filesystem identifiers so Corsa can invalidate
    /// the affected dependency graph without replacing the project session.
    pub fn refresh_materialized_files(
        &mut self,
        changed: &[PathBuf],
        created: &[PathBuf],
        deleted: &[PathBuf],
    ) -> Result<(), String> {
        self.purge_deleted_materialized_overlays(deleted)?;
        let Some(file_changes) = materialized_file_changes(changed, created, deleted)? else {
            return Ok(());
        };

        self.clear_diagnostics_cache();
        // The reusable editor session runs its own process against the mirror it
        // read when it built its program, and a project-session refresh does not
        // reach that copy. Retiring it keeps the next request, including the
        // batch diagnostics that fall back to this transport, on the files the
        // delta just wrote.
        self.retire_editor_lsp()?;
        if !self.has_project_session() {
            return Ok(());
        }
        block_on(self.project_session_mut()?.refresh(Some(file_changes)))
            .map_err(|error| cstr!("Failed to refresh materialized Corsa files: {error}"))
    }

    fn purge_deleted_materialized_overlays(&mut self, deleted: &[PathBuf]) -> Result<(), String> {
        let mut deleted = deleted.to_vec();
        deleted.sort();
        deleted.dedup();
        for path in deleted {
            let uri = path_to_file_uri(&path);
            if self.document_texts.contains_key(uri.as_str()) {
                self.delete_overlay_document(uri.as_str())?;
                continue;
            }
            self.overlay_versions.remove(uri.as_str());
            if let Some(mapped) = self.session_document_uris.remove(uri.as_str()) {
                self.external_document_uris.remove(mapped.as_str());
            }
            self.external_document_uris.remove(uri.as_str());
            self.diagnostics.remove(uri.as_str());
        }
        Ok(())
    }
}

fn materialized_file_changes(
    changed: &[PathBuf],
    created: &[PathBuf],
    deleted: &[PathBuf],
) -> Result<Option<FileChanges>, String> {
    if changed.is_empty() && created.is_empty() && deleted.is_empty() {
        return Ok(None);
    }

    Ok(Some(FileChanges::Summary(FileChangeSummary {
        changed: document_identifiers(changed)?,
        created: document_identifiers(created)?,
        deleted: document_identifiers(deleted)?,
    })))
}

fn document_identifiers(paths: &[PathBuf]) -> Result<Vec<DocumentIdentifier>, String> {
    let mut paths = paths.to_vec();
    paths.sort();
    paths.dedup();
    paths
        .into_iter()
        .map(|path| {
            let path_str = path.to_str().ok_or_else(|| {
                cstr!(
                    "Corsa cannot represent non-UTF-8 materialized path {:?}",
                    path.as_os_str()
                )
            })?;
            Ok(DocumentIdentifier::from(path_str))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{CorsaProjectClient, materialized_file_changes};
    use corsa::api::{DocumentIdentifier, FileChanges};
    use std::path::PathBuf;

    #[test]
    fn groups_deduplicated_materialized_paths_by_change_kind() {
        let changed = vec![
            PathBuf::from("/workspace/b.ts"),
            PathBuf::from("/workspace/a.ts"),
            PathBuf::from("/workspace/a.ts"),
        ];

        let created = vec![PathBuf::from("/workspace/created.ts")];
        let deleted = vec![PathBuf::from("/workspace/deleted.ts")];

        let FileChanges::Summary(summary) = materialized_file_changes(&changed, &created, &deleted)
            .expect("paths should be representable")
            .expect("changes should exist")
        else {
            panic!("expected a file-change summary");
        };

        assert_eq!(
            summary.changed,
            vec![
                DocumentIdentifier::from("/workspace/a.ts"),
                DocumentIdentifier::from("/workspace/b.ts"),
            ]
        );
        assert_eq!(
            summary.created,
            vec![DocumentIdentifier::from("/workspace/created.ts")]
        );
        assert_eq!(
            summary.deleted,
            vec![DocumentIdentifier::from("/workspace/deleted.ts")]
        );
    }

    #[test]
    fn skips_refresh_when_no_materialized_paths_changed() {
        assert!(
            materialized_file_changes(&[], &[], &[])
                .expect("empty changes should be valid")
                .is_none()
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_distinct_non_utf8_paths_instead_of_merging_them() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let first = PathBuf::from(OsString::from_vec(b"/workspace/\x80.ts".to_vec()));
        let second = PathBuf::from(OsString::from_vec(b"/workspace/\x81.ts".to_vec()));
        assert_ne!(first, second);
        assert_eq!(first.to_string_lossy(), second.to_string_lossy());

        let error = materialized_file_changes(&[first, second], &[], &[])
            .expect_err("lossy paths must not collapse into one refresh entry");
        assert!(error.contains("cannot represent non-UTF-8 materialized path"));
    }

    #[test]
    fn deleted_materialized_files_are_removed_from_every_overlay_identity_map() {
        let root = tempfile::tempdir().unwrap();
        let deleted = root.path().join("mirror/Deleted.vue.ts");
        let uri = crate::file_uri::path_to_file_uri(&deleted);
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        client
            .document_texts
            .insert(uri.clone(), "export const stale = true;".into());
        client.overlay_versions.insert(uri.clone(), 9);
        client
            .session_document_uris
            .insert(uri.clone(), uri.clone());
        client
            .external_document_uris
            .insert(uri.clone(), uri.clone());

        client
            .refresh_materialized_files(&[], &[], std::slice::from_ref(&deleted))
            .unwrap();

        assert!(!client.document_texts.contains_key(uri.as_str()));
        assert!(!client.overlay_versions.contains_key(uri.as_str()));
        assert!(!client.session_document_uris.contains_key(uri.as_str()));
        assert!(!client.external_document_uris.contains_key(uri.as_str()));
        assert!(client.editor_lsp_documents_dirty);
    }
}
