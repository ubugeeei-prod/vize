#![expect(
    clippy::disallowed_types,
    reason = "completed work must retain the exact shared store or existing server owner until guarded publication"
)]

use std::sync::Arc;

use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};

use crate::document::DocumentStore;
use crate::runtime::block_on;
use crate::server::ServerState;
use crate::source_project::{SnapshotRefusal, SourceQueryProject};

#[test]
fn shared_results_recheck_external_host_edits_and_retain_host_until_cancelled_publication() {
    let documents = Arc::new(DocumentStore::new());
    let retained_store = Arc::downgrade(&documents);
    let controller = Arc::new(SourceQueryProject::new_shared(Arc::clone(&documents)));
    let uri = Url::parse("file:///shared-ready.ts").unwrap();
    controller.open(uri.clone(), "before".into(), 1, "typescript".into());
    let (old, old_cancel) = controller.begin_query(&uri).unwrap();
    let ready = block_on(old.run(|_| async { 7 })).unwrap();
    // A direct external mutation does not notify this controller's registry,
    // but its genuine guarded publication must still consult the same store.
    assert!(documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "after".into()
        }],
        2
    ));
    assert!(!old_cancel.is_aborted());
    let mut old_published = false;
    assert_eq!(
        ready.publish(|_| old_published = true),
        Err(SnapshotRefusal::Superseded)
    );
    assert!(!old_published);
    let (fresh, fresh_cancel) = controller.begin_query(&uri).unwrap();
    let fresh_ready = block_on(fresh.run(|snapshot| async move { snapshot.version() })).unwrap();
    drop(documents);
    drop(controller);
    assert!(fresh_cancel.is_aborted());
    assert!(retained_store.upgrade().is_some());
    let mut fresh_published = false;
    assert_eq!(
        fresh_ready.publish(|_| fresh_published = true),
        Err(SnapshotRefusal::Cancelled)
    );
    assert!(!fresh_published);
    assert!(retained_store.upgrade().is_none());
}

#[test]
fn real_server_owner_projects_inline_store_across_open_change_close_and_guarded_queries() {
    let state = Arc::new(ServerState::new());
    let retained_state = Arc::downgrade(&state);
    let controller = Arc::new(SourceQueryProject::new_server(Arc::clone(&state)));
    let request = Arc::clone(&controller);
    let lifecycle = Arc::clone(&controller);
    assert!(core::ptr::eq(controller.host.documents(), &state.documents));
    let uri = Url::parse("file:///server-owned.ts").unwrap();
    lifecycle.open(uri.clone(), "a😀b".into(), 1, "typescript".into());
    assert_eq!(state.documents.text(&uri).as_deref(), Some("a😀b"));
    let (old, old_cancel) = request.begin_query(&uri).unwrap();
    let original = Arc::clone(old.snapshot());
    let ready =
        block_on(old.run(|snapshot| async move { snapshot.source().as_bytes().to_vec() })).unwrap();
    drop(state);
    assert!(retained_state.upgrade().is_some());
    assert!(lifecycle.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: Some(Range::new(Position::new(0, 1), Position::new(0, 3))),
            range_length: None,
            text: "α".into(),
        }],
        2
    ));
    assert!(old_cancel.is_aborted());
    let mut old_published = false;
    assert_eq!(
        ready.publish(|_| old_published = true),
        Err(SnapshotRefusal::Superseded)
    );
    assert!(!old_published);
    let (fresh, fresh_cancel) = request.begin_query(&uri).unwrap();
    assert_eq!(fresh.snapshot().source(), "aαb");
    assert_ne!(fresh.snapshot().revision(), original.revision());
    assert_eq!(fresh.snapshot().version(), 2);
    let fresh_ready =
        block_on(fresh.run(|snapshot| async move { snapshot.source().as_bytes().to_vec() }))
            .unwrap();
    lifecycle.close(&uri);
    assert!(fresh_cancel.is_aborted());
    let mut closed_published = false;
    assert_eq!(
        fresh_ready.publish(|_| closed_published = true),
        Err(SnapshotRefusal::MissingDocument)
    );
    assert!(!closed_published);
    drop(request);
    drop(lifecycle);
    drop(controller);
    assert!(retained_state.upgrade().is_none());
}
