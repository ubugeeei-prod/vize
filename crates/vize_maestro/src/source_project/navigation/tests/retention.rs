use super::{
    Arc, DocumentStore, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject,
    block_on, cached, location, uri,
};
use crate::source_project::SnapshotRefusal;
use std::{
    future::Future,
    sync::atomic::Ordering,
    task::{Context, Poll},
};
use tower_lsp::lsp_types::TextDocumentContentChangeEvent;
use vize_l0::Span;

#[cfg(feature = "native")]
#[test]
fn module_link_actual_context_change_preserves_original_program_worker_and_unrelated_queries() {
    let state = Arc::new(crate::server::ServerState::new());
    state.set_workspace_root("/original-root".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new_server(Arc::clone(&state)));
    project
        .source
        .open(uri(), "const value=1;value;".into(), 1, "javascript".into());
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 15))),
        Ok(Some(location((0, 6), (0, 11))))
    );
    let original = cached(&project);
    let before = block_on(original.inspect(Position::new(0, 15))).unwrap();
    let context = project.source.capture_module_link_context().unwrap();
    let (_, cancel) = project.source.begin_query(&uri()).unwrap();
    state.set_workspace_root("/new-root".into());
    assert!(!cancel.is_aborted());
    assert_eq!(
        state.with_current_module_link_context(&context, || ()),
        Err(crate::server::ModuleLinkContextError::Superseded)
    );
    assert_eq!(
        block_on(project.references(&uri(), Position::new(0, 7), true)),
        Ok(vec![location((0, 6), (0, 11)), location((0, 14), (0, 19))])
    );
    assert!(Arc::ptr_eq(&original, &cached(&project)));
    assert_eq!(
        block_on(original.inspect(Position::new(0, 15))).unwrap(),
        before
    );
    assert_eq!(before.parses, 1);
}

fn project(documents: &DocumentStore) -> NativeNavigationProject<'_> {
    documents.open(uri(), "const value=1;value;".into(), 1, "javascript".into());
    NativeNavigationProject::new(SourceQueryProject::new(documents))
}

#[test]
fn repeated_requests_query_the_same_live_original_file_and_program_once() {
    let documents = DocumentStore::new();
    let project = project(&documents);
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 15))),
        Ok(Some(location((0, 6), (0, 11))))
    );
    let worker = cached(&project);
    let before = block_on(worker.inspect(Position::new(0, 15))).unwrap();
    assert_eq!(before.statements, 2);
    assert_eq!(before.declaration, Some(Span::new(6, 11)));
    assert_eq!(before.parses, 1);
    assert_eq!(
        block_on(project.references(&uri(), Position::new(0, 7), true)),
        Ok(vec![location((0, 6), (0, 11)), location((0, 14), (0, 19))])
    );
    let after = block_on(worker.inspect(Position::new(0, 7))).unwrap();
    assert_eq!(before, after);
    assert_eq!(worker.counts(), (1, 2));
}

#[test]
fn dropped_waiting_request_skips_projection_and_preserves_other_requests() {
    let documents = DocumentStore::new();
    let project = project(&documents);
    let (query, _) = project.source.begin_query(&uri()).unwrap();
    let worker = project.worker(Arc::clone(query.snapshot())).unwrap();
    let resume = worker.pause();
    let mut pending = Box::pin(worker.definition(Position::new(0, 15)));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert_eq!(pending.as_mut().poll(&mut context), Poll::Pending);
    drop(pending);
    resume.send(()).unwrap();
    assert_eq!(
        block_on(worker.definition(Position::new(0, 15))),
        Ok(Some(location((0, 6), (0, 11))))
    );
    assert_eq!(worker.counts(), (1, 1));
}

#[test]
fn explicit_query_abort_wakes_a_queued_request_without_waiting_for_worker() {
    let documents = DocumentStore::new();
    let project = project(&documents);
    let (query, cancel) = project.source.begin_query(&uri()).unwrap();
    let worker = project.worker(Arc::clone(query.snapshot())).unwrap();
    let resume = worker.pause();
    let mut pending =
        Box::pin(query.run(|_| async { worker.definition(Position::new(0, 15)).await }));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    cancel.abort();
    assert!(matches!(
        pending.as_mut().poll(&mut context),
        Poll::Ready(Err(SnapshotRefusal::Cancelled))
    ));
    drop(pending);
    resume.send(()).unwrap();
    assert_eq!(
        block_on(worker.definition(Position::new(0, 15))),
        Ok(Some(location((0, 6), (0, 11))))
    );
    assert_eq!(worker.counts(), (1, 1));
}

#[test]
fn close_retires_original_owners_even_with_an_outstanding_handle() {
    let documents = DocumentStore::new();
    let project = project(&documents);
    block_on(project.definition(&uri(), Position::new(0, 15))).unwrap();
    let worker = cached(&project);
    let weak = Arc::downgrade(&project.workers.lock().get(&uri()).unwrap().snapshot);
    project.source.close(&uri());
    project.notify_host_change(&uri());
    worker.wait_exit();
    assert!(weak.upgrade().is_none());
    assert_eq!(project.live.load(Ordering::Acquire), 0);
    assert_eq!(
        block_on(worker.definition(Position::new(0, 15))),
        Err(NavigationRefusal::WorkerUnavailable)
    );
}

#[test]
fn old_computation_cannot_replace_or_reparse_a_newer_cached_snapshot() {
    let documents = DocumentStore::new();
    let project = project(&documents);
    let (old, _) = project.source.begin_query(&uri()).unwrap();
    let snapshot = Arc::clone(old.snapshot());
    project.source.apply_changes(
        &uri(),
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "const newer=1;newer;".into(),
        }],
        2,
    );
    block_on(project.definition(&uri(), Position::new(0, 15))).unwrap();
    let current = cached(&project);
    assert!(matches!(
        project.worker(snapshot),
        Err(NavigationRefusal::Host(SnapshotRefusal::Superseded))
    ));
    assert!(Arc::ptr_eq(&current, &cached(&project)));
    assert_eq!(current.counts(), (1, 1));
}

#[test]
fn a_completed_empty_cache_notification_prevents_stale_worker_admission() {
    for closed in [false, true] {
        let documents = DocumentStore::new();
        let project = project(&documents);
        let (old, _) = project.source.begin_query(&uri()).unwrap();
        let snapshot = Arc::clone(old.snapshot());
        if closed {
            documents.close(&uri());
        } else {
            documents.apply_changes(
                &uri(),
                vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: "const newer=1;newer;".into(),
                }],
                2,
            );
        }
        project.notify_host_change(&uri());
        assert!(matches!(
            project.worker(snapshot),
            Err(NavigationRefusal::Host(SnapshotRefusal::Superseded))
        ));
        assert!(project.workers.lock().is_empty());
        assert_eq!(project.live.load(Ordering::Acquire), 0);
    }
}

#[test]
fn project_drop_retires_workers_without_joining_under_the_cache_lock() {
    let documents = DocumentStore::new();
    let project = project(&documents);
    block_on(project.definition(&uri(), Position::new(0, 15))).unwrap();
    let worker = cached(&project);
    let live = Arc::clone(&project.live);
    let weak = Arc::downgrade(&project.workers.lock().get(&uri()).unwrap().snapshot);
    drop(project);
    worker.wait_exit();
    assert_eq!(live.load(Ordering::Acquire), 0);
    assert!(weak.upgrade().is_none());
}

#[test]
fn actual_worker_unwind_releases_owners_and_returns_a_refusal() {
    let documents = DocumentStore::new();
    let project = project(&documents);
    block_on(project.definition(&uri(), Position::new(0, 15))).unwrap();
    let worker = cached(&project);
    worker.panic_worker();
    worker.wait_exit();
    assert_eq!(project.live.load(Ordering::Acquire), 0);
    assert_eq!(
        block_on(worker.definition(Position::new(0, 15))),
        Err(NavigationRefusal::WorkerUnavailable)
    );
}
