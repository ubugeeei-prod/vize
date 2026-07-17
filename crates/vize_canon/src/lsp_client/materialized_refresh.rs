use super::CorsaProjectClient;
use corsa::{
    api::{DocumentIdentifier, FileChangeSummary, FileChanges},
    runtime::block_on,
};
use std::path::PathBuf;
use vize_carton::{String, cstr};

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
        let Some(file_changes) = materialized_file_changes(changed, created, deleted)? else {
            return Ok(());
        };

        self.clear_diagnostics_cache();
        block_on(self.session.refresh(Some(file_changes)))
            .map_err(|error| cstr!("Failed to refresh materialized Corsa files: {error}"))
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
    use super::materialized_file_changes;
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
}
