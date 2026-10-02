#![expect(
    clippy::disallowed_types,
    reason = "real host buffers and future wake/drop observations use standard Arc sharing"
)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::task::{Context, Poll};

use futures::Future;
use futures::task::{ArcWake, waker_ref};
use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};

use crate::document::DocumentStore;
use crate::runtime::block_on;

use super::{SnapshotRefusal, SourceSnapshotCache};

fn uri(path: &str) -> Url {
    Url::parse(path).expect("actual host URI")
}

fn replace(text: &str) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: text.into(),
    }
}

#[test]
fn same_actual_revision_reuses_one_original_buffer_even_with_equal_content_updates() {
    let documents = DocumentStore::new();
    let uri = uri("file:///snapshot.vue");
    documents.open(uri.clone(), "<div>α</div>".into(), 7, "vue".into());
    let cache = SourceSnapshotCache::default();
    let original = cache.capture(&documents, &uri).unwrap();
    let reused = cache.capture(&documents, &uri).unwrap();
    assert!(Arc::ptr_eq(&original, &reused));
    assert_eq!(original.source().as_ptr(), reused.source().as_ptr());
    assert!(documents.apply_changes(&uri, vec![replace(original.source())], 8));
    let updated = cache.capture(&documents, &uri).unwrap();
    assert_ne!(original.revision(), updated.revision());
    assert_eq!(original.source(), updated.source());
    assert!(!Arc::ptr_eq(&original, &updated));
    assert_eq!(
        original.check_current(&documents),
        Err(SnapshotRefusal::Superseded)
    );
    assert_eq!(updated.version(), 8);
}

#[test]
fn actual_incremental_utf16_changes_refresh_snapshot_without_mutating_retained_source() {
    let documents = DocumentStore::new();
    let uri = uri("file:///incremental.ts");
    documents.open(
        uri.clone(),
        "a😀b\r\nx\u{2028}y\n".into(),
        1,
        "typescript".into(),
    );
    let cache = SourceSnapshotCache::default();
    let before = cache.capture(&documents, &uri).unwrap();
    let change = TextDocumentContentChangeEvent {
        range: Some(Range::new(Position::new(0, 1), Position::new(0, 3))),
        range_length: None,
        text: "α".into(),
    };
    assert!(documents.apply_changes(&uri, vec![change], 2));
    let after = cache.capture(&documents, &uri).unwrap();
    assert_eq!(before.source(), "a😀b\r\nx\u{2028}y\n");
    assert_eq!(after.source(), "aαb\r\nx\u{2028}y\n");
    assert_ne!(before.key(), after.key());
    assert_eq!(after.check_current(&documents), Ok(()));
}

#[test]
fn identical_editor_versions_and_empty_buffers_keep_distinct_real_host_keys() {
    let documents = DocumentStore::new();
    let first_uri = uri("file:///first.ts");
    let second_uri = uri("file:///second.ts");
    documents.open(first_uri.clone(), "".into(), 1, "typescript".into());
    documents.open(second_uri.clone(), "".into(), 1, "typescript".into());
    let cache = SourceSnapshotCache::default();
    let first = cache.capture(&documents, &first_uri).unwrap();
    let second = cache.capture(&documents, &second_uri).unwrap();
    assert_eq!(first.version(), second.version());
    assert_eq!(first.source(), second.source());
    assert_ne!(first.key(), second.key());
    assert_eq!(first.uri(), &first_uri);
    assert_eq!(second.uri(), &second_uri);
}

#[test]
fn close_reopen_and_rename_do_not_reuse_a_client_version_as_snapshot_authority() {
    let documents = DocumentStore::new();
    let old_uri = uri("file:///before.vue");
    let new_uri = uri("file:///after.vue");
    documents.open(old_uri.clone(), "<div/>".into(), 1, "vue".into());
    let cache = SourceSnapshotCache::default();
    let first = cache.capture(&documents, &old_uri).unwrap();
    documents.close(&old_uri);
    assert_eq!(
        first.check_current(&documents),
        Err(SnapshotRefusal::MissingDocument)
    );
    documents.open(old_uri.clone(), "<div/>".into(), 1, "vue".into());
    let reopened = cache.capture(&documents, &old_uri).unwrap();
    assert_eq!(
        first.check_current(&documents),
        Err(SnapshotRefusal::Superseded)
    );
    assert_ne!(first.key(), reopened.key());
    assert!(documents.rename(&old_uri, new_uri.clone()));
    assert_eq!(
        reopened.check_current(&documents),
        Err(SnapshotRefusal::MissingDocument)
    );
    let renamed = cache.capture(&documents, &new_uri).unwrap();
    assert_eq!(renamed.uri(), &new_uri);
    assert_eq!(renamed.source(), reopened.source());
    assert_ne!(renamed.revision(), reopened.revision());
}

#[test]
fn cancelling_before_first_poll_never_starts_computation() {
    let documents = DocumentStore::new();
    let uri = uri("file:///cancel.ts");
    documents.open(
        uri.clone(),
        "const value = 1;".into(),
        1,
        "typescript".into(),
    );
    let cache = SourceSnapshotCache::default();
    let (query, cancel) = cache.begin_query(&documents, &uri).unwrap();
    let started = AtomicBool::new(false);
    cancel.abort();
    let result = block_on(query.run(&documents, |_| async {
        started.store(true, Ordering::Relaxed);
    }));
    assert!(matches!(result, Err(SnapshotRefusal::Cancelled)));
    assert!(!started.load(Ordering::Relaxed));
}

#[derive(Default)]
struct WakeCounter(AtomicUsize);

impl ArcWake for WakeCounter {
    fn wake_by_ref(counter: &Arc<Self>) {
        counter.0.fetch_add(1, Ordering::Relaxed);
    }
}

struct DropCounter(Arc<AtomicUsize>);

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn genuine_abort_wakes_pending_work_and_drops_its_retained_computation() {
    let documents = DocumentStore::new();
    let uri = uri("file:///pending.ts");
    documents.open(uri.clone(), "let value;".into(), 1, "typescript".into());
    let cache = SourceSnapshotCache::default();
    let (query, cancel) = cache.begin_query(&documents, &uri).unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let marker = DropCounter(Arc::clone(&drops));
    let mut running = Box::pin(query.run(&documents, |_| async move {
        let _retained = marker;
        futures::future::pending::<()>().await;
    }));
    let wake = Arc::new(WakeCounter::default());
    let waker = waker_ref(&wake);
    let mut context = Context::from_waker(&waker);
    assert!(running.as_mut().poll(&mut context).is_pending());
    assert!(running.as_mut().poll(&mut context).is_pending());
    let before = wake.0.load(Ordering::Relaxed);
    cancel.abort();
    assert!(wake.0.load(Ordering::Relaxed) > before);
    assert!(matches!(
        running.as_mut().poll(&mut context),
        Poll::Ready(Err(SnapshotRefusal::Cancelled))
    ));
    drop(running);
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}

#[test]
fn a_real_store_change_while_query_is_suspended_has_no_live_shard_guard() {
    let documents = DocumentStore::new();
    let uri = uri("file:///suspended.ts");
    documents.open(uri.clone(), "before".into(), 1, "typescript".into());
    let cache = SourceSnapshotCache::default();
    let (query, _) = cache.begin_query(&documents, &uri).unwrap();
    let (sender, receiver) = futures::channel::oneshot::channel::<()>();
    let mut running = Box::pin(query.run(&documents, |_| async { receiver.await.unwrap() }));
    let wake = Arc::new(WakeCounter::default());
    let waker = waker_ref(&wake);
    let mut context = Context::from_waker(&waker);
    assert!(running.as_mut().poll(&mut context).is_pending());
    assert!(running.as_mut().poll(&mut context).is_pending());
    assert!(documents.apply_changes(&uri, vec![replace("after")], 2));
    sender.send(()).unwrap();
    assert!(matches!(
        running.as_mut().poll(&mut context),
        Poll::Ready(Err(SnapshotRefusal::Superseded))
    ));
}

#[test]
fn host_change_after_computation_prevents_publication_callback() {
    let documents = DocumentStore::new();
    let uri = uri("file:///publication.ts");
    documents.open(uri.clone(), "before".into(), 1, "typescript".into());
    let cache = SourceSnapshotCache::default();
    let (query, _) = cache.begin_query(&documents, &uri).unwrap();
    let ready = block_on(query.run(&documents, |_| async { 7 })).unwrap();
    assert!(documents.apply_changes(&uri, vec![replace("after")], 2));
    let published = AtomicBool::new(false);
    assert_eq!(
        ready.publish(&documents, |_| published.store(true, Ordering::Relaxed)),
        Err(SnapshotRefusal::Superseded)
    );
    assert!(!published.load(Ordering::Relaxed));
}

#[test]
fn cancellation_after_computation_prevents_publication_callback() {
    let documents = DocumentStore::new();
    let uri = uri("file:///completed.ts");
    documents.open(uri.clone(), "original".into(), 1, "typescript".into());
    let cache = SourceSnapshotCache::default();
    let (query, cancel) = cache.begin_query(&documents, &uri).unwrap();
    let ready = block_on(query.run(&documents, |_| async { 7 })).unwrap();
    cancel.abort();
    let published = AtomicBool::new(false);
    assert_eq!(
        ready.publish(&documents, |_| published.store(true, Ordering::Relaxed)),
        Err(SnapshotRefusal::Cancelled)
    );
    assert!(!published.load(Ordering::Relaxed));
}

#[test]
fn current_result_publishes_host_source_but_cannot_mint_native_file() {
    let documents = DocumentStore::new();
    let uri = uri("file:///current.ts");
    documents.open(uri.clone(), "actual source".into(), 4, "typescript".into());
    let cache = SourceSnapshotCache::default();
    let (query, _) = cache.begin_query(&documents, &uri).unwrap();
    let snapshot = Arc::clone(query.snapshot());
    let ready = block_on(query.run(&documents, |source| async move {
        (source.revision(), source.version())
    }))
    .unwrap();
    assert_eq!(
        ready.publish(&documents, |value| value),
        Ok((snapshot.revision(), 4))
    );
    assert_eq!(snapshot.source(), "actual source");
    assert_eq!(
        snapshot.native_file(&documents),
        Err(SnapshotRefusal::NativeFileUnavailable)
    );
}
