use super::{
    Arc, DocumentStore, ModuleOperandRefusal, NativeNavigationProject, NavigationRefusal, Position,
    SnapshotRefusal, SourceQueryProject, SourceSnapshotCache, block_on, uri, worker,
};
use std::{
    future::Future,
    sync::atomic::Ordering,
    task::{Context, Poll},
};

#[test]
fn module_link_dropped_queued_reply_skips_collection_and_keeps_original_worker() {
    let documents = DocumentStore::new();
    documents.open(
        uri(),
        "import './a.ts'; const value=1;value;".into(),
        1,
        "javascript".into(),
    );
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let worker = worker(&project);
    let before = block_on(worker.inspect(Position::new(0, 31))).unwrap();
    let resume = worker.pause();
    let mut pending = Box::pin(worker.module_operands());
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert_eq!(pending.as_mut().poll(&mut context), Poll::Pending);
    drop(pending);
    resume.send(()).unwrap();
    let operands = block_on(worker.module_operands()).unwrap();
    assert_eq!(operands.len(), 1);
    assert_eq!(
        block_on(worker.inspect(Position::new(0, 31))).unwrap(),
        before
    );
    assert_eq!(worker.counts(), (1, 1));
}

#[test]
fn module_link_original_query_abort_wakes_without_waiting_for_parked_worker() {
    let documents = DocumentStore::new();
    documents.open(uri(), "import './a.ts';".into(), 1, "javascript".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let (query, cancel) = project.source.begin_query(&uri()).unwrap();
    let worker = project.worker(Arc::clone(query.snapshot())).unwrap();
    let resume = worker.pause();
    let mut pending = Box::pin(query.run(|_| worker.module_operands()));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    cancel.abort();
    assert!(matches!(
        pending.as_mut().poll(&mut context),
        Poll::Ready(Err(SnapshotRefusal::Cancelled))
    ));
    drop(pending);
    resume.send(()).unwrap();
    assert_eq!(block_on(worker.module_operands()).unwrap().len(), 1);
    assert_eq!(worker.counts(), (1, 1));
}

#[test]
fn module_link_full_existing_mailbox_refuses_and_releases_all_dropped_replies() {
    let documents = DocumentStore::new();
    documents.open(uri(), "import './a.ts';".into(), 1, "javascript".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let worker = worker(&project);
    let resume = worker.pause();
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    let mut pending = Vec::new();
    for _ in 0..16 {
        let mut request = Box::pin(worker.module_operands());
        assert!(request.as_mut().poll(&mut context).is_pending());
        pending.push(request);
    }
    assert_eq!(
        block_on(worker.module_operands()),
        Err(ModuleOperandRefusal::Navigation(NavigationRefusal::Busy))
    );
    drop(pending);
    resume.send(()).unwrap();
    block_on(worker.drain_barrier(Position::new(0, 0))).unwrap();
    assert_eq!(block_on(worker.module_operands()).unwrap().len(), 1);
    assert_eq!(worker.counts(), (1, 1));
}

#[test]
fn module_link_ready_operands_cannot_publish_after_edit_close_aba_or_cancel() {
    for mutation in 0..4 {
        let documents = DocumentStore::new();
        let source = "import './a.ts';";
        documents.open(uri(), source.into(), 1, "javascript".into());
        let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
        let (query, cancel) = project.source.begin_query(&uri()).unwrap();
        let worker = project.worker(Arc::clone(query.snapshot())).unwrap();
        let ready = block_on(query.run(|_| worker.module_operands())).unwrap();
        let expected = match mutation {
            0 => {
                documents.open(uri(), "import './b.ts';".into(), 1, "javascript".into());
                SnapshotRefusal::Superseded
            }
            1 => {
                documents.close(&uri());
                SnapshotRefusal::MissingDocument
            }
            2 => {
                documents.close(&uri());
                documents.open(uri(), source.into(), 1, "javascript".into());
                SnapshotRefusal::Superseded
            }
            _ => {
                cancel.abort();
                SnapshotRefusal::Cancelled
            }
        };
        assert_eq!(ready.publish(|response| response), Err(expected));
        project.notify_host_change(&uri());
    }
}

#[test]
fn module_link_equal_foreign_snapshot_does_not_reuse_original_program_worker() {
    let documents = DocumentStore::new();
    let foreign = DocumentStore::new();
    for store in [&documents, &foreign] {
        store.open(uri(), "import './a.ts';".into(), 1, "javascript".into());
    }
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let worker = worker(&project);
    let snapshot = SourceSnapshotCache::default()
        .capture(&foreign, &uri())
        .unwrap();
    assert!(!worker.belongs_to(&snapshot));
    assert!(matches!(
        project.worker(snapshot),
        Err(NavigationRefusal::Host(SnapshotRefusal::Superseded))
    ));
    assert_eq!(block_on(worker.module_operands()).unwrap().len(), 1);
    assert_eq!(worker.counts(), (1, 1));
}

#[test]
fn module_link_close_and_project_drop_retire_original_view_and_native_owners() {
    for closed in [false, true] {
        let documents = DocumentStore::new();
        documents.open(uri(), "import './a.ts';".into(), 1, "javascript".into());
        let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
        let worker = worker(&project);
        assert_eq!(block_on(worker.module_operands()).unwrap().len(), 1);
        let weak = Arc::downgrade(&project.workers.lock().get(&uri()).unwrap().snapshot);
        let live = Arc::clone(&project.live);
        if closed {
            documents.close(&uri());
            project.notify_host_change(&uri());
        } else {
            drop(project);
        }
        worker.wait_exit();
        assert!(weak.upgrade().is_none());
        assert_eq!(live.load(Ordering::Acquire), 0);
        assert_eq!(
            block_on(worker.module_operands()),
            Err(ModuleOperandRefusal::Navigation(
                NavigationRefusal::WorkerUnavailable
            ))
        );
    }
}

#[test]
fn module_link_actual_worker_unwind_releases_view_and_refuses_new_commands() {
    let documents = DocumentStore::new();
    documents.open(uri(), "import './a.ts';".into(), 1, "javascript".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let worker = worker(&project);
    assert_eq!(block_on(worker.module_operands()).unwrap().len(), 1);
    worker.panic_worker();
    worker.wait_exit();
    assert_eq!(project.live.load(Ordering::Acquire), 0);
    assert_eq!(
        block_on(worker.module_operands()),
        Err(ModuleOperandRefusal::Navigation(
            NavigationRefusal::WorkerUnavailable
        ))
    );
}
