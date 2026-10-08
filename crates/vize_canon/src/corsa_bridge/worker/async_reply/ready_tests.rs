//! Semantic FIFO waits share their original deadline and cancellation lease.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;

use futures::{FutureExt, executor::block_on};

use super::super::{BoundedWorker, WorkerError};

const SETTLE: Duration = Duration::from_secs(10);

fn abandoned_owner() -> (BoundedWorker<u32>, mpsc::Sender<()>) {
    let worker = BoundedWorker::new("vize-test-ready-worker", 0);
    let (release, held) = mpsc::channel();
    let (entered, observed) = mpsc::channel();
    let mut first = Box::pin(worker.submit_async(SETTLE, move |state| {
        entered.send(()).unwrap();
        held.recv().unwrap();
        *state = 7;
    }));
    assert_eq!(first.as_mut().now_or_never(), None);
    observed.recv_timeout(SETTLE).unwrap();
    drop(first);
    assert!(worker.is_draining());
    (worker, release)
}

#[test]
fn semantic_reply_queues_once_and_reads_the_drained_owners_current_state() {
    let (worker, release) = abandoned_owner();
    let ran = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&ran);
    let mut next = Box::pin(worker.submit_ready_async(SETTLE, move |state| {
        counted.fetch_add(1, Ordering::SeqCst);
        *state += 2;
        *state
    }));
    assert_eq!(next.as_mut().now_or_never(), None);
    assert_eq!(ran.load(Ordering::SeqCst), 0);
    assert_eq!(block_on(async { "syntax-only" }), "syntax-only");
    // Compatibility/shutdown callers still refuse an outstanding stall.
    assert_eq!(worker.submit(SETTLE, |_| ()), Err(WorkerError::TimedOut));
    release.send(()).unwrap();
    assert_eq!(block_on(next), Ok(9));
    assert_eq!(ran.load(Ordering::SeqCst), 1);
    assert!(!worker.is_draining());
}

#[test]
fn semantic_deadline_expires_before_drain_and_never_enters_native_work() {
    let (worker, release) = abandoned_owner();
    let ran = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&ran);
    assert_eq!(
        block_on(
            worker.submit_ready_async(Duration::from_millis(50), move |_| {
                counted.fetch_add(1, Ordering::SeqCst);
            })
        ),
        Err(WorkerError::TimedOut)
    );
    assert_eq!(worker.abandoned.load(Ordering::Acquire), 2);
    release.send(()).unwrap();
    assert_eq!(
        block_on(worker.submit_ready_async(SETTLE, |state| *state)),
        Ok(7)
    );
    assert_eq!(ran.load(Ordering::SeqCst), 0);
    assert!(!worker.is_draining());
}

#[test]
fn cancelled_semantic_waiter_is_skipped_and_the_following_call_remains_usable() {
    let (worker, release) = abandoned_owner();
    let ran = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&ran);
    let mut cancelled = Box::pin(worker.submit_ready_async(SETTLE, move |_| {
        counted.fetch_add(1, Ordering::SeqCst);
    }));
    assert_eq!(cancelled.as_mut().now_or_never(), None);
    drop(cancelled);
    assert_eq!(worker.abandoned.load(Ordering::Acquire), 2);
    release.send(()).unwrap();
    assert_eq!(
        block_on(worker.submit_ready_async(SETTLE, |state| *state)),
        Ok(7)
    );
    assert_eq!(ran.load(Ordering::SeqCst), 0);
    assert!(!worker.is_draining());
}
