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
        let Some(file_changes) = materialized_file_changes(changed, created, deleted) else {
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
) -> Option<FileChanges> {
    if changed.is_empty() && created.is_empty() && deleted.is_empty() {
        return None;
    }

    Some(FileChanges::Summary(FileChangeSummary {
        changed: document_identifiers(changed),
        created: document_identifiers(created),
        deleted: document_identifiers(deleted),
    }))
}

fn document_identifiers(paths: &[PathBuf]) -> Vec<DocumentIdentifier> {
    let mut paths: Vec<_> = paths
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    paths.sort();
    paths.dedup();
    paths.into_iter().map(DocumentIdentifier::from).collect()
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

        let FileChanges::Summary(summary) =
            materialized_file_changes(&changed, &created, &deleted).expect("changes should exist")
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
        assert!(materialized_file_changes(&[], &[], &[]).is_none());
    }
}
