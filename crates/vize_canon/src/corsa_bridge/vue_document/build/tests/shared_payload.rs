//! Exercise real query preparation and the original full ordered Vec oracle.
use super::*;
use futures::FutureExt;
use std::sync::{Arc, Weak, mpsc};
use std::time::Duration;

const BOUND: Duration = Duration::from_secs(10);

fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let a = root.path().join("A.vue");
    let b = root.path().join("B.vue");
    std::fs::write(&a, HOST).unwrap();
    std::fs::write(&b, CHILD).unwrap();
    std::fs::write(root.path().join("Child.vue"), CHILD).unwrap();
    std::fs::write(root.path().join("tsconfig.json"), "{}").unwrap();
    (root, a, b)
}

#[test]
fn actual_repeated_preparation_shares_base_bytes_and_keeps_the_complete_native_vec() {
    let (_root, a, b) = fixture();
    let session = EditorMirrorSession::new();
    let overlays = [(a.clone(), HOST), (b.clone(), CHILD)];
    let first = prepare(&session, &a, HOST, &overlays, &[]);
    let second = prepare(&session, &a, HOST, &overlays, &[]);
    assert_eq!(first.documents, second.documents);
    assert!(!first.documents.is_empty());
    let bytes = first
        .documents
        .iter()
        .map(|(_, code)| code.len())
        .sum::<usize>();
    assert!(
        bytes > HOST.len(),
        "the real generated payload, not a short mock"
    );
    for (before, after) in first.documents.iter().zip(second.documents.iter()) {
        assert!(
            std::ptr::eq(before, after),
            "old surface.documents.clone copied every tuple"
        );
        assert_eq!(before, after);
    }
    // A sibling expands native membership without changing this host's closure.
    let _sibling = prepare(&session, &b, CHILD, &overlays, &[]);
    let current = prepare(&session, &a, HOST, &overlays, &[]);
    let mut old_vec = current.documents.to_vec();
    let mut shared = current.documents.clone();
    let root = current.session_project_root.as_deref().unwrap();
    current
        .source_catalog
        .extend_native_project_documents(root, &mut old_vec);
    current
        .source_catalog
        .extend_native_project_document_batch(root, &mut shared);
    assert_eq!(
        shared.to_vec(),
        old_vec,
        "whole ordered payload and live-member precedence"
    );
    assert_eq!(
        shared.iter().map(|(_, code)| code.len()).sum::<usize>(),
        old_vec.iter().map(|(_, code)| code.len()).sum::<usize>()
    );
    for (before, after) in current.documents.iter().zip(shared.iter()) {
        assert!(
            std::ptr::eq(before, after),
            "appending cannot copy the immutable base"
        );
    }
}

#[test]
fn duplicates_order_and_changed_revision_never_rewrite_an_inflight_payload() {
    let mut batch = CorsaNativeDocumentBatch::from(vec![
        ("same".into(), "first 雪\r\n".into()),
        ("same".into(), "second 🌸".into()),
    ]);
    let mut old = batch.to_vec();
    batch.push(("same".into(), "last".into()));
    old.push(("same".into(), "last".into()));
    assert_eq!(
        batch.to_vec(),
        old,
        "duplicate order is not normalized or deduplicated"
    );
    let (_root, a, b) = fixture();
    let session = EditorMirrorSession::new();
    let first = prepare(
        &session,
        &a,
        HOST,
        &[(a.clone(), HOST), (b.clone(), CHILD)],
        &[],
    );
    let old = first.documents.to_vec();
    let held = first.documents.clone();
    let changed = HOST.replace("日本語 🦀", "changed 🦀");
    let later = prepare(
        &session,
        &a,
        &changed,
        &[(a.clone(), changed.as_str()), (b, CHILD)],
        &[],
    );
    assert_ne!(later.documents, old);
    assert_eq!(held.to_vec(), old);
    for (before, after) in held.iter().zip(later.documents.iter()) {
        if before.1 != after.1 {
            assert!(!std::ptr::eq(before, after));
        }
    }
}

fn detached_payload() -> (
    CorsaNativeDocumentBatch,
    Weak<crate::corsa_bridge::CorsaMaterializedSource>,
) {
    let (_root, a, b) = fixture();
    let session = EditorMirrorSession::new();
    let project = prepare(&session, &a, HOST, &[(a.clone(), HOST), (b, CHILD)], &[]);
    let member = project.host.materialized_sources.first().unwrap();
    let leaf = project
        .source_catalog
        .get(&member.materialized_path)
        .unwrap();
    (project.documents.clone(), Arc::downgrade(&leaf))
}

struct EnteredPayload {
    value: Option<CorsaNativeDocumentBatch>,
    dropped: mpsc::Sender<()>,
}
impl Drop for EnteredPayload {
    fn drop(&mut self) {
        drop(self.value.take());
        self.dropped.send(()).unwrap();
    }
}

#[test]
fn dropped_query_and_failed_worker_keep_only_selected_bytes_until_the_same_job_finishes() {
    use crate::corsa_bridge::worker::BoundedWorker;
    for cancelled in [false, true] {
        for fail in [false, true] {
            let (batch, map) = detached_payload();
            assert!(
                map.upgrade().is_none(),
                "payload cannot keep a catalog/mapping owner alive"
            );
            let weak = batch.payload_owner_for_test();
            let expected = batch.to_vec();
            let (dropped, observed) = mpsc::channel();
            let held = EnteredPayload {
                value: Some(batch),
                dropped,
            };
            let (entered, started) = mpsc::channel();
            let (release, permit) = mpsc::channel();
            let worker = BoundedWorker::new("vize-sync-payload-test", ());
            let mut job = Box::pin(worker.submit_async(BOUND, move |_| {
                entered.send(()).unwrap();
                permit.recv_timeout(BOUND).unwrap();
                assert_eq!(held.value.as_ref().unwrap().to_vec(), expected);
                if fail {
                    Err("original sync error")
                } else {
                    Ok(())
                }
            }));
            assert!(job.as_mut().now_or_never().is_none());
            started.recv_timeout(BOUND).unwrap();
            assert!(weak.upgrade().is_some());
            if cancelled {
                drop(job);
                drop(worker);
                assert!(
                    weak.upgrade().is_some(),
                    "entered command retains its immutable bytes"
                );
                release.send(()).unwrap();
            } else {
                release.send(()).unwrap();
                assert_eq!(
                    futures::executor::block_on(job),
                    Ok(if fail {
                        Err("original sync error")
                    } else {
                        Ok(())
                    })
                );
                drop(worker);
            }
            observed.recv_timeout(BOUND).unwrap();
            assert!(
                weak.upgrade().is_none(),
                "completed or failed command releases its payload"
            );
        }
    }
}
