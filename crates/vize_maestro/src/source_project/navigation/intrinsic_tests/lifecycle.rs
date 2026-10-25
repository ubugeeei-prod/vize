use super::{
    Arc, DocumentStore, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject,
    block_on, cached, uri,
};
use crate::source_project::SnapshotRefusal;
use std::{
    future::Future,
    task::{Context, Poll},
};

const SOURCE: &str = "export const view = <button/>;";
const AT: Position = Position {
    line: 0,
    character: 22,
};

#[test]
fn changed_source_cancels_the_queued_intrinsic_query_before_old_reply_publication() {
    let documents = DocumentStore::new();
    documents.open(uri(), SOURCE.into(), 1, "javascriptreact".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    assert_eq!(
        block_on(project.intrinsic_tag(&uri(), AT))
            .unwrap()
            .unwrap()
            .name
            .as_str(),
        "button"
    );
    let worker = cached(&project);
    let resume = worker.pause();
    let target_uri = uri();
    let mut pending = Box::pin(project.intrinsic_tag(&target_uri, AT));
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    documents.open(
        uri(),
        "export const view = <input/>;".into(),
        1,
        "typescriptreact".into(),
    );
    project.notify_host_change(&uri());
    assert!(matches!(
        pending.as_mut().poll(&mut context),
        Poll::Ready(Err(NavigationRefusal::Host(SnapshotRefusal::Cancelled)))
    ));
    drop(pending);
    resume.send(()).unwrap();
    worker.wait_exit();
    assert_eq!(
        block_on(project.intrinsic_tag(&uri(), AT))
            .unwrap()
            .unwrap()
            .name
            .as_str(),
        "input"
    );
    assert!(!Arc::ptr_eq(&worker, &cached(&project)));
    assert_eq!(cached(&project).counts(), (1, 1));
    assert_eq!(cached(&project).program_walks(), 1);
    documents.close(&uri());
    let newer = cached(&project);
    project.notify_host_change(&uri());
    newer.wait_exit();
    assert_eq!(
        block_on(project.intrinsic_tag(&uri(), AT)),
        Err(NavigationRefusal::Host(SnapshotRefusal::MissingDocument))
    );
}

#[test]
fn ready_intrinsic_data_cannot_escape_same_version_reopen_or_cancellation() {
    let documents = DocumentStore::new();
    documents.open(uri(), SOURCE.into(), 1, "typescriptreact".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    for reopen in [true, false] {
        let (query, cancel) = project.source.begin_query(&uri()).unwrap();
        let ready = block_on(
            query.run(|snapshot| async { project.worker(snapshot)?.intrinsic_tag(AT).await }),
        )
        .unwrap();
        let expected = if reopen {
            documents.open(uri(), SOURCE.into(), 1, "typescriptreact".into());
            SnapshotRefusal::Superseded
        } else {
            cancel.abort();
            SnapshotRefusal::Cancelled
        };
        assert_eq!(ready.publish(|reply| reply), Err(expected));
    }
}
