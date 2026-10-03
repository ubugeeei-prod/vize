use super::{
    Arc, DocumentStore, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject,
    block_on, cached, location, uri,
};
use crate::source_project::SnapshotRefusal;
use tower_lsp::lsp_types::TextDocumentContentChangeEvent;

#[test]
fn change_close_reopen_invalidate_equal_version_buffers_without_aba() {
    let documents = DocumentStore::new();
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    project
        .source
        .open(uri(), "const old=1;old;".into(), 1, "javascript".into());
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 13))),
        Ok(Some(location((0, 6), (0, 9))))
    );
    let original = cached(&project);
    project.source.apply_changes(
        &uri(),
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "const newName=1;newName;".into(),
        }],
        2,
    );
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 17))),
        Ok(Some(location((0, 6), (0, 13))))
    );
    assert!(!Arc::ptr_eq(&original, &cached(&project)));
    project.source.close(&uri());
    project.notify_host_change(&uri());
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 13))),
        Err(NavigationRefusal::Host(SnapshotRefusal::MissingDocument))
    );
    project
        .source
        .open(uri(), "const old=1;old;".into(), 1, "javascript".into());
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 13))),
        Ok(Some(location((0, 6), (0, 9))))
    );
    assert!(!Arc::ptr_eq(&original, &cached(&project)));
}

#[test]
fn post_mutation_notification_cancels_old_query_but_preserves_new_query_and_cache() {
    let documents = DocumentStore::new();
    documents.open(uri(), "const value=1;value;".into(), 1, "javascript".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let (old, old_cancel) = project.source.begin_query(&uri()).unwrap();
    documents.open(uri(), "const fresh=1;fresh;".into(), 1, "typescript".into());
    let (fresh, fresh_cancel) = project.source.begin_query(&uri()).unwrap();
    let snapshot = Arc::clone(fresh.snapshot());
    project.notify_host_change(&uri());
    assert!(old_cancel.is_aborted());
    assert!(!fresh_cancel.is_aborted());
    let (same, _) = project.source.begin_query(&uri()).unwrap();
    assert!(Arc::ptr_eq(&snapshot, same.snapshot()));
    assert!(matches!(
        block_on(old.run(|_| async {})),
        Err(SnapshotRefusal::Cancelled)
    ));
    assert_eq!(
        block_on(fresh.run(|_| async { 7 }))
            .unwrap()
            .publish(|value| value),
        Ok(7)
    );
}

#[test]
fn native_ready_response_is_refused_after_change_or_explicit_cancellation() {
    let documents = DocumentStore::new();
    documents.open(uri(), "const value=1;value;".into(), 1, "javascript".into());
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let (query, _) = project.source.begin_query(&uri()).unwrap();
    let owner = &project;
    let ready = block_on(query.run(|snapshot| async move {
        owner
            .worker(snapshot)?
            .definition(Position::new(0, 15))
            .await
    }))
    .unwrap();
    documents.open(uri(), "const other=1;other;".into(), 1, "javascript".into());
    assert_eq!(
        ready.publish(|value| value),
        Err(SnapshotRefusal::Superseded)
    );
    let (query, cancel) = project.source.begin_query(&uri()).unwrap();
    let ready = block_on(query.run(|snapshot| async move {
        owner
            .worker(snapshot)?
            .definition(Position::new(0, 15))
            .await
    }))
    .unwrap();
    cancel.abort();
    assert_eq!(
        ready.publish(|value| value),
        Err(SnapshotRefusal::Cancelled)
    );
}
