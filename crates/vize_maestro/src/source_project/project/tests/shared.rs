#![expect(
    clippy::disallowed_types,
    reason = "real request and lifecycle handles share the same controller and host allocation"
)]

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::task::{Context, Poll};

use futures::Future;
use futures::task::waker_ref;
use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};

use crate::document::DocumentStore;
use crate::runtime::block_on;
use crate::source_project::{SnapshotRefusal, SourceQueryProject};

use super::WakeCounter;

mod publication;

#[test]
fn real_shared_request_and_lifecycle_handles_reuse_the_same_host_buffer() {
    let documents = Arc::new(DocumentStore::new());
    let controller = Arc::new(SourceQueryProject::new_shared(Arc::clone(&documents)));
    let request = Arc::clone(&controller);
    let lifecycle = Arc::clone(&controller);
    let uri = Url::parse("file:///shared-source.vue").unwrap();
    lifecycle.open(uri.clone(), "<div>😀</div>".into(), 7, "vue".into());
    assert!(core::ptr::eq(request.host.documents(), documents.as_ref()));
    let (first, _) = request.begin_query(&uri).unwrap();
    let original = Arc::clone(first.snapshot());
    let (second, _) = lifecycle.begin_query(&uri).unwrap();
    assert!(Arc::ptr_eq(&original, second.snapshot()));
    assert_eq!(original.uri(), &uri);
    let ready = block_on(first.run(|snapshot| async move {
        (
            snapshot.revision(),
            snapshot.version(),
            snapshot.source().as_ptr(),
        )
    }))
    .unwrap();
    assert_eq!(
        ready.publish(|value| value),
        Ok((original.revision(), 7, original.source().as_ptr()))
    );
    assert_eq!(
        original.native_file(&documents),
        Err(SnapshotRefusal::NativeFileUnavailable)
    );
}

#[test]
fn incremental_change_across_shared_handles_refreshes_key_and_refuses_old_publication() {
    let documents = Arc::new(DocumentStore::new());
    let controller = Arc::new(SourceQueryProject::new_shared(Arc::clone(&documents)));
    let request = Arc::clone(&controller);
    let lifecycle = Arc::clone(&controller);
    let uri = Url::parse("file:///shared-incremental.ts").unwrap();
    lifecycle.open(uri.clone(), "a😀b\r\nvalue".into(), 1, "typescript".into());
    let (old, cancel) = request.begin_query(&uri).unwrap();
    let original = Arc::clone(old.snapshot());
    let ready =
        block_on(old.run(|snapshot| async move { snapshot.source().as_bytes().to_vec() })).unwrap();
    assert!(lifecycle.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: Some(Range::new(Position::new(0, 1), Position::new(0, 3))),
            range_length: None,
            text: "α".into(),
        }],
        2
    ));
    assert!(cancel.is_aborted());
    let mut stale_published = false;
    assert_eq!(
        ready.publish(|_| stale_published = true),
        Err(SnapshotRefusal::Superseded)
    );
    assert!(!stale_published);
    let (fresh, _) = request.begin_query(&uri).unwrap();
    assert_eq!(fresh.snapshot().source(), "aαb\r\nvalue");
    assert_ne!(fresh.snapshot().revision(), original.revision());
    assert_eq!(fresh.snapshot().version(), 2);
    assert_eq!(original.source(), "a😀b\r\nvalue");
    assert_eq!(documents.text(&uri).as_deref(), Some("aαb\r\nvalue"));
    let fresh_ready =
        block_on(fresh.run(|snapshot| async move { snapshot.source().as_bytes().to_vec() }))
            .unwrap();
    assert_eq!(
        fresh_ready.publish(|value| value),
        Ok("aαb\r\nvalue".as_bytes().to_vec())
    );
    lifecycle.close(&uri);
    assert!(matches!(
        request.begin_query(&uri),
        Err(SnapshotRefusal::MissingDocument)
    ));
}

#[test]
fn only_last_controller_handle_drop_wakes_work_and_query_retains_the_real_store() {
    let documents = Arc::new(DocumentStore::new());
    let retained_store = Arc::downgrade(&documents);
    let controller = Arc::new(SourceQueryProject::new_shared(Arc::clone(&documents)));
    let lifecycle = Arc::clone(&controller);
    let uri = Url::parse("file:///shared-pending.ts").unwrap();
    lifecycle.open(uri.clone(), "let value;".into(), 1, "typescript".into());
    let (query, cancel) = controller.begin_query(&uri).unwrap();
    let mut running = Box::pin(query.run(|snapshot| async move {
        let _retained_source = snapshot;
        futures::future::pending::<()>().await;
    }));
    let wake = Arc::new(WakeCounter::default());
    let waker = waker_ref(&wake);
    let mut context = Context::from_waker(&waker);
    assert!(running.as_mut().poll(&mut context).is_pending());
    assert!(running.as_mut().poll(&mut context).is_pending());
    drop(documents);
    drop(controller);
    assert!(!cancel.is_aborted());
    assert!(retained_store.upgrade().is_some());
    let before = wake.0.load(Ordering::Relaxed);
    drop(lifecycle);
    assert!(cancel.is_aborted());
    assert!(wake.0.load(Ordering::Relaxed) > before);
    assert!(retained_store.upgrade().is_some());
    assert!(matches!(
        running.as_mut().poll(&mut context),
        Poll::Ready(Err(SnapshotRefusal::Cancelled))
    ));
    drop(running);
    assert!(retained_store.upgrade().is_none());
}
