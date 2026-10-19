//! Dependency repairs, canceled workers and bounded reload retries.
use super::*;

#[test]
fn newly_introduced_missing_or_malformed_inherited_bases_recover_without_config_edit() {
    for shared in [false, true] {
        for malformed in [false, true] {
            let (fixture, state, a, b) = tests::fixture();
            let root = fixture.path().join("packages/a");
            let base = root.join("base.json");
            let (specifier, target) = if shared {
                (
                    "../../shared-options.json",
                    fixture.path().join("shared-options.json"),
                )
            } else {
                ("./next-base.json", root.join("next-base.json"))
            };
            std::fs::write(root.join("tsconfig.json"), r#"{"extends":"./base.json"}"#).unwrap();
            std::fs::write(&base, r#"{"vueCompilerOptions":{"target":2.7}}"#).unwrap();
            let before = state.document_project_state(&a).unwrap();
            let sibling = state.document_project_state(&b).unwrap();
            assert_eq!(
                before.type_checker_vue_version(),
                vize_l0::config::VueVersion::V2_7
            );
            if malformed {
                std::fs::write(&target, "{").unwrap();
            }
            std::fs::write(
                &base,
                serde_json::to_vec(&serde_json::json!({"extends":specifier})).unwrap(),
            )
            .unwrap();
            let rejected = state
                .observe_project_config_events_checked(&[event(&base, FileChangeType::CHANGED)]);
            assert_error(&rejected, &root, &base);
            assert!(Arc::ptr_eq(
                &before,
                &state.document_project_state(&a).unwrap()
            ));
            std::fs::write(&target, r#"{"vueCompilerOptions":{"target":3.5}}"#).unwrap();
            let change = event(
                &target,
                if malformed {
                    FileChangeType::CHANGED
                } else {
                    FileChangeType::CREATED
                },
            );
            let repaired =
                state.observe_project_config_events_checked(std::slice::from_ref(&change));
            assert!(repaired.errors.is_empty());
            let current = state.document_project_state(&a).unwrap();
            assert!(!Arc::ptr_eq(&before, &current));
            assert_eq!(
                current.type_checker_vue_version(),
                vize_l0::config::VueVersion::V3
            );
            assert!(Arc::ptr_eq(
                &sibling,
                &state.document_project_state(&b).unwrap()
            ));
            assert!(
                repaired
                    .accepted_file_events(&sibling, &[change])
                    .is_empty()
            );
            assert!(
                state
                    .project_contexts
                    .pending_config_reloads
                    .lock()
                    .is_empty()
            );
        }
    }
}

#[test]
fn canceled_awaiter_keeps_root_admission_owned_by_the_running_worker() {
    use futures::future::{AbortHandle, Abortable};
    use std::{
        sync::mpsc,
        time::{Duration, Instant},
    };
    let (_fixture, state, a, _) = tests::fixture();
    let root = state
        .document_context_root(&a.to_file_path().unwrap())
        .unwrap();
    let gates = state.config_reload_gates(&[root]);
    let gate = gates.first().unwrap().clone();
    let admission = futures::executor::block_on(gate.clone().lock_owned());
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let (canceled_tx, canceled_rx) = mpsc::channel();
    let (cancel, registration) = AbortHandle::new_pair();
    let deadline = Instant::now() + Duration::from_secs(10);
    let awaiter = std::thread::spawn(move || {
        // A full shared queue must wake and re-poll this real executor before
        // cancellation. One NOOP poll cannot establish worker admission.
        let result = futures::executor::block_on(Abortable::new(
            ServerState::project_background(move || {
                let _admission = admission;
                entered_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(10)).unwrap();
                done_tx.send(()).unwrap();
            }),
            registration,
        ));
        canceled_tx.send(result).unwrap();
    });
    entered_rx
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .unwrap();
    cancel.abort();
    let canceled = canceled_rx
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .unwrap();
    assert!(
        canceled.is_err(),
        "the admitted job's awaiter must be canceled"
    );
    awaiter.join().unwrap();
    assert!(
        gate.try_lock().is_none(),
        "canceling the notification must not admit a concurrent owner reload"
    );
    release_tx.send(()).unwrap();
    done_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let _released = futures::executor::block_on(gate.lock());
}

#[test]
fn idle_new_package_markers_do_not_import_or_initialize_that_package() {
    let (fixture, state, a, b) = tests::fixture();
    let current = state.document_project_state(&a).unwrap();
    let idle = fixture.path().join("packages/idle");
    std::fs::create_dir_all(&idle).unwrap();
    std::fs::write(idle.join("package.json"), "{}").unwrap();
    std::fs::write(
        idle.join("vite.config.mjs"),
        "throw new Error('idle configuration must not execute');",
    )
    .unwrap();
    let before = state.stable_project_routing_generation();
    let result = state.observe_project_config_events_checked(&[event(
        &idle.join("package.json"),
        FileChangeType::CREATED,
    )]);
    assert!(result.documents.is_empty());
    assert!(result.errors.is_empty());
    assert_eq!(state.stable_project_routing_generation(), before);
    assert_eq!(state.cached_project_states().len(), 1);
    assert!(Arc::ptr_eq(
        &current,
        &state.document_project_state(&a).unwrap()
    ));
    assert!(
        !state
            .project_contexts
            .contexts
            .lock()
            .contains_key(&ProjectContextKey::from(idle.clone()))
    );
    assert!(
        !state
            .project_contexts
            .contexts
            .lock()
            .contains_key(&ProjectContextKey::from(fixture.path().join("packages/b")))
    );
    assert_eq!(state.documents.version(&b), None);
}

#[test]
fn unavailable_worker_retains_known_configuration_events_and_pending_retry() {
    let (fixture, state, a, b) = tests::fixture();
    let root = fixture.path().join("packages/a");
    let helper = root.join("settings.mjs");
    std::fs::write(
        &helper,
        "export default { languageServer: { hover: false } };",
    )
    .unwrap();
    std::fs::write(
        root.join("vite.config.mjs"),
        "import vue from './settings.mjs'; export default { vue };",
    )
    .unwrap();
    let owner = state.document_project_state(&a).unwrap();
    let sibling = state.document_project_state(&b).unwrap();
    let before = settings(&owner);
    let stamp = owner.corsa_request_stamp();
    std::fs::remove_file(&helper).unwrap();
    let changed = event(&helper, FileChangeType::DELETED);
    let ordinary = event(&root.join("src/ordinary.ts"), FileChangeType::CHANGED);
    let failure = state.config_worker_unavailable(&root, &[changed.clone(), ordinary.clone()]);
    assert_eq!(
        serde_json::to_value(&failure.errors).unwrap(),
        serde_json::json!([{
            "type":1,
            "message":"Vize could not reload project configuration because the configuration worker is unavailable. Keeping the last good configuration."
        }])
    );
    assert_eq!(
        failure.accepted_file_events(&owner, &[changed.clone(), ordinary.clone()]),
        vec![ordinary]
    );
    assert!(
        failure
            .accepted_file_events(&sibling, &[changed])
            .is_empty()
    );
    assert!(Arc::ptr_eq(
        &owner,
        &state.document_project_state(&a).unwrap()
    ));
    assert_eq!(settings(&owner), before);
    assert!(stamp.is_current(&owner));
    assert!(
        state
            .project_contexts
            .pending_config_reloads
            .lock()
            .contains(&ProjectContextKey::from(root.clone()))
    );
    std::fs::write(
        &helper,
        "export default { languageServer: { hover: true } };",
    )
    .unwrap();
    let repaired =
        state.observe_project_config_events_checked(&[event(&helper, FileChangeType::CREATED)]);
    assert!(repaired.errors.is_empty());
    assert!(
        state
            .document_project_state(&a)
            .unwrap()
            .lsp_features()
            .hover
    );
    assert!(
        state
            .project_contexts
            .pending_config_reloads
            .lock()
            .is_empty()
    );
}

#[test]
fn failed_incidental_retry_keeps_shared_tsconfig_included_global_source_events_ordinary() {
    let (fixture, state, a, b) = tests::fixture();
    let root = fixture.path().join("packages/a");
    let helper = root.join("settings.mjs");
    let shared = fixture.path().join("shared-global.ts");
    std::fs::write(&shared, "declare const sharedPolicy: string;").unwrap();
    std::fs::write(
        fixture.path().join("packages/b/tsconfig.json"),
        r#"{"files":["../../shared-global.ts"]}"#,
    )
    .unwrap();
    std::fs::write(
        &helper,
        "export default { languageServer: { hover: false } };",
    )
    .unwrap();
    std::fs::write(
        root.join("vite.config.mjs"),
        "import vue from './settings.mjs'; export default { vue };",
    )
    .unwrap();
    let owner = state.document_project_state(&a).unwrap();
    let sibling = state.document_project_state(&b).unwrap();
    state.documents.open(
        b.clone(),
        "<script setup lang=\"ts\">const value: string = sharedPolicy;</script>".into(),
        1,
        "vue".into(),
    );
    std::fs::remove_file(&helper).unwrap();
    let initial =
        state.observe_project_config_events_checked(&[event(&helper, FileChangeType::DELETED)]);
    assert_error(&initial, &root, &helper);
    std::fs::write(&shared, "declare const sharedPolicy: number;").unwrap();
    let change = event(&shared, FileChangeType::CHANGED);
    assert!(
        crate::server::importers::open_typecheck_dependents(&sibling, &change.uri).is_empty(),
        "the governing global has no ES import edge"
    );
    let retry = state.observe_project_config_events_checked(std::slice::from_ref(&change));
    assert!(retry.errors.is_empty());
    assert_eq!(
        retry.accepted_file_events(&owner, std::slice::from_ref(&change)),
        vec![change.clone()]
    );
    assert_eq!(
        retry.accepted_file_events(&sibling, std::slice::from_ref(&change)),
        vec![change]
    );
    assert!(Arc::ptr_eq(
        &owner,
        &state.document_project_state(&a).unwrap()
    ));
    assert!(Arc::ptr_eq(
        &sibling,
        &state.document_project_state(&b).unwrap()
    ));
    assert_eq!(
        std::fs::read_to_string(fixture.path().join("packages/b/tsconfig.json")).unwrap(),
        r#"{"files":["../../shared-global.ts"]}"#
    );
}

#[test]
fn queued_or_unavailable_worker_after_workspace_removal_does_not_retain_retry_metadata() {
    let fixture = tempfile::tempdir().unwrap();
    let primary = fixture.path().join("primary");
    let secondary = fixture.path().join("secondary");
    std::fs::create_dir_all(&primary).unwrap();
    std::fs::create_dir_all(&secondary).unwrap();
    let state = Arc::new(ServerState::new());
    state.set_workspace_root(primary.clone());
    state.load_workspace_config(&primary);
    state.set_workspace_folders(vec![primary.clone(), secondary.clone()]);
    state.documents.open(
        Url::from_file_path(secondary.join("App.vue")).unwrap(),
        "<template />".into(),
        1,
        "vue".into(),
    );
    let config = secondary.join("vite.config.mjs");
    std::fs::write(&config, "export default {};").unwrap();
    let changes = vec![event(&config, FileChangeType::CREATED)];
    let queued_scope = state.project_config_event_scope(&changes);
    assert_eq!(
        queued_scope.affected,
        vec![ProjectContextKey::from(secondary.clone())]
    );
    state.set_workspace_folders(vec![primary]);
    state.refresh_project_routes();
    let unavailable = state.config_worker_unavailable(&secondary, &changes);
    assert!(unavailable.documents.is_empty());
    assert!(unavailable.errors.is_empty());
    assert!(
        state
            .project_contexts
            .pending_config_reloads
            .lock()
            .is_empty()
    );
    // This is the exact operation owned by an already queued worker, after
    // the workspace-removal handler has completed its refresh.
    let result = state.apply_project_config_events(&changes, queued_scope);
    assert!(result.documents.is_empty());
    assert!(result.errors.is_empty());
    assert!(state.project_contexts.contexts.lock().is_empty());
    assert!(
        state
            .project_contexts
            .pending_config_reloads
            .lock()
            .is_empty()
    );
}

#[test]
fn first_failed_admission_observations_route_external_base_creation_to_primary_and_nested_owners() {
    for primary in [false, true] {
        let (fixture, state, nested, sibling) = tests::fixture();
        let external = tempfile::tempdir().unwrap();
        let base = external.path().join("base.json");
        let (root, uri) = if primary {
            (
                fixture.path().to_path_buf(),
                Url::from_file_path(fixture.path().join("Main.vue")).unwrap(),
            )
        } else {
            (fixture.path().join("packages/a"), nested)
        };
        assert!(
            !is_config_marker(&base),
            "repair must use observations, not marker discovery"
        );
        std::fs::write(
            root.join("tsconfig.json"),
            serde_json::to_vec(&serde_json::json!({"extends":base})).unwrap(),
        )
        .unwrap();
        std::fs::write(uri.to_file_path().unwrap(), "<template>disk</template>").unwrap();
        state.documents.open(
            uri.clone(),
            "<template>unsaved</template>".into(),
            5,
            "vue".into(),
        );
        let sibling_owner = state.document_project_state(&sibling).unwrap();
        let sibling_stamp = sibling_owner.corsa_request_stamp();
        let initial = if primary {
            state.load_workspace_config(&root);
            state.clone()
        } else {
            state.document_project_state(&uri).unwrap()
        };
        assert!(
            initial
                .project_contexts
                .config_dependencies
                .read()
                .is_empty()
        );
        assert!(
            initial
                .project_contexts
                .compiler_configuration
                .read()
                .is_none()
        );
        assert!(
            state
                .configuration_watch_paths()
                .contains(&super::paths::physical_path(&base))
        );
        std::fs::write(&base, r#"{"vueCompilerOptions":{"target":2.7}}"#).unwrap();
        let change = event(&base, FileChangeType::CREATED);
        let scope = state.project_config_event_scope(std::slice::from_ref(&change));
        assert_eq!(scope.affected, vec![ProjectContextKey::from(root)]);
        let repaired = state.observe_project_config_events_checked(&[change]);
        assert!(repaired.errors.is_empty());
        assert_eq!(repaired.documents, vec![(uri.clone(), 5)]);
        let current = state.document_project_state(&uri).unwrap();
        assert_eq!(current.type_checker_vue_version().as_str(), "2.7");
        assert!(
            current
                .project_contexts
                .compiler_configuration
                .read()
                .is_some()
        );
        assert_eq!(
            current.documents.text(&uri).as_deref(),
            Some("<template>unsaved</template>")
        );
        assert_eq!(current.documents.version(&uri), Some(5));
        assert_eq!(
            std::fs::read_to_string(uri.to_file_path().unwrap()).unwrap(),
            "<template>disk</template>"
        );
        assert!(Arc::ptr_eq(
            &sibling_owner,
            &state.document_project_state(&sibling).unwrap()
        ));
        assert!(sibling_stamp.is_current(&sibling_owner));
    }
}
