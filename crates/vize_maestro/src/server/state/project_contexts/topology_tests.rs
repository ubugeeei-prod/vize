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
