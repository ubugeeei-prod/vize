//! The executor must retain a wake even when a polled dependency parks the thread.

use std::{sync::mpsc, task::Poll, thread, time::Duration};

#[test]
fn a_dependency_cannot_consume_the_executors_wake_notification() {
    let (tx, rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        let mut polls = 0;
        let value = super::block_on(futures::future::poll_fn(|cx| {
            polls += 1;
            if polls == 1 {
                cx.waker().wake_by_ref();
                // Blocking libraries and nested executors share this thread's
                // park token. Consuming it cannot cancel the task's wake.
                thread::park_timeout(Duration::ZERO);
                Poll::Pending
            } else {
                Poll::Ready(polls)
            }
        }));
        let _ = tx.send(value);
    });
    let result = rx.recv_timeout(Duration::from_secs(2));
    // Release a broken executor before reporting failure, so the regression
    // fails promptly without leaking a parked thread into the rest of the suite.
    worker.thread().unpark();
    worker.join().unwrap();
    assert_eq!(result, Ok(2));
}
