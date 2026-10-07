use super::{FileChangeType, FileEvent, ServerState, Url};

#[test]
fn project_inventory_file_operations_retire_stale_disk_namespaces_until_recreation() {
    crate::runtime::block_on(async {
        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join("components");
        std::fs::create_dir(&directory).unwrap();
        let old = directory.join("Old.vue");
        let module = directory.join("module.ts");
        let outside = root.path().join("outside.js");
        let new = root.path().join("New.vue");
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/lsp/project-navigation-8013/original-symbol-consumers/OldCard.vue.txt"
        ));
        let changed = source.replace("movedLabel", "unsavedLabel");
        let dirty = changed.as_str();
        for (path, text) in [(&old, source), (&module, "module"), (&outside, "outside")] {
            std::fs::write(path, text).unwrap();
        }
        let uri = |path: &std::path::Path| Url::from_file_path(path).unwrap();
        let old_uri = uri(&old);
        let new_uri = uri(&new);
        let sorted = |mut sources: Vec<(Url, std::string::String)>| {
            sources.sort_by(|(a, _), (b, _)| a.as_str().cmp(b.as_str()));
            sources
        };
        let originals = sorted(vec![
            (old_uri.clone(), source.into()),
            (uri(&module), "module".into()),
            (uri(&outside), "outside".into()),
        ]);
        let state = ServerState::new();
        state.set_workspace_root(root.path().to_owned());
        assert_eq!(state.discover_workspace_project_sources().await, originals);
        state
            .documents
            .open(old_uri.clone(), dirty.into(), 1, "vue".into());
        state.forget_workspace_vue_files(old_uri.as_str());
        state.track_workspace_vue_files(new_uri.as_str());
        assert!(state.rename_document(&old_uri, new_uri.clone()));
        assert!(old.is_file());
        assert!(!new.exists());
        assert_eq!(
            state.discover_workspace_project_sources().await,
            sorted(vec![
                (new_uri.clone(), dirty.into()),
                (uri(&module), "module".into()),
                (uri(&outside), "outside".into()),
            ])
        );
        state.documents.close(&new_uri);
        assert_eq!(
            state.discover_workspace_project_sources().await,
            sorted(vec![
                (uri(&module), "module".into()),
                (uri(&outside), "outside".into())
            ])
        );
        state.observe_workspace_project_file_events(&[FileEvent {
            uri: old_uri,
            typ: FileChangeType::CREATED,
        }]);
        assert_eq!(state.discover_workspace_project_sources().await, originals);
        state.forget_workspace_vue_files(uri(&directory).as_str());
        assert_eq!(
            state.discover_workspace_project_sources().await,
            vec![(uri(&outside), "outside".into())]
        );
        state.track_workspace_vue_files(uri(&directory).as_str());
        assert_eq!(state.discover_workspace_project_sources().await, originals);
    });
}
