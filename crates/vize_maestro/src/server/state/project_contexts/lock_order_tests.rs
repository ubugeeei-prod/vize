//! Exercise real routing boundaries while the opposite lock is held.
use super::{Arc, ServerState, Url, registry};
use std::{cell::RefCell, sync::mpsc, time::Duration};
use tower_lsp::lsp_types::{FileChangeType, FileEvent};

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Contexts,
    Routes,
    ConfigInstall,
}

type Hook = (Phase, Box<dyn FnOnce()>);

thread_local! {
    static HOOK: RefCell<Option<Hook>> = const { RefCell::new(None) };
}

fn arrive(phase: Phase) {
    let hook = HOOK.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot
            .as_ref()
            .is_some_and(|(expected, _)| *expected == phase)
        {
            slot.take().map(|(_, hook)| hook)
        } else {
            None
        }
    });
    if let Some(hook) = hook {
        hook();
    }
}

pub(super) fn before_contexts_lock() {
    arrive(Phase::Contexts);
}

pub(super) fn before_route_changes() {
    arrive(Phase::Routes);
}

pub(super) fn before_config_install() {
    arrive(Phase::ConfigInstall);
}

pub(super) fn at_config_install(action: impl FnOnce() + 'static) {
    HOOK.with(|slot| *slot.borrow_mut() = Some((Phase::ConfigInstall, Box::new(action))));
}

fn pause_at(phase: Phase) -> (mpsc::Receiver<()>, mpsc::Sender<()>, impl FnOnce()) {
    let (entered_tx, entered_rx) = mpsc::channel();
    let (resume_tx, resume_rx) = mpsc::channel();
    let install = move || {
        HOOK.with(|slot| {
            *slot.borrow_mut() = Some((
                phase,
                Box::new(move || {
                    entered_tx.send(()).unwrap();
                    resume_rx.recv_timeout(Duration::from_secs(10)).unwrap();
                }),
            ));
        });
    };
    (entered_rx, resume_tx, install)
}

fn fixture() -> (
    tempfile::TempDir,
    Arc<ServerState>,
    Url,
    Arc<registry::ProjectContext>,
) {
    let root = tempfile::tempdir().unwrap();
    let state = Arc::new(ServerState::new());
    state.set_workspace_root(root.path().to_path_buf());
    let uri = Url::from_file_path(root.path().join("App.vue")).unwrap();
    let context = Arc::new(registry::ProjectContext::new(root.path().to_path_buf()));
    state
        .project_contexts
        .contexts
        .lock()
        .insert(root.path().to_path_buf().into(), context.clone());
    state
        .project_contexts
        .routes
        .insert(uri.clone(), root.path().to_path_buf().into());
    (root, state, uri, context)
}

fn marker_corpus() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../../../../tests/_fixtures/differential/config/project-marker-retirement.json"
    ))
    .unwrap()
}

#[test]
fn cached_route_releases_its_shard_before_waiting_for_contexts() {
    let (_root, state, uri, expected) = fixture();
    let contexts = state.project_contexts.contexts.lock();
    let (entered, resume, install) = pause_at(Phase::Contexts);
    let worker_state = state.clone();
    let worker_uri = uri.clone();
    let worker = std::thread::spawn(move || {
        install();
        worker_state.document_project_context(&worker_uri).unwrap()
    });
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    let shard_released = matches!(
        state.project_contexts.routes.try_get_mut(&uri),
        dashmap::try_result::TryResult::Present(_)
    );
    drop(contexts);
    resume.send(()).unwrap();
    let actual = worker.join().unwrap();
    assert!(
        shard_released,
        "routing must not wait for contexts with a shard guard"
    );
    assert!(Arc::ptr_eq(&actual, &expected));
}

#[test]
fn config_watcher_releases_contexts_before_waiting_for_a_route_shard() {
    let (root, state, uri, previous) = fixture();
    let route = state.project_contexts.routes.get(&uri).unwrap();
    let generation = state.project_routing_generation();
    let marker = Url::from_file_path(root.path().join("vite.config.mjs")).unwrap();
    let (entered, resume, install) = pause_at(Phase::Routes);
    let worker_state = state.clone();
    let worker = std::thread::spawn(move || {
        install();
        worker_state.observe_project_config_events(&[FileEvent {
            uri: marker,
            typ: FileChangeType::CHANGED,
        }])
    });
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    let contexts_released = state.project_contexts.contexts.try_lock().is_some();
    assert!(state.stable_project_routing_generation().is_none());
    drop(route);
    resume.send(()).unwrap();
    assert!(worker.join().unwrap().is_empty());
    assert!(
        contexts_released,
        "watcher must not wait for routes with contexts held"
    );
    assert!(previous.initialize(&state).is_none());
    assert!(state.project_contexts.routes.is_empty());
    assert!(state.stable_project_routing_generation().unwrap() > generation);
}

#[test]
fn generated_and_dependency_markers_do_not_retire_ordinary_nested_owners() {
    let corpus = marker_corpus();
    let (root, state, uri, previous) = fixture();
    let version = corpus["expectedRetainedVersion"].as_i64().unwrap() as i32;
    state.documents.open(
        uri.clone(),
        corpus["openSource"].as_str().unwrap().into(),
        version,
        "vue".into(),
    );
    let generation = state.project_routing_generation();
    let events = corpus["excludedDirectories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|directory| FileEvent {
            uri: Url::from_file_path(
                root.path()
                    .join(directory.as_str().unwrap())
                    .join("nested/package.json"),
            )
            .unwrap(),
            typ: FileChangeType::CHANGED,
        })
        .collect::<Vec<_>>();
    assert!(state.observe_project_config_events(&events).is_empty());
    assert_eq!(state.project_routing_generation(), generation);
    assert!(Arc::ptr_eq(
        &state.document_project_context(&uri).unwrap(),
        &previous
    ));
    let nested = root
        .path()
        .join(corpus["ordinaryNestedMarker"].as_str().unwrap());
    std::fs::create_dir_all(nested.parent().unwrap()).unwrap();
    std::fs::write(&nested, "export default {};").unwrap();
    assert_eq!(
        state.observe_project_config_events(&[FileEvent {
            uri: Url::from_file_path(nested).unwrap(),
            typ: FileChangeType::CREATED,
        }]),
        vec![(uri, version)]
    );
    assert!(previous.initialize(&state).is_none());
    assert!(state.project_routing_generation() > generation);
}

#[cfg(unix)]
#[test]
fn authored_and_physical_dependency_markers_both_refuse_retirement() {
    let (root, state, uri, previous) = fixture();
    let dependency = root.path().join("node_modules/dependency");
    let authored = root.path().join("linked-dependency");
    std::fs::create_dir_all(&dependency).unwrap();
    std::fs::write(dependency.join("package.json"), "{}").unwrap();
    std::os::unix::fs::symlink(&dependency, &authored).unwrap();
    let physical = root.path().join("ordinary-source");
    std::fs::create_dir_all(&physical).unwrap();
    std::fs::write(physical.join("package.json"), "{}").unwrap();
    let ignored = root.path().join("node_modules/linked-source");
    std::os::unix::fs::symlink(&physical, &ignored).unwrap();
    let generation = state.project_routing_generation();
    for marker in [authored.join("package.json"), ignored.join("package.json")] {
        assert!(
            state
                .observe_project_config_events(&[FileEvent {
                    uri: Url::from_file_path(marker).unwrap(),
                    typ: FileChangeType::CHANGED,
                }])
                .is_empty()
        );
    }
    assert_eq!(state.project_routing_generation(), generation);
    assert!(Arc::ptr_eq(
        &state.document_project_context(&uri).unwrap(),
        &previous
    ));
}

#[test]
fn registered_workspaces_named_after_generated_directories_observe_their_own_configuration() {
    let corpus = marker_corpus();
    for name in corpus["acceptedWorkspaceNames"].as_array().unwrap() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join(name.as_str().unwrap());
        std::fs::create_dir_all(&root).unwrap();
        let marker = root.join("vite.config.mjs");
        std::fs::write(&marker, "export default { vue: {} };").unwrap();
        let state = Arc::new(ServerState::new());
        state.set_workspace_root(root.clone());
        let uri = Url::from_file_path(root.join("App.vue")).unwrap();
        state.documents.open(
            uri.clone(),
            "<template>unsaved</template>".into(),
            9,
            "vue".into(),
        );
        let generation = state.project_routing_generation();
        assert_eq!(
            state.observe_project_config_events(&[FileEvent {
                uri: Url::from_file_path(marker).unwrap(),
                typ: FileChangeType::CHANGED,
            }]),
            vec![(uri, 9)]
        );
        assert!(state.project_context_retired());
        assert!(state.project_routing_generation() > generation);
    }
}

#[path = "config_reload_admission_tests.rs"]
mod config_reload_admission_tests;
