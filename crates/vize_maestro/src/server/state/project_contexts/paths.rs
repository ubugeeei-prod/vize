//! Select physical project boundaries while retaining authored document URIs.
use super::{Path, PathBuf, document_root};

pub(super) fn document_context_root(path: &Path, roots: &[PathBuf]) -> Option<PathBuf> {
    let physical = physical_path(path);
    roots
        .iter()
        .filter_map(|root| {
            let physical_root = physical_path(root);
            physical
                .strip_prefix(&physical_root)
                .ok()
                // Keep one boundary spelling per registered workspace root.
                .map(|relative| {
                    (
                        root,
                        root.join(relative),
                        physical_root.components().count(),
                    )
                })
        })
        .max_by_key(|(_, _, depth)| *depth)
        .map(|(root, source, _)| document_root(&source, root))
        .or_else(|| {
            roots
                .iter()
                .filter(|root| path.starts_with(root))
                .max_by_key(|root| root.components().count())
                .map(|root| document_root(path, root))
        })
}

pub(super) fn physical_path(path: &Path) -> PathBuf {
    for ancestor in path.ancestors() {
        if let Ok(physical) = ancestor.canonicalize()
            && let Ok(relative) = path.strip_prefix(ancestor)
        {
            // Unsaved or newly renamed documents can have no file yet.
            return crate::ide::normalize_physical_path(physical.join(relative));
        }
    }
    path.to_path_buf()
}

#[cfg(all(test, unix))]
mod tests {
    use super::{document_context_root, physical_path};

    #[test]
    fn physical_aliases_and_missing_buffers_share_the_registered_project_boundary() {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("actual");
        let alias = fixture.path().join("editor");
        let package = root.join("packages/a");
        std::fs::create_dir_all(package.join("src")).unwrap();
        std::fs::write(package.join("vite.config.mjs"), "export default {};").unwrap();
        std::os::unix::fs::symlink(&root, &alias).unwrap();
        let source = alias.join("packages/a/src/New.vue");
        assert_eq!(
            document_context_root(&source, std::slice::from_ref(&root)),
            Some(package.clone())
        );
        assert_eq!(
            document_context_root(&physical_path(&source), std::slice::from_ref(&alias)),
            Some(alias.join("packages/a"))
        );
        std::fs::write(&source, "<template />").unwrap();
        assert_eq!(document_context_root(&source, &[root]), Some(package));
    }

    #[test]
    fn config_watch_alias_retires_the_open_authored_uri_owner() {
        use super::super::{Arc, ServerState, Url};
        use tower_lsp::lsp_types::{FileChangeType, FileEvent};
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("actual");
        let alias = fixture.path().join("editor");
        let package = root.join("packages/a");
        std::fs::create_dir_all(package.join("src")).unwrap();
        let config = package.join("vize.config.json");
        std::fs::write(&config, r#"{"lsp":{"hover":false}}"#).unwrap();
        std::os::unix::fs::symlink(&root, &alias).unwrap();
        let owner = Arc::new(ServerState::new());
        owner.set_workspace_root(alias.clone());
        owner.load_workspace_config(&alias);
        let uri = Url::from_file_path(alias.join("packages/a/src/New.vue")).unwrap();
        owner
            .documents
            .open(uri.clone(), "<template />".into(), 1, "vue".into());
        let previous = owner.document_project_state(&uri).unwrap();
        assert!(!previous.lsp_features().hover);
        std::fs::write(&config, r#"{"lsp":{"hover":true}}"#).unwrap();
        assert_eq!(
            owner.observe_project_config_events(&[FileEvent {
                uri: Url::from_file_path(physical_path(&config)).unwrap(),
                typ: FileChangeType::CHANGED,
            }]),
            vec![(uri.clone(), 1)]
        );
        assert!(previous.project_context_retired());
        let current = owner.document_project_state(&uri).unwrap();
        assert!(current.lsp_features().hover);
        assert!(!Arc::ptr_eq(&previous, &current));
    }
}
