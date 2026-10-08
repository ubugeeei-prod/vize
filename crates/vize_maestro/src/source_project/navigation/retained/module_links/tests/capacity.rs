use super::{
    Arc, DocumentStore, NativeNavigationProject, NavigationRefusal, SourceQueryProject, Url,
    block_on,
};
use std::sync::atomic::Ordering;

#[test]
fn module_link_existing_live_worker_cap_waits_for_original_owner_exit() {
    let documents = DocumentStore::new();
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let mut uris = Vec::new();
    let mut workers = Vec::new();
    for index in 0..16 {
        let uri = Url::parse(&vize_l0::cstr!("file:///module-cap-{index}.ts")).unwrap();
        documents.open(
            uri.clone(),
            "import './a.ts';".into(),
            1,
            "javascript".into(),
        );
        let (query, _) = project.source.begin_query(&uri).unwrap();
        let worker = project.worker(Arc::clone(query.snapshot())).unwrap();
        assert_eq!(block_on(worker.module_operands()).unwrap().len(), 1);
        uris.push(uri);
        workers.push(worker);
    }
    let extra = Url::parse("file:///module-cap-extra.ts").unwrap();
    documents.open(
        extra.clone(),
        "import './a.ts';".into(),
        1,
        "javascript".into(),
    );
    let (query, _) = project.source.begin_query(&extra).unwrap();
    assert!(matches!(
        project.worker(Arc::clone(query.snapshot())),
        Err(NavigationRefusal::Capacity)
    ));
    assert!(!project.workers.lock().contains_key(&extra));
    let resume = workers[0].pause();
    documents.close(&uris[0]);
    project.notify_host_change(&uris[0]);
    assert_eq!(project.live.load(Ordering::Acquire), 16);
    assert!(matches!(
        project.worker(Arc::clone(query.snapshot())),
        Err(NavigationRefusal::Capacity)
    ));
    resume.send(()).unwrap();
    workers[0].wait_exit();
    assert_eq!(project.live.load(Ordering::Acquire), 15);
    let admitted = project.worker(Arc::clone(query.snapshot())).unwrap();
    assert_eq!(block_on(admitted.module_operands()).unwrap().len(), 1);
    assert_eq!(project.live.load(Ordering::Acquire), 16);
    assert_eq!(workers[1].counts(), (1, 1));
}
