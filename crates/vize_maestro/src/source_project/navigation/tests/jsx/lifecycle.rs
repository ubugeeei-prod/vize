use super::{
    Arc, DocumentStore, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject,
    block_on, cached, location, uri,
};
use crate::source_project::{SnapshotRefusal, SourceSnapshotCache};

#[test]
fn react_language_change_and_close_reopen_reject_original_ready_data_without_aba() {
    let documents = DocumentStore::new();
    let source = "const UI=1;const view=<UI/>;";
    documents.open(uri(), source.into(), 1, "javascriptreact".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 23))),
        Ok(Some(location((0, 6), (0, 8))))
    );
    let original = cached(&project);
    let (query, _) = project.source.begin_query(&uri()).unwrap();
    let owner = &project;
    let ready =
        block_on(query.run(|snapshot| async move {
            owner.summary(snapshot)?.definition(Position::new(0, 23))
        }))
        .unwrap();
    documents.open(uri(), source.into(), 1, "typescript".into());
    assert_eq!(
        ready.publish(|value| value),
        Err(SnapshotRefusal::Superseded)
    );
    project.notify_host_change(&uri());
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 23))),
        Err(NavigationRefusal::Syntax)
    );
    project.source.close(&uri());
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 23))),
        Err(NavigationRefusal::Host(SnapshotRefusal::MissingDocument))
    );
    project
        .source
        .open(uri(), source.into(), 1, "typescriptreact".into());
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 23))),
        Ok(Some(location((0, 6), (0, 8))))
    );
    assert!(!Arc::ptr_eq(&original, &cached(&project)));
    let (query, cancel) = project.source.begin_query(&uri()).unwrap();
    let ready =
        block_on(query.run(|snapshot| async move {
            owner.summary(snapshot)?.definition(Position::new(0, 23))
        }))
        .unwrap();
    cancel.abort();
    assert_eq!(
        ready.publish(|value| value),
        Err(SnapshotRefusal::Cancelled)
    );
}

#[test]
fn equal_foreign_react_source_cannot_reuse_original_snapshot_summary() {
    let documents = DocumentStore::new();
    let foreign = DocumentStore::new();
    for store in [&documents, &foreign] {
        store.open(
            uri(),
            "const UI=1;const view=<UI/>;".into(),
            1,
            "typescriptreact".into(),
        );
    }
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let (local, _) = project.source.begin_query(&uri()).unwrap();
    let summary = project.summary(Arc::clone(local.snapshot())).unwrap();
    let snapshot = SourceSnapshotCache::default()
        .capture(&foreign, &uri())
        .unwrap();
    assert_eq!(snapshot.source(), local.snapshot().source());
    assert!(!summary.belongs_to(&snapshot));
    let rebuilt = project.summary(snapshot).unwrap();
    assert!(!Arc::ptr_eq(&summary, &rebuilt));
}
