//! Root-local reload admission and concurrent lifecycle transitions.
use super::*;

#[test]
fn serialized_config_reload_rechecks_latest_disk_after_an_older_preparation() {
    let (root, state, uri, _) = fixture();
    let config = root.path().join("vite.config.mjs");
    let marker = Url::from_file_path(&config).unwrap();
    std::fs::write(
        &config,
        "export default { vue: { languageServer: { hover: false } } };",
    )
    .unwrap();
    let (entered, resume, install) = pause_at(Phase::ConfigInstall);
    let first_state = state.clone();
    let first_marker = marker.clone();
    let first = std::thread::spawn(move || {
        install();
        first_state.observe_project_config_events(&[FileEvent {
            uri: first_marker,
            typ: FileChangeType::CHANGED,
        }])
    });
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    let gates = state.config_reload_gates(&[root.path().to_path_buf()]);
    assert!(
        gates.first().unwrap().try_lock().is_none(),
        "prepared reload retains root admission through installation"
    );
    std::fs::write(
        &config,
        "export default { vue: { languageServer: { hover: true } } };",
    )
    .unwrap();
    let second_state = state.clone();
    let second = std::thread::spawn(move || {
        second_state.observe_project_config_events(&[FileEvent {
            uri: marker,
            typ: FileChangeType::CHANGED,
        }])
    });
    resume.send(()).unwrap();
    first.join().unwrap();
    second.join().unwrap();
    assert!(
        state
            .document_project_state(&uri)
            .unwrap()
            .lsp_features()
            .hover
    );
}

#[test]
fn shutdown_between_preparation_and_installation_never_recreates_primary_owner() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("vite.config.mjs");
    std::fs::write(&config, "export default {};").unwrap();
    let state = Arc::new(ServerState::new());
    state.set_workspace_root(root.path().to_path_buf());
    state.load_workspace_config(root.path());
    assert!(state.project_contexts.contexts.lock().is_empty());
    let (entered, resume, install) = pause_at(Phase::ConfigInstall);
    let worker_state = state.clone();
    let worker = std::thread::spawn(move || {
        install();
        worker_state.observe_project_config_events(&[FileEvent {
            uri: Url::from_file_path(config).unwrap(),
            typ: FileChangeType::CHANGED,
        }])
    });
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    state.retire_project_contexts();
    resume.send(()).unwrap();
    assert!(worker.join().unwrap().is_empty());
    assert!(state.project_contexts.contexts.lock().is_empty());
    assert!(state.cached_project_states().is_empty());
    assert!(
        state
            .project_contexts
            .pending_config_reloads
            .lock()
            .is_empty()
    );
    assert!(Arc::ptr_eq(&state, &state.current_primary_project_state()));
}

#[test]
fn same_root_async_waiters_do_not_consume_workers_needed_by_an_unrelated_sibling() {
    use futures::Future;
    use std::task::{Context, Poll};
    let (fixture, state, a, b) = super::super::tests::fixture();
    state.document_project_state(&a).unwrap();
    let old_b = state.document_project_state(&b).unwrap();
    let root_a = fixture.path().join("packages/a");
    let gates = state.config_reload_gates(std::slice::from_ref(&root_a));
    let held = futures::executor::block_on(gates.first().unwrap().lock());
    let changes_a = vec![FileEvent {
        uri: Url::from_file_path(root_a.join("vite.config.mjs")).unwrap(),
        typ: FileChangeType::CHANGED,
    }];
    let mut first = std::pin::pin!(state.observe_project_config_events_async(changes_a.clone()));
    let mut second = std::pin::pin!(state.observe_project_config_events_async(changes_a));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(matches!(first.as_mut().poll(&mut context), Poll::Pending));
    assert!(matches!(second.as_mut().poll(&mut context), Poll::Pending));
    let changes_b = vec![FileEvent {
        uri: Url::from_file_path(fixture.path().join("packages/b/vite.config.mjs")).unwrap(),
        typ: FileChangeType::CHANGED,
    }];
    let (finished_tx, finished_rx) = mpsc::channel();
    let worker_state = state.clone();
    let worker = std::thread::spawn(move || {
        finished_tx
            .send(
                futures::executor::block_on(
                    worker_state.observe_project_config_events_async(changes_b),
                )
                .errors
                .len(),
            )
            .unwrap();
    });
    let finished = finished_rx.recv_timeout(Duration::from_secs(10));
    drop(held);
    assert_eq!(
        finished.unwrap(),
        0,
        "a sibling evaluates while both same-root waiters remain pending"
    );
    worker.join().unwrap();
    assert!(!Arc::ptr_eq(
        &old_b,
        &state.document_project_state(&b).unwrap()
    ));
}

#[test]
fn dependency_event_during_preparation_queues_against_the_new_graph() {
    use futures::Future;
    use std::task::{Context, Poll};
    let (root, state, uri, _) = fixture();
    let helper = root.path().join("settings.mjs");
    let next = root.path().join("next-settings.mjs");
    std::fs::write(
        root.path().join("vite.config.mjs"),
        "import vue from './settings.mjs'; export default { vue };",
    )
    .unwrap();
    std::fs::write(
        &helper,
        "export default { languageServer: { hover: false } };",
    )
    .unwrap();
    let original = state.document_project_state(&uri).unwrap();
    assert!(!original.lsp_features().hover);
    std::fs::write(
        &next,
        "export default { languageServer: { hover: false } };",
    )
    .unwrap();
    std::fs::write(&helper, "export { default } from './next-settings.mjs';").unwrap();
    let (entered, resume, install) = pause_at(Phase::ConfigInstall);
    let first_state = state.clone();
    let first = std::thread::spawn(move || {
        install();
        first_state.observe_project_config_events(&[FileEvent {
            uri: Url::from_file_path(helper).unwrap(),
            typ: FileChangeType::CHANGED,
        }])
    });
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    std::fs::write(&next, "export default { languageServer: { hover: true } };").unwrap();
    let mut newer = Box::pin(state.observe_project_config_events_async(vec![FileEvent {
        uri: Url::from_file_path(next).unwrap(),
        typ: FileChangeType::CHANGED,
    }]));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(
        matches!(newer.as_mut().poll(&mut context), Poll::Pending),
        "the new dependency event must await in-flight root admission rather than return empty against the old graph"
    );
    resume.send(()).unwrap();
    first.join().unwrap();
    assert!(futures::executor::block_on(newer).errors.is_empty());
    let current = state.document_project_state(&uri).unwrap();
    assert!(current.lsp_features().hover);
    assert!(!Arc::ptr_eq(&original, &current));
    assert!(
        state
            .project_contexts
            .pending_config_reloads
            .lock()
            .is_empty()
    );
}

#[test]
fn removed_secondary_workspace_cannot_install_a_prepared_new_marker_owner() {
    let fixture = tempfile::tempdir().unwrap();
    let primary = fixture.path().join("primary");
    let secondary = fixture.path().join("secondary");
    std::fs::create_dir_all(&primary).unwrap();
    std::fs::create_dir_all(&secondary).unwrap();
    let state = Arc::new(ServerState::new());
    state.set_workspace_root(primary.clone());
    state.load_workspace_config(&primary);
    state.set_workspace_folders(vec![primary.clone(), secondary.clone()]);
    let uri = Url::from_file_path(secondary.join("App.vue")).unwrap();
    state.documents.open(
        uri,
        "<template>open buffer</template>".into(),
        1,
        "vue".into(),
    );
    assert!(state.project_contexts.contexts.lock().is_empty());
    let config = secondary.join("vite.config.mjs");
    std::fs::write(
        &config,
        "export default { vue: { languageServer: { hover: false } } };",
    )
    .unwrap();
    let (entered, resume, install) = pause_at(Phase::ConfigInstall);
    let worker_state = state.clone();
    let worker = std::thread::spawn(move || {
        install();
        worker_state.observe_project_config_events(&[FileEvent {
            uri: Url::from_file_path(config).unwrap(),
            typ: FileChangeType::CREATED,
        }])
    });
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(
        state
            .project_contexts
            .pending_config_reloads
            .lock()
            .contains(&super::super::ProjectContextKey::from(secondary.clone()))
    );
    state.set_workspace_folders(vec![primary.clone()]);
    state.refresh_project_routes();
    assert!(
        !state
            .project_contexts
            .pending_config_reloads
            .lock()
            .contains(&super::super::ProjectContextKey::from(secondary.clone()))
    );
    resume.send(()).unwrap();
    assert!(worker.join().unwrap().is_empty());
    assert!(state.project_contexts.contexts.lock().is_empty());
    assert!(state.cached_project_states().is_empty());
    assert_eq!(state.workspace_root_paths(), vec![primary]);
}

#[test]
fn cached_document_dispatch_awaits_only_its_own_active_configuration_reload() {
    use futures::Future;
    use std::task::{Context, Poll};
    let (fixture, state, a, b) = super::super::tests::fixture();
    let previous = state.document_project_state(&a).unwrap();
    let sibling = state.document_project_state(&b).unwrap();
    let config = fixture.path().join("packages/a/vite.config.mjs");
    std::fs::write(
        &config,
        "export default { vue: { languageServer: { hover: true } } };",
    )
    .unwrap();
    let (entered, resume, install) = pause_at(Phase::ConfigInstall);
    let worker_state = state.clone();
    let worker = std::thread::spawn(move || {
        install();
        worker_state.observe_project_config_events(&[FileEvent {
            uri: Url::from_file_path(config).unwrap(),
            typ: FileChangeType::CHANGED,
        }]);
    });
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    let mut request = Box::pin(state.document_project_state_async(&a));
    let mut control = Box::pin(state.document_project_state_async(&b));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(matches!(request.as_mut().poll(&mut context), Poll::Pending));
    let Poll::Ready(Some(unaffected)) = control.as_mut().poll(&mut context) else {
        panic!("a cached unrelated sibling must not wait for another root's reload");
    };
    assert!(Arc::ptr_eq(&sibling, &unaffected));
    resume.send(()).unwrap();
    worker.join().unwrap();
    let current = futures::executor::block_on(request).unwrap();
    assert!(!Arc::ptr_eq(&previous, &current));
    assert!(current.lsp_features().hover);
}
