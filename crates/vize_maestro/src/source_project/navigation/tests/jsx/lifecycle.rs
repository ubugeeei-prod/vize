use super::{
    Arc, DocumentStore, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject,
    block_on, cached, location, uri,
};
use crate::source_project::{SnapshotRefusal, SourceSnapshotCache};
use std::{
    future::Future,
    sync::atomic::Ordering,
    task::{Context, Poll},
};

const SOURCE: &str = "const value=1;\nconst tree=<div>{value}</div>;";

#[test]
fn equal_foreign_and_previous_language_snapshots_cannot_reuse_the_jsx_worker() {
    let documents = DocumentStore::new();
    let foreign = DocumentStore::new();
    for store in [&documents, &foreign] {
        store.open(uri(), SOURCE.into(), 1, "javascriptreact".into());
    }
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let (local, _) = project.source.begin_query(&uri()).unwrap();
    let old_snapshot = Arc::clone(local.snapshot());
    let worker = project.worker(Arc::clone(&old_snapshot)).unwrap();
    let foreign = SourceSnapshotCache::default()
        .capture(&foreign, &uri())
        .unwrap();
    assert_eq!(foreign.source(), old_snapshot.source());
    assert!(!worker.belongs_to(&foreign));
    assert!(matches!(
        project.worker(foreign),
        Err(NavigationRefusal::Host(SnapshotRefusal::Superseded))
    ));
    documents.open(uri(), SOURCE.into(), 1, "typescriptreact".into());
    project.notify_host_change(&uri());
    worker.wait_exit();
    assert!(matches!(
        project.worker(old_snapshot),
        Err(NavigationRefusal::Host(SnapshotRefusal::Superseded))
    ));
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(1, 18))),
        Ok(Some(location((0, 6), (0, 11))))
    );
    assert!(!Arc::ptr_eq(&worker, &cached(&project)));
}

#[test]
fn actual_change_aborts_a_queued_jsx_request_and_keeps_the_new_owner() {
    let documents = DocumentStore::new();
    documents.open(uri(), SOURCE.into(), 1, "javascriptreact".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let (query, _) = project.source.begin_query(&uri()).unwrap();
    let worker = project.worker(Arc::clone(query.snapshot())).unwrap();
    let resume = worker.pause();
    let mut pending =
        Box::pin(query.run(|_| async { worker.definition(Position::new(1, 18)).await }));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    documents.open(
        uri(),
        "const newer=1;\nconst tree=<div>{newer}</div>;".into(),
        1,
        "typescriptreact".into(),
    );
    project.notify_host_change(&uri());
    assert!(matches!(
        pending.as_mut().poll(&mut context),
        Poll::Ready(Err(SnapshotRefusal::Cancelled))
    ));
    drop(pending);
    resume.send(()).unwrap();
    worker.wait_exit();
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(1, 18))),
        Ok(Some(location((0, 6), (0, 11))))
    );
    assert_eq!(cached(&project).counts(), (1, 1));
    assert!(!Arc::ptr_eq(&worker, &cached(&project)));
}

#[test]
fn ready_tsx_responses_recheck_current_source_and_explicit_cancellation() {
    let documents = DocumentStore::new();
    documents.open(uri(), SOURCE.into(), 1, "typescriptreact".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let owner = &project;
    let (query, _) = project.source.begin_query(&uri()).unwrap();
    let ready = block_on(query.run(|snapshot| async move {
        owner
            .worker(snapshot)?
            .definition(Position::new(1, 18))
            .await
    }))
    .unwrap();
    documents.open(uri(), SOURCE.into(), 1, "typescriptreact".into());
    assert_eq!(
        ready.publish(|response| response),
        Err(SnapshotRefusal::Superseded)
    );
    let (query, cancel) = project.source.begin_query(&uri()).unwrap();
    let ready = block_on(query.run(|snapshot| async move {
        owner
            .worker(snapshot)?
            .definition(Position::new(1, 18))
            .await
    }))
    .unwrap();
    cancel.abort();
    assert_eq!(
        ready.publish(|response| response),
        Err(SnapshotRefusal::Cancelled)
    );
}

#[test]
fn close_reopen_and_drop_retire_each_original_react_language_owner() {
    for language in ["javascriptreact", "typescriptreact"] {
        let documents = DocumentStore::new();
        documents.open(uri(), SOURCE.into(), 1, language.into());
        let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
        block_on(project.definition(&uri(), Position::new(1, 18))).unwrap();
        let worker = cached(&project);
        let snapshot = Arc::downgrade(&project.workers.lock().get(&uri()).unwrap().snapshot);
        project.source.close(&uri());
        project.notify_host_change(&uri());
        worker.wait_exit();
        assert!(snapshot.upgrade().is_none());
        assert_eq!(project.live.load(Ordering::Acquire), 0);
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(1, 18))),
            Err(NavigationRefusal::Host(SnapshotRefusal::MissingDocument))
        );
        documents.open(uri(), SOURCE.into(), 1, language.into());
        assert_eq!(
            block_on(project.definition(&uri(), Position::new(1, 18))),
            Ok(Some(location((0, 6), (0, 11))))
        );
        let reopened = cached(&project);
        assert!(!Arc::ptr_eq(&worker, &reopened));
        let live = Arc::clone(&project.live);
        drop(project);
        reopened.wait_exit();
        assert_eq!(live.load(Ordering::Acquire), 0);
    }
}
