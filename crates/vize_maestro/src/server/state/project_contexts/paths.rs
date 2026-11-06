//! Select physical project boundaries while retaining authored document URIs.
use super::{
    Path, PathBuf, document_root,
    root_discovery::{ProjectRootCache, cached_document_root},
};

pub(super) fn document_context_root(path: &Path, roots: &[PathBuf]) -> Option<PathBuf> {
    document_context_root_with(path, roots, document_root)
}

pub(super) fn cached_document_context_root(
    path: &Path,
    roots: &[PathBuf],
    cache: &ProjectRootCache,
) -> Option<PathBuf> {
    document_context_root_with(path, roots, |source, root| {
        cached_document_root(source, root, cache)
    })
}

fn document_context_root_with(
    path: &Path,
    roots: &[PathBuf],
    discover: impl Fn(&Path, &Path) -> PathBuf,
) -> Option<PathBuf> {
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
        .map(|(root, source, _)| discover(&source, root))
        .or_else(|| {
            roots
                .iter()
                .filter(|root| path.starts_with(root))
                .max_by_key(|root| root.components().count())
                .map(|root| discover(path, root))
        })
}

pub(super) fn physical_path(path: &Path) -> PathBuf {
    vize_carton::path::physical_path(path)
}

#[cfg(all(test, unix))]
mod tests {
    use super::{Path, cached_document_context_root, document_context_root, physical_path};
    use crate::server::state::project_contexts::root_discovery::ProjectRootCache;

    #[test]
    fn existing_files_remain_readable_file_paths_after_identity_normalization() {
        let fixture = tempfile::tempdir().unwrap();
        let config = fixture.path().join("tsconfig.json");
        std::fs::write(&config, "{}").unwrap();
        let normalized = physical_path(&config);
        assert!(
            normalized.is_file(),
            "a config identity must not append a directory separator"
        );
        assert_eq!(std::fs::read_to_string(&normalized).unwrap(), "{}");
        let alias = fixture.path().join("linked-tsconfig.json");
        std::os::unix::fs::symlink(&config, &alias).unwrap();
        let linked = physical_path(&alias);
        assert!(linked.is_file());
        assert_eq!(std::fs::read_to_string(linked).unwrap(), "{}");
        let directory = physical_path(fixture.path());
        assert!(directory.is_dir());
        assert!(std::fs::read_dir(&directory).unwrap().count() >= 2);
        let missing = physical_path(&fixture.path().join("new/Unsaved.vue"));
        assert!(!missing.exists());
        assert_eq!(
            missing.strip_prefix(directory).unwrap(),
            Path::new("new/Unsaved.vue")
        );
    }

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
    fn nested_vite_config_stays_in_the_ancestor_typescript_project() {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path();
        let gallery = root.join("gallery");
        std::fs::create_dir_all(&gallery).unwrap();
        std::fs::write(root.join("tsconfig.json"), "{}").unwrap();
        std::fs::write(root.join("vite.config.ts"), "export default { vue: {} }").unwrap();
        std::fs::write(gallery.join("vite.config.ts"), "export default {}").unwrap();

        assert_eq!(
            document_context_root(&gallery.join("Widget.vue"), &[root.to_path_buf()]),
            Some(root.to_path_buf())
        );
    }

    #[test]
    fn package_vite_config_stays_owner_under_workspace_tsconfig() {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path();
        let package = root.join("packages/a");
        std::fs::create_dir_all(package.join("src")).unwrap();
        std::fs::write(root.join("tsconfig.json"), "{}").unwrap();
        std::fs::write(package.join("package.json"), "{}").unwrap();
        std::fs::write(package.join("vite.config.ts"), "export default {}").unwrap();

        assert_eq!(
            document_context_root(&package.join("src/App.vue"), &[root.to_path_buf()]),
            Some(package)
        );
    }

    #[test]
    fn sibling_files_share_one_discovery_until_marker_invalidation() {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path();
        let package = root.join("packages/a");
        let source = package.join("src");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(package.join("package.json"), "{}").unwrap();
        let cache = ProjectRootCache::default();
        let roots = [root.to_path_buf()];

        assert_eq!(
            cached_document_context_root(&source.join("A.vue"), &roots, &cache),
            Some(package.clone())
        );
        assert_eq!(cache.len(), 1);
        std::fs::write(source.join("tsconfig.json"), "{}").unwrap();
        assert_eq!(
            cached_document_context_root(&source.join("B.vue"), &roots, &cache),
            Some(package),
            "a sibling request must reuse the directory result"
        );
        assert_eq!(cache.len(), 1);

        cache.clear();
        assert_eq!(
            cached_document_context_root(&source.join("B.vue"), &roots, &cache),
            Some(source)
        );
    }

    #[test]
    fn marker_event_clears_cached_roots_before_an_empty_reload_scope_returns() {
        use super::super::{Arc, ServerState, Url, watched::ConfigEventScope};

        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path();
        let package = root.join("packages/a");
        let source = package.join("src");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(package.join("package.json"), "{}").unwrap();
        let owner = Arc::new(ServerState::new());
        owner.set_workspace_root(root.to_path_buf());

        assert_eq!(
            owner.document_context_root(&source.join("A.vue")),
            Some(package)
        );
        let marker = source.join("tsconfig.json");
        std::fs::write(&marker, "{}").unwrap();
        let marker_uri = Url::from_file_path(&marker).unwrap();
        let changes = owner.apply_project_config_paths(
            std::slice::from_ref(&marker_uri),
            ConfigEventScope::for_topology(Vec::new(), vec![marker]),
        );
        assert!(changes.documents.is_empty());
        assert_eq!(
            owner.document_context_root(&source.join("B.vue")),
            Some(source)
        );
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
        let config = package.join("vite.config.mjs");
        std::fs::write(
            &config,
            r#"export default { vue: {"languageServer":{"hover":false}} };"#,
        )
        .unwrap();
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
        std::fs::write(
            &config,
            r#"export default { vue: {"languageServer":{"hover":true}} };"#,
        )
        .unwrap();
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
