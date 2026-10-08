use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use futures::FutureExt;
use futures::executor::block_on;

use super::super::{BoundedWorker, WorkerError};

const SETTLE: Duration = Duration::from_secs(10);

fn wait_until_drained(worker: &BoundedWorker<()>) {
    let started = Instant::now();
    while worker.abandoned.load(Ordering::Acquire) != 0 {
        assert!(started.elapsed() < SETTLE, "abandoned work did not drain");
        std::thread::yield_now();
    }
}

#[test]
fn backend_wait_yields_and_an_unrelated_future_finishes_before_release() {
    let worker = BoundedWorker::new("vize-test-async-worker", ());
    let (release, held) = mpsc::channel();
    let (entered, observed) = mpsc::channel();
    let mut reply = Box::pin(worker.submit_async(SETTLE, move |_| {
        entered.send(()).unwrap();
        held.recv().unwrap();
        7
    }));
    assert_eq!(reply.as_mut().now_or_never(), None);
    observed.recv_timeout(SETTLE).unwrap();
    assert_eq!(block_on(async { "syntax-only" }), "syntax-only");
    release.send(()).unwrap();
    assert_eq!(block_on(reply), Ok(7));
}

#[test]
fn dropping_an_entered_reply_retires_publication_until_ipc_drains() {
    let worker = BoundedWorker::new("vize-test-async-worker", ());
    let (release, held) = mpsc::channel();
    let (entered, observed) = mpsc::channel();
    let mut reply = Box::pin(worker.submit_async(SETTLE, move |_| {
        entered.send(()).unwrap();
        held.recv().unwrap();
        7
    }));
    assert_eq!(reply.as_mut().now_or_never(), None);
    observed.recv_timeout(SETTLE).unwrap();
    drop(reply);
    assert_eq!(worker.abandoned.load(Ordering::Acquire), 1);
    assert_eq!(
        block_on(worker.submit_async(SETTLE, |_| 9)),
        Err(WorkerError::TimedOut)
    );
    release.send(()).unwrap();
    wait_until_drained(&worker);
    assert_eq!(block_on(worker.submit_async(SETTLE, |_| 9)), Ok(9));
}

#[test]
fn cancellation_skips_queued_work_and_keeps_completed_state_intact() {
    let worker = BoundedWorker::new("vize-test-async-worker", ());
    let (release, held) = mpsc::channel();
    let (entered, observed) = mpsc::channel();
    let ran = Arc::new(AtomicUsize::new(0));
    let mut first = Box::pin(worker.submit_async(SETTLE, move |_| {
        entered.send(()).unwrap();
        held.recv().unwrap();
        7
    }));
    assert_eq!(first.as_mut().now_or_never(), None);
    observed.recv_timeout(SETTLE).unwrap();
    let observed_runs = Arc::clone(&ran);
    let mut queued = Box::pin(worker.submit_async(SETTLE, move |_| {
        observed_runs.fetch_add(1, Ordering::SeqCst);
    }));
    assert_eq!(queued.as_mut().now_or_never(), None);
    drop(queued);
    release.send(()).unwrap();
    assert_eq!(block_on(first), Ok(7));
    wait_until_drained(&worker);
    assert_eq!(ran.load(Ordering::SeqCst), 0);
    assert_eq!(worker.submit(SETTLE, |_| 11), Ok(11));
}

#[test]
fn each_queued_deadline_fires_while_the_worker_is_still_held() {
    let worker = BoundedWorker::new("vize-test-async-worker", ());
    let (release, held) = mpsc::channel();
    let (entered, observed) = mpsc::channel();
    let mut first = Box::pin(worker.submit_async(SETTLE, move |_| {
        entered.send(()).unwrap();
        held.recv().unwrap();
        7
    }));
    assert_eq!(first.as_mut().now_or_never(), None);
    observed.recv_timeout(SETTLE).unwrap();
    assert_eq!(
        block_on(worker.submit_async(Duration::from_millis(50), |_| 8)),
        Err(WorkerError::TimedOut),
    );
    assert_eq!(worker.abandoned.load(Ordering::Acquire), 1);
    release.send(()).unwrap();
    assert_eq!(block_on(first), Ok(7));
    wait_until_drained(&worker);
    assert_eq!(block_on(worker.submit_async(SETTLE, |_| 9)), Ok(9));
}

#[test]
fn dropping_an_unpolled_call_enqueues_nothing() {
    let worker = BoundedWorker::new("vize-test-async-worker", 0);
    drop(worker.submit_async(SETTLE, |state| *state += 1));
    assert_eq!(worker.submit(SETTLE, |state| *state), Ok(0));
}

#[test]
fn an_unread_completed_reply_does_not_poison_later_calls() {
    let worker = BoundedWorker::new("vize-test-async-worker", 0);
    let mut reply = Box::pin(worker.submit_async(SETTLE, |state| *state += 1));
    let _ = reply.as_mut().now_or_never();
    // The synchronous barrier proves the preceding job and its completion
    // lease finished; deliberately discard the unread async reply afterwards.
    assert_eq!(worker.submit(SETTLE, |state| *state), Ok(1));
    drop(reply);
    assert_eq!(worker.abandoned.load(Ordering::Acquire), 0);
    assert_eq!(block_on(worker.submit_async(SETTLE, |state| *state)), Ok(1));
}
