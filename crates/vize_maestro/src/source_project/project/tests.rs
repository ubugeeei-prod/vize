#![expect(
    clippy::disallowed_types,
    reason = "actual host lifecycle tests retain standard Arc wake/drop counters"
)]

mod lifecycle;
#[cfg(feature = "experimental-source-navigation")]
mod modules;
mod shared;

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};

use futures::Future;
use futures::task::{ArcWake, waker_ref};
use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};

use crate::document::DocumentStore;
use crate::runtime::block_on;
use crate::source_project::SnapshotRefusal;

use super::{ProjectQueryResult, SourceQueryProject};

fn uri(name: &str) -> Url {
    Url::parse(name).expect("real host URI")
}

fn replace(source: &str) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: source.into(),
    }
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

fn pending<'host>(
    project: &SourceQueryProject<'host>,
    uri: &Url,
    drops: &Arc<AtomicUsize>,
) -> impl Future<Output = Result<ProjectQueryResult<'host, ()>, SnapshotRefusal>> + 'host + use<'host>
{
    let (query, _) = project
        .begin_query(uri)
        .expect("actual project host snapshot");
    let marker = DropCounter(Arc::clone(drops));
    query.run(|_| async move {
        let _retained = marker;
        futures::future::pending::<()>().await;
    })
}

fn poll<F: Future>(future: Pin<&mut F>, wake: &Arc<WakeCounter>) -> Poll<F::Output> {
    let waker = waker_ref(wake);
    future.poll(&mut Context::from_waker(&waker))
}

#[test]
fn accepted_incremental_host_change_wakes_and_drops_pending_original_query() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = uri("file:///project.ts");
    project.open(uri.clone(), "a😀b\nvalue".into(), 1, "typescript".into());
    let drops = Arc::new(AtomicUsize::new(0));
    let wake = Arc::new(WakeCounter::default());
    let mut work = Box::pin(pending(&project, &uri, &drops));
    assert!(poll(work.as_mut(), &wake).is_pending());
    assert!(poll(work.as_mut(), &wake).is_pending());
    let before = wake.0.load(Ordering::Relaxed);
    let change = TextDocumentContentChangeEvent {
        range: Some(Range::new(Position::new(0, 1), Position::new(0, 3))),
        range_length: None,
        text: "α".into(),
    };
    assert!(project.apply_changes(&uri, vec![change], 2));
    assert!(wake.0.load(Ordering::Relaxed) > before);
    assert!(matches!(
        poll(work.as_mut(), &wake),
        Poll::Ready(Err(SnapshotRefusal::Cancelled))
    ));
    assert_eq!(documents.text(&uri).as_deref(), Some("aαb\nvalue"));
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}

#[test]
fn rejected_old_version_keeps_pending_work_and_original_snapshot() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = uri("file:///ordered.ts");
    project.open(uri.clone(), "original".into(), 3, "typescript".into());
    let (query, cancel) = project.begin_query(&uri).unwrap();
    let original = Arc::clone(query.snapshot());
    let drops = Arc::new(AtomicUsize::new(0));
    let wake = Arc::new(WakeCounter::default());
    let mut work = Box::pin(pending(&project, &uri, &drops));
    assert!(poll(work.as_mut(), &wake).is_pending());
    assert!(poll(work.as_mut(), &wake).is_pending());
    assert!(!project.apply_changes(&uri, vec![replace("stale")], 2));
    assert!(!cancel.is_aborted());
    assert!(poll(work.as_mut(), &wake).is_pending());
    let (same, _) = project.begin_query(&uri).unwrap();
    assert!(Arc::ptr_eq(&original, same.snapshot()));
    assert_eq!(documents.text(&uri).as_deref(), Some("original"));
    assert_eq!(drops.load(Ordering::Relaxed), 0);
    drop(work);
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}

#[test]
fn dropping_suspended_query_releases_lease_and_computation() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = uri("file:///drop.ts");
    project.open(uri.clone(), "value".into(), 1, "typescript".into());
    let drops = Arc::new(AtomicUsize::new(0));
    let wake = Arc::new(WakeCounter::default());
    let mut work = Box::pin(pending(&project, &uri, &drops));
    assert!(poll(work.as_mut(), &wake).is_pending());
    assert!(poll(work.as_mut(), &wake).is_pending());
    assert_eq!(project.active.0.lock().len(), 1);
    drop(work);
    assert_eq!(drops.load(Ordering::Relaxed), 1);
    assert!(project.active.0.lock().is_empty());
}

#[test]
fn completed_query_remains_guarded_until_publication_after_real_change() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = uri("file:///ready.ts");
    project.open(uri.clone(), "before".into(), 1, "typescript".into());
    let (query, cancel) = project.begin_query(&uri).unwrap();
    let result = block_on(query.run(|_| async { 7 })).unwrap();
    assert!(project.apply_changes(&uri, vec![replace("after")], 2));
    assert!(cancel.is_aborted());
    let mut published = false;
    assert_eq!(
        result.publish(|_| {
            published = true;
        }),
        Err(SnapshotRefusal::Superseded)
    );
    assert!(!published);
    assert!(project.active.0.lock().is_empty());
}
