//! Explicit owner retirement refuses issued/ready work and drains every family.
use super::{
    Arc, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject, block_on,
};
use crate::{server::ServerState, source_project::SnapshotRefusal};
use std::{
    future::Future,
    sync::atomic::{AtomicUsize, Ordering},
    task::{Context, Poll},
};
use tower_lsp::lsp_types::{LinkedEditingRanges, Location, Range, Url};

struct RetirementWake {
    project: Arc<NativeNavigationProject<'static>>,
    wakes: AtomicUsize,
}

impl futures::task::ArcWake for RetirementWake {
    fn wake_by_ref(wake: &Arc<Self>) {
        for cache in [
            &wake.project.workers,
            &wake.project.selected_workers,
            &wake.project.names_workers,
        ] {
            assert!(
                cache.try_lock().is_some(),
                "retirement must wake outside worker-map locks"
            );
        }
        wake.wakes.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(feature = "native")]
#[test]
fn primary_marker_retirement_refuses_ready_source_and_module_publication_before_drain() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("vite.config.mjs");
    std::fs::write(&config, "export default { vue: {} };").unwrap();
    let state = Arc::new(ServerState::new());
    state.set_workspace_root(root.path().to_path_buf());
    state.load_workspace_config(root.path());
    let uri = Url::from_file_path(root.path().join("local.ts")).unwrap();
    state.documents.open(
        uri.clone(),
        "const value=1;value;".into(),
        9,
        "typescript".into(),
    );
    let project = NativeNavigationProject::new(SourceQueryProject::new_server(state.clone()));
    let (source, source_cancel) = project.source.begin_query(&uri).unwrap();
    let ready =
        block_on(source.run(|snapshot| async move { snapshot.source().to_owned() })).unwrap();
    let (module, module_cancel) = project.source.begin_query(&uri).unwrap();
    let module_ready =
        block_on(module.run(|snapshot| async move { snapshot.source().to_owned() })).unwrap();
    let context = module_ready.capture_current_module_link_context().unwrap();
    state.observe_project_config_events(&[tower_lsp::lsp_types::FileEvent {
        uri: Url::from_file_path(config).unwrap(),
        typ: tower_lsp::lsp_types::FileChangeType::CHANGED,
    }]);
    assert!(!source_cancel.is_aborted());
    assert!(!module_cancel.is_aborted());
    let mut published = false;
    assert_eq!(
        ready.publish(|_| published = true),
        Err(SnapshotRefusal::Superseded)
    );
    assert_eq!(
        module_ready.capture_current_module_link_context().err(),
        Some(
            crate::source_project::project::ModuleLinkPublicationError::Source(
                SnapshotRefusal::Superseded
            )
        )
    );
    assert_eq!(
        module_ready.publish_with_module_link_context(&context, |_| published = true),
        Err(
            crate::source_project::project::ModuleLinkPublicationError::Source(
                SnapshotRefusal::Superseded
            )
        )
    );
    assert!(!published);
    assert_eq!(state.documents.version(&uri), Some(9));
    project.retire();
}

#[test]
fn retired_projects_refuse_issued_and_ready_work_without_mutating_document_versions() {
    let state = Arc::new(ServerState::new());
    let uri = Url::parse("file:///retired.ts").unwrap();
    state.documents.open(
        uri.clone(),
        "const value=1;value;".into(),
        7,
        "typescript".into(),
    );
    let project = NativeNavigationProject::new(SourceQueryProject::new_server(state.clone()));
    let (issued, cancel) = project.source.begin_query(&uri).unwrap();
    let snapshot = issued.snapshot().clone();
    let (prepared, _) = project.source.begin_query(&uri).unwrap();
    let ready =
        block_on(prepared.run(|snapshot| async move { snapshot.source().to_owned() })).unwrap();
    project.mark_retired();
    assert!(!cancel.is_aborted());
    let mut published = false;
    assert_eq!(
        ready.publish(|_| published = true),
        Err(SnapshotRefusal::Superseded)
    );
    assert!(!published);
    assert!(matches!(
        project.source.begin_query(&uri),
        Err(SnapshotRefusal::Superseded)
    ));
    assert!(matches!(
        project.worker(snapshot),
        Err(NavigationRefusal::Host(SnapshotRefusal::Superseded))
    ));
    project.retire();
    assert!(cancel.is_aborted());
    assert!(matches!(
        block_on(issued.run(|_| async {})),
        Err(SnapshotRefusal::Cancelled)
    ));
    assert_eq!(state.documents.version(&uri), Some(7));
}

#[test]
fn retirement_wakes_genuine_queued_work_before_its_parked_worker_can_drain() {
    let state = Arc::new(ServerState::new());
    let uri = Url::parse("file:///queued.ts").unwrap();
    state.documents.open(
        uri.clone(),
        "const value=1;value;".into(),
        1,
        "typescript".into(),
    );
    let project = Arc::new(NativeNavigationProject::new(
        SourceQueryProject::new_server(state),
    ));
    let (query, cancel) = project.source.begin_query(&uri).unwrap();
    let worker = project.worker(query.snapshot().clone()).unwrap();
    let resume = worker.pause();
    let mut pending =
        Box::pin(query.run(|_| async { worker.definition(Position::new(0, 15)).await }));
    let wake = Arc::new(RetirementWake {
        project: project.clone(),
        wakes: AtomicUsize::new(0),
    });
    let waker = futures::task::waker(wake.clone());
    let mut context = Context::from_waker(&waker);
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    let before = wake.wakes.load(Ordering::Relaxed);
    project.retire();
    assert!(wake.wakes.load(Ordering::Relaxed) > before);
    assert!(cancel.is_aborted());
    assert!(matches!(
        pending.as_mut().poll(&mut context),
        Poll::Ready(Err(SnapshotRefusal::Cancelled))
    ));
    drop(pending);
    resume.send(()).unwrap();
    worker.wait_exit();
    assert_eq!(project.live.load(Ordering::Acquire), 0);
}

#[test]
fn explicit_retirement_drains_original_selected_and_names_workers_with_live_arcs() {
    let state = Arc::new(ServerState::new());
    let original_uri = Url::parse("file:///original.ts").unwrap();
    let selected_uri = Url::parse("file:///selected.vue").unwrap();
    let names_uri = Url::parse("file:///names.vue").unwrap();
    state.documents.open(
        original_uri.clone(),
        "const value=1;value;".into(),
        1,
        "typescript".into(),
    );
    let selected_source = "<template><button @click='let value=$event;return value'/></template>";
    state.documents.open(
        selected_uri.clone(),
        selected_source.into(),
        1,
        "vue".into(),
    );
    state.documents.open(
        names_uri.clone(),
        "<template><Card></Card></template>".into(),
        1,
        "vue".into(),
    );
    let project = NativeNavigationProject::new(SourceQueryProject::new_server(state));
    assert_eq!(
        block_on(project.definition(&original_uri, Position::new(0, 15))),
        Ok(Some(Location::new(
            original_uri.clone(),
            Range::new(Position::new(0, 6), Position::new(0, 11))
        )))
    );
    let declaration = selected_source.find("value").unwrap() as u32;
    let reference = selected_source.rfind("value").unwrap() as u32;
    assert_eq!(
        block_on(project.template_definition(&selected_uri, Position::new(0, reference))),
        Ok(vec![Location::new(
            selected_uri.clone(),
            Range::new(
                Position::new(0, declaration),
                Position::new(0, declaration + 5)
            )
        )])
    );
    assert_eq!(
        block_on(project.linked_editing(&names_uri, Position::new(0, 12))),
        Ok(Some(LinkedEditingRanges {
            ranges: vec![
                Range::new(Position::new(0, 11), Position::new(0, 15)),
                Range::new(Position::new(0, 18), Position::new(0, 22))
            ],
            word_pattern: None
        }))
    );
    let workers = [
        &project.workers,
        &project.selected_workers,
        &project.names_workers,
    ]
    .map(|cache| {
        cache
            .lock()
            .values()
            .next()
            .unwrap()
            .result
            .as_ref()
            .unwrap()
            .clone()
    });
    assert_eq!(workers[0].counts(), (1, 1));
    assert_eq!(workers[1].sfc_productions(), 1);
    assert_eq!(workers[2].counts(), (0, 1));
    project.retire();
    for cache in [
        &project.workers,
        &project.selected_workers,
        &project.names_workers,
    ] {
        assert!(cache.try_lock().unwrap().is_empty());
    }
    for worker in workers {
        worker.wait_exit();
        assert_eq!(
            block_on(worker.definition(Position::new(0, 15))),
            Err(NavigationRefusal::WorkerUnavailable)
        );
    }
    assert_eq!(project.live.load(Ordering::Acquire), 0);
    for uri in [original_uri, selected_uri, names_uri] {
        assert!(matches!(
            project.source.begin_query(&uri),
            Err(SnapshotRefusal::Superseded)
        ));
    }
}
