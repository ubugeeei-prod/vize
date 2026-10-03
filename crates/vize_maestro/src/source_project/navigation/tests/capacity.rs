use super::{
    Arc, DocumentStore, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject,
    block_on, cached, location, uri,
};
use std::{future::Future, sync::atomic::Ordering, task::Context};
use tower_lsp::lsp_types::Url;

#[test]
fn a_full_mailbox_refuses_without_blocking_or_discarding_other_requests() {
    let documents = DocumentStore::new();
    documents.open(uri(), "const value=1;value;".into(), 1, "javascript".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    block_on(project.definition(&uri(), Position::new(0, 15))).unwrap();
    let worker = cached(&project);
    let resume = worker.pause();
    let mut context = Context::from_waker(futures::task::noop_waker_ref());
    let mut pending = Vec::new();
    for _ in 0..16 {
        let mut request = Box::pin(worker.definition(Position::new(0, 15)));
        assert!(request.as_mut().poll(&mut context).is_pending());
        pending.push(request);
    }
    assert_eq!(
        block_on(worker.definition(Position::new(0, 15))),
        Err(NavigationRefusal::Busy)
    );
    drop(pending);
    resume.send(()).unwrap();
    block_on(worker.drain_barrier(Position::new(0, 15))).unwrap();
    assert_eq!(
        block_on(worker.definition(Position::new(0, 15))),
        Ok(Some(location((0, 6), (0, 11))))
    );
    assert_eq!(worker.counts(), (1, 2));
}

#[test]
fn live_worker_capacity_is_released_only_after_native_owner_exit_and_is_retryable() {
    let documents = DocumentStore::new();
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let mut uris = Vec::new();
    let mut workers = Vec::new();
    for index in 0..16 {
        let uri = Url::parse(&vize_l0::cstr!("file:///capacity-{index}.js")).unwrap();
        documents.open(
            uri.clone(),
            "const value=1;value;".into(),
            1,
            "javascript".into(),
        );
        block_on(project.definition(&uri, Position::new(0, 15))).unwrap();
        workers.push(Arc::clone(
            project
                .workers
                .lock()
                .get(&uri)
                .unwrap()
                .result
                .as_ref()
                .unwrap(),
        ));
        uris.push(uri);
    }
    let extra = Url::parse("file:///capacity-extra.js").unwrap();
    documents.open(
        extra.clone(),
        "const value=1;value;".into(),
        1,
        "javascript".into(),
    );
    assert_eq!(
        block_on(project.definition(&extra, Position::new(0, 15))),
        Err(NavigationRefusal::Capacity)
    );
    assert!(!project.workers.lock().contains_key(&extra));
    let resume = workers[0].pause();
    documents.close(&uris[0]);
    project.notify_host_change(&uris[0]);
    assert_eq!(project.live.load(Ordering::Acquire), 16);
    assert_eq!(
        block_on(project.definition(&extra, Position::new(0, 15))),
        Err(NavigationRefusal::Capacity)
    );
    resume.send(()).unwrap();
    workers[0].wait_exit();
    assert_eq!(project.live.load(Ordering::Acquire), 15);
    assert!(
        block_on(project.definition(&extra, Position::new(0, 15)))
            .unwrap()
            .is_some()
    );
    assert_eq!(project.live.load(Ordering::Acquire), 16);
    assert!(workers[1].belongs_to(&project.workers.lock().get(&uris[1]).unwrap().snapshot));
    assert_eq!(workers[1].counts(), (1, 1));
}
