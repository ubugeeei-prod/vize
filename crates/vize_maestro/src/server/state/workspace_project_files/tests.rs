use super::{FileChangeType, FileEvent, ServerState, Url};

#[test]
fn project_inventory_is_lazy_reuses_paths_and_reads_current_closed_or_dirty_text() {
    crate::runtime::block_on(async {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("module.ts");
        std::fs::write(&path, "export const disk = 1;").unwrap();
        let uri = Url::from_file_path(&path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(root.path().to_owned());
        assert!(state.workspace_project_files.paths.read().is_none());
        assert_eq!(
            state.discover_workspace_project_sources().await,
            vec![(uri.clone(), "export const disk = 1;".into())]
        );
        state.documents.open(
            uri.clone(),
            "export const dirty = 2;".into(),
            1,
            "typescript".into(),
        );
        std::fs::write(&path, "export const saved = 3;").unwrap();
        assert_eq!(
            state.discover_workspace_project_sources().await,
            vec![(uri.clone(), "export const dirty = 2;".into())]
        );
        state.documents.close(&uri);
        assert_eq!(
            state.discover_workspace_project_sources().await,
            vec![(uri, "export const saved = 3;".into())]
        );
        let added = root.path().join("Added.vue");
        std::fs::write(&added, "<template />").unwrap();
        assert_eq!(state.discover_workspace_project_sources().await.len(), 1);
        state.track_workspace_vue_files(Url::from_file_path(&added).unwrap().as_str());
        assert_eq!(state.discover_workspace_project_sources().await.len(), 2);
        std::fs::remove_file(&added).unwrap();
        state.forget_workspace_vue_files(Url::from_file_path(&added).unwrap().as_str());
        assert_eq!(state.discover_workspace_project_sources().await.len(), 1);
        let new_script = root.path().join("created.ts");
        std::fs::write(&new_script, "created").unwrap();
        let event = FileEvent {
            uri: Url::from_file_path(&new_script).unwrap(),
            typ: FileChangeType::CHANGED,
        };
        state.observe_workspace_project_file_events(std::slice::from_ref(&event));
        assert_eq!(state.discover_workspace_project_sources().await.len(), 1);
        state.observe_workspace_project_file_events(&[FileEvent {
            typ: FileChangeType::CREATED,
            ..event
        }]);
        assert_eq!(state.discover_workspace_project_sources().await.len(), 2);
        let next_root = tempfile::tempdir().unwrap();
        state.set_workspace_root(next_root.path().to_owned());
        assert_eq!(state.discover_workspace_project_sources().await, Vec::new());
    });
}

#[test]
fn project_inventory_preserves_multiple_roots_and_metadata_dependency_exclusions() {
    crate::runtime::block_on(async {
        let root = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        for file in [
            "src/module.ts",
            ".github/Workflow.vue",
            ".git/Hidden.ts",
            "node_modules/Hidden.ts",
            "target/Hidden.vue",
        ] {
            let path = root.path().join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "source").unwrap();
        }
        let other_file = other.path().join("other.mjs");
        std::fs::write(&other_file, "other").unwrap();
        let state = ServerState::new();
        state.set_workspace_root(root.path().to_owned());
        state.set_workspace_folders(vec![root.path().to_owned(), other.path().to_owned()]);
        let mut expected = vec![
            (
                Url::from_file_path(root.path().join("src/module.ts")).unwrap(),
                "source".into(),
            ),
            (
                Url::from_file_path(root.path().join(".github/Workflow.vue")).unwrap(),
                "source".into(),
            ),
            (Url::from_file_path(other_file).unwrap(), "other".into()),
        ];
        expected.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));
        assert_eq!(state.discover_workspace_project_sources().await, expected);
    });
}

#[test]
fn structural_workspace_symbols_need_file_events_without_enabling_the_checker() {
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(
        &serde_json::json!({"editor":true,"typecheck":false}),
    ));
    assert!(!state.is_lsp_typecheck_enabled());
    assert!(state.project_source_watcher_enabled());
    assert!(state.workspace_project_files.paths.read().is_none());
    state.apply_lsp_initialization_options(Some(
        &serde_json::json!({"editor":false,"typecheck":false}),
    ));
    assert!(!state.project_source_watcher_enabled());
    assert!(state.workspace_project_files.paths.read().is_none());
}

#[test]
fn failed_project_walk_retains_available_sources_and_retries_without_an_event() {
    crate::runtime::block_on(async {
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("not-created-yet");
        let available = root.path().join("existing.ts");
        std::fs::write(&available, "export const existing = 1;").unwrap();
        let state = ServerState::new();
        state.set_workspace_root(root.path().to_owned());
        state.set_workspace_folders(vec![missing.clone()]);
        let mut expected = vec![(
            Url::from_file_path(&available).unwrap(),
            "export const existing = 1;".into(),
        )];
        assert_eq!(state.discover_workspace_project_sources().await, expected);
        assert!(state.workspace_project_files.paths.read().is_none());
        std::fs::create_dir(&missing).unwrap();
        let created = missing.join("created.ts");
        std::fs::write(&created, "export const created = 2;").unwrap();
        expected.push((
            Url::from_file_path(created).unwrap(),
            "export const created = 2;".into(),
        ));
        expected.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));
        assert_eq!(state.discover_workspace_project_sources().await, expected);
        assert!(state.workspace_project_files.paths.read().is_some());
    });
}

#[test]
fn cached_project_read_retries_in_flight_root_folder_and_membership_changes() {
    crate::runtime::block_on(async {
        for change in ["root", "folders", "created", "deleted"] {
            let root = tempfile::tempdir().unwrap();
            let next = tempfile::tempdir().unwrap();
            let path = root.path().join("original.ts");
            let added = next.path().join("added.ts");
            std::fs::write(&path, "original").unwrap();
            std::fs::write(&added, "added").unwrap();
            let uri = Url::from_file_path(&path).unwrap();
            let added_uri = Url::from_file_path(&added).unwrap();
            let state = ServerState::new();
            state.set_workspace_root(root.path().to_owned());
            assert_eq!(
                state.discover_workspace_project_sources().await,
                vec![(uri.clone(), "original".into())]
            );
            let (reached, waiting) = futures::channel::oneshot::channel();
            let (resume, paused) = futures::channel::oneshot::channel();
            *state.workspace_project_files.read_pause.write() = Some((reached, paused));
            let mutation = async {
                waiting.await.unwrap();
                let mut expected = vec![(uri.clone(), "original".into())];
                match change {
                    "root" => {
                        state.set_workspace_root(next.path().to_owned());
                        expected = vec![(added_uri.clone(), "added".into())];
                    }
                    "folders" => {
                        state.set_workspace_folders(vec![next.path().to_owned()]);
                        expected.push((added_uri.clone(), "added".into()));
                    }
                    "created" => {
                        let created = root.path().join("created.ts");
                        std::fs::write(&created, "created").unwrap();
                        let created_uri = Url::from_file_path(created).unwrap();
                        state.observe_workspace_project_file_events(&[FileEvent {
                            uri: created_uri.clone(),
                            typ: FileChangeType::CREATED,
                        }]);
                        expected.push((created_uri, "created".into()));
                    }
                    "deleted" => {
                        std::fs::remove_file(&path).unwrap();
                        state.observe_workspace_project_file_events(&[FileEvent {
                            uri: uri.clone(),
                            typ: FileChangeType::DELETED,
                        }]);
                        expected.clear();
                    }
                    _ => unreachable!(),
                }
                expected.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));
                resume.send(()).unwrap();
                expected
            };
            let (actual, expected) =
                futures::join!(state.discover_workspace_project_sources(), mutation);
            assert_eq!(actual, expected, "{change}");
        }
    });
}
