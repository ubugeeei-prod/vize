//! Marker topology changes preserve buffers and independent native owners.
use super::{Arc, ProjectContextKey, ServerState, Url, tests};
use futures::executor::block_on;

fn refresh(state: &Arc<ServerState>, uri: &Url) -> super::config_reload::ProjectConfigChanges {
    block_on(state.refresh_document_configuration_async(uri))
}

#[test]
fn nested_vite_and_tsconfig_markers_create_repair_and_delete_without_watch_notifications() {
    for (name, valid, invalid) in [
        (
            "vite.config.mjs",
            "export default {vue:{languageServer:{hover:true}}};",
            "export default { broken syntax",
        ),
        (
            "tsconfig.json",
            r#"{"vueCompilerOptions":{"target":2.7}}"#,
            "{ broken syntax",
        ),
    ] {
        let (fixture, state, _, b) = tests::fixture();
        let root = fixture.path().join("packages/a");
        let nested = root.join("src/Nested");
        std::fs::create_dir_all(&nested).unwrap();
        let a = Url::from_file_path(nested.join("App.vue")).unwrap();
        let c = Url::from_file_path(root.join("src/Control.vue")).unwrap();
        for uri in [&a, &c] {
            std::fs::write(uri.to_file_path().unwrap(), "<template>disk</template>").unwrap();
            state.documents.open(
                uri.clone(),
                "<template>unsaved</template>".into(),
                17,
                "vue".into(),
            );
            state.document_project_state(uri).unwrap();
        }
        assert!(refresh(&state, &a).errors.is_empty());
        let previous = state.document_project_state(&a).unwrap();
        let control = state.document_project_state(&c).unwrap();
        let sibling = state.document_project_state(&b).unwrap();
        assert!(Arc::ptr_eq(&previous, &control));
        previous.update_virtual_docs(&a, "<template>unsaved</template>");
        previous.update_virtual_docs(&c, "<template>unsaved</template>");
        let marker = nested.join(name);
        std::fs::write(&marker, invalid).unwrap();
        assert_eq!(refresh(&state, &a).errors.len(), 1);
        assert!(Arc::ptr_eq(
            &previous,
            &state.document_project_state(&a).unwrap()
        ));
        assert!(refresh(&state, &a).errors.is_empty());
        assert!(Arc::ptr_eq(
            &previous,
            &state.document_project_state(&a).unwrap()
        ));
        std::fs::write(&marker, valid).unwrap();
        assert!(refresh(&state, &a).errors.is_empty());
        let selected = state.document_project_state(&a).unwrap();
        assert!(!Arc::ptr_eq(&previous, &selected));
        assert!(selected.lsp_features().hover);
        assert_eq!(
            selected.type_checker_vue_version().as_str(),
            if name == "tsconfig.json" { "2.7" } else { "3" }
        );
        assert_eq!(
            selected.documents.text(&a).as_deref(),
            Some("<template>unsaved</template>")
        );
        assert_eq!(selected.documents.version(&a), Some(17));
        assert!(selected.get_virtual_docs(&a).is_some());
        assert!(previous.get_virtual_docs(&a).is_none());
        assert!(previous.get_virtual_docs(&c).is_some());
        assert!(!previous.project_context_retired());
        assert!(Arc::ptr_eq(
            &control,
            &state.document_project_state(&c).unwrap()
        ));
        assert!(Arc::ptr_eq(
            &sibling,
            &state.document_project_state(&b).unwrap()
        ));

        std::fs::remove_file(&marker).unwrap();
        assert!(refresh(&state, &a).errors.is_empty());
        let restored = state.document_project_state(&a).unwrap();
        assert!(Arc::ptr_eq(&control, &restored));
        assert!(!restored.lsp_features().hover);
        assert_eq!(restored.type_checker_vue_version().as_str(), "3");
        assert_eq!(
            restored.documents.text(&a).as_deref(),
            Some("<template>unsaved</template>")
        );
        assert_eq!(restored.documents.version(&a), Some(17));
        assert!(restored.get_virtual_docs(&a).is_some());
        assert!(selected.project_context_retired());
        assert!(
            !state
                .project_contexts
                .contexts
                .lock()
                .contains_key(&ProjectContextKey::from(nested.clone()))
        );
        assert!(
            !state
                .project_contexts
                .configuration_attempts
                .lock()
                .contains_key(&ProjectContextKey::from(nested.clone()))
        );
        assert!(Arc::ptr_eq(
            &sibling,
            &state.document_project_state(&b).unwrap()
        ));
    }
}

#[test]
fn rejected_changed_input_baselines_confirm_now_stable_initial_and_reload_revisions() {
    for initial_absence in [true, false] {
        let (fixture, state, a, _) = tests::fixture();
        let root = fixture.path().join("packages/a");
        let config = root.join("tsconfig.json");
        if !initial_absence {
            std::fs::write(&config, r#"{"vueCompilerOptions":{"target":2.7}}"#).unwrap();
        }
        let previous = state.document_project_state(&a).unwrap();
        let observed = vize_canon::CompilerConfigurationSnapshot::capture(
            &root,
            (!initial_absence).then_some(config.as_path()),
        )
        .unwrap();
        let raw = previous
            .project_contexts
            .loaded_config
            .read()
            .clone()
            .unwrap();
        let applied = previous
            .project_contexts
            .applied_config
            .read()
            .clone()
            .unwrap();
        // The exact old consumed observations differ from the now-valid final
        // revision, as when checked admission rejects an in-flight edit.
        std::fs::write(&config, r#"{"vueCompilerOptions":{"target":3.5}}"#).unwrap();
        assert!(!observed.observations().changed_paths().is_empty());
        let failed = super::freshness::ConfigurationObservation::failed(
            &raw,
            &applied,
            observed.observations(),
        );
        assert!(failed.needs_confirmation());
        state
            .project_contexts
            .configuration_attempts
            .lock()
            .insert(root.clone().into(), failed);
        assert!(refresh(&state, &a).errors.is_empty());
        let current = state.document_project_state(&a).unwrap();
        assert!(!Arc::ptr_eq(&previous, &current));
        assert_eq!(current.type_checker_vue_version().as_str(), "3");
        assert!(refresh(&state, &a).errors.is_empty());
    }
}

#[test]
fn watched_marker_transfers_to_checked_full_key_without_reloading_or_retiring_siblings() {
    use tower_lsp::lsp_types::{FileChangeType, FileEvent};

    for (primary, referenced) in [(true, false), (true, true), (false, false), (false, true)] {
        let fixture = crate::test_support::tempdir().unwrap();
        let workspace = fixture.path();
        let root = if primary {
            workspace.to_path_buf()
        } else {
            workspace.join("packages/a")
        };
        let nested = root.join("nested");
        let sibling_root = workspace.join("sibling");
        std::fs::create_dir_all(nested.join("src")).unwrap();
        std::fs::create_dir_all(&sibling_root).unwrap();
        let source = "<template>unsaved 😀</template>";
        let probe = Url::from_file_path(nested.join("src/Probe.vue")).unwrap();
        let control = Url::from_file_path(root.join("Control.vue")).unwrap();
        let sibling = Url::from_file_path(sibling_root.join("Sibling.vue")).unwrap();
        for uri in [&probe, &control, &sibling] {
            std::fs::write(uri.to_file_path().unwrap(), "<template>disk</template>").unwrap();
        }
        std::fs::write(workspace.join("package.json"), "{}").unwrap();
        std::fs::write(root.join("package.json"), "{}").unwrap();
        let primary_counter = root.join("evaluations.txt");
        let nested_counter = nested.join("evaluations.txt");
        std::fs::write(&primary_counter, "").unwrap();
        std::fs::write(&nested_counter, "").unwrap();
        let config_source = |hover| {
            vize_l0::cstr!(
                "import {{appendFileSync}} from 'node:fs'; export default () => {{ appendFileSync(new URL('./evaluations.txt', import.meta.url), 'x'); return {{vue:{{languageServer:{{hover:{hover}}}}}}}; }};"
            )
        };
        std::fs::write(
            root.join("vite.config.mjs"),
            config_source(false).as_bytes(),
        )
        .unwrap();
        std::fs::write(sibling_root.join("package.json"), "{}").unwrap();
        std::fs::write(
            sibling_root.join("vite.config.mjs"),
            "export default {vue:{}};",
        )
        .unwrap();
        let state = Arc::new(ServerState::new());
        state.set_workspace_root(workspace.to_path_buf());
        state.load_workspace_config(workspace);
        state.apply_initialize_workspace_folders(None, Some(workspace));
        for uri in [&probe, &control, &sibling] {
            state
                .documents
                .open(uri.clone(), source.into(), 17, "vue".into());
            let owner = state
                .document_project_state(uri)
                .unwrap_or_else(|| state.clone());
            owner.update_virtual_docs(uri, source);
        }
        let previous = state
            .document_project_state(&probe)
            .unwrap_or_else(|| state.clone());
        let previous_stamp = previous.corsa_request_stamp();
        let sibling_owner = state.document_project_state(&sibling).unwrap();
        let sibling_stamp = sibling_owner.corsa_request_stamp();
        assert!(previous.get_virtual_docs(&probe).is_some());
        assert!(previous.get_virtual_docs(&control).is_some());
        let leaf = nested.join("config/leaf.json");
        if referenced {
            std::fs::create_dir_all(leaf.parent().unwrap()).unwrap();
            std::fs::write(&leaf, r#"{"files":["../src/Probe.vue"]}"#).unwrap();
            std::fs::write(
                nested.join("tsconfig.json"),
                r#"{"files":[],"references":[{"path":"./config/leaf.json"}]}"#,
            )
            .unwrap();
        }
        let marker = nested.join("vite.config.mjs");
        std::fs::write(&marker, "export default { broken syntax").unwrap();
        let event = |typ| FileEvent {
            uri: Url::from_file_path(&marker).unwrap(),
            typ,
        };
        assert_eq!(
            state
                .project_config_event_scope(&[event(FileChangeType::CREATED)])
                .affected,
            vec![ProjectContextKey::from(nested.clone())],
            "the parent is excluded before async work is split by root",
        );
        let rejected = block_on(
            state.observe_project_config_events_async(vec![event(FileChangeType::CREATED)]),
        );
        assert_eq!(rejected.errors.len(), 1);
        assert_eq!(
            state.project_contexts.routes.get(&probe).unwrap().root,
            root
        );
        assert!(!previous.project_context_retired());
        assert!(previous.get_virtual_docs(&probe).is_some());
        assert!(
            previous_stamp.is_current(&previous),
            "a failed descendant cannot retire the parent epoch"
        );
        assert!(Arc::ptr_eq(
            &previous,
            &state
                .document_project_state(&probe)
                .unwrap_or_else(|| state.clone())
        ));
        assert_eq!(std::fs::read_to_string(&nested_counter).unwrap(), "");

        std::fs::write(&marker, config_source(true).as_bytes()).unwrap();
        let accepted = block_on(
            state.observe_project_config_events_async(vec![event(FileChangeType::CHANGED)]),
        );
        assert!(accepted.errors.is_empty());
        assert_eq!(accepted.documents, vec![(probe.clone(), 17)]);
        let expected = ProjectContextKey {
            root: nested.clone(),
            tsconfig: referenced.then(|| super::paths::physical_path(&leaf)),
        };
        assert_eq!(
            state.project_contexts.routes.get(&probe).unwrap().value(),
            &expected,
            "the watch operation itself transfers the route before another document request",
        );
        let selected = state
            .project_contexts
            .contexts
            .lock()
            .get(&expected)
            .and_then(|context| context.initialized_state())
            .unwrap();
        assert!(selected.lsp_features().hover);
        assert_eq!(selected.documents.text(&probe).as_deref(), Some(source));
        assert_eq!(selected.documents.version(&probe), Some(17));
        assert!(selected.get_virtual_docs(&probe).is_some());
        assert!(previous.get_virtual_docs(&probe).is_none());
        assert!(previous.get_virtual_docs(&control).is_some());
        assert!(!previous.project_context_retired());
        assert!(Arc::ptr_eq(
            &selected,
            &state.document_project_state(&probe).unwrap()
        ));
        assert!(Arc::ptr_eq(
            &previous,
            &state
                .document_project_state(&control)
                .unwrap_or_else(|| state.clone())
        ));
        assert!(Arc::ptr_eq(
            &sibling_owner,
            &state.document_project_state(&sibling).unwrap()
        ));
        assert!(sibling_stamp.is_current(&sibling_owner));
        assert_eq!(std::fs::read_to_string(&primary_counter).unwrap(), "x");
        assert_eq!(std::fs::read_to_string(&nested_counter).unwrap(), "x");
        for uri in [&probe, &control, &sibling] {
            assert_eq!(
                std::fs::read_to_string(uri.to_file_path().unwrap()).unwrap(),
                "<template>disk</template>"
            );
            assert_eq!(state.documents.text(uri).as_deref(), Some(source));
            assert_eq!(state.documents.version(uri), Some(17));
        }
    }
}
