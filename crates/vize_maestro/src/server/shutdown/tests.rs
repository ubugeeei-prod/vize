//! Concurrent protocol waiters share a completion, including its failure.
use super::Shutdown;
use futures::{FutureExt, channel::oneshot, executor::block_on};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::{io, time::Duration};

#[test]
fn repeated_shutdown_reuses_the_complete_failure_without_running_again() {
    let shutdown = Shutdown::default();
    let calls = Arc::new(AtomicUsize::new(0));
    let counted = calls.clone();
    let expected = tower_lsp::jsonrpc::Error::internal_error();
    let result = block_on(shutdown.run(async move {
        counted.fetch_add(1, Ordering::SeqCst);
        Err(tower_lsp::jsonrpc::Error::internal_error())
    }));
    assert_eq!(result, Err(expected.clone()));
    assert_eq!(block_on(shutdown.run(async { Ok(()) })), Err(expected));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn concurrent_waiters_do_not_acknowledge_an_unfinished_owner() {
    let shutdown = Shutdown::default();
    let (release, held) = oneshot::channel();
    let mut first = Box::pin(shutdown.run(async move {
        held.await.unwrap();
        Ok(())
    }));
    assert!(first.as_mut().now_or_never().is_none());
    let mut second = Box::pin(shutdown.run(async { panic!("duplicate shutdown work") }));
    assert!(second.as_mut().now_or_never().is_none());
    release.send(()).unwrap();
    assert_eq!(
        block_on(futures::future::join(first, second)),
        (Ok(()), Ok(()))
    );
}

#[test]
fn cancellation_before_bridge_enqueue_does_not_stop_the_independent_driver() {
    let shutdown = Shutdown::default();
    assert!(
        shutdown.0.lock().is_none(),
        "startup must not start a driver"
    );
    let admission = Arc::new(futures::lock::Mutex::new(()));
    let held = block_on(admission.lock());
    let driver_admission = admission.clone();
    let (entered, observed) = std::sync::mpsc::channel();
    let (finished, completed) = std::sync::mpsc::channel();
    let owner = Arc::new(());
    let weak = Arc::downgrade(&owner);
    let mut first = Box::pin(shutdown.run(async move {
        entered.send(()).unwrap();
        let _accepted = driver_admission.lock().await;
        drop(owner);
        finished.send(()).unwrap();
        Ok(())
    }));
    assert!(first.as_mut().now_or_never().is_none());
    observed.recv_timeout(Duration::from_secs(10)).unwrap();
    drop(first); // No request waiter is left to poll the drain.
    assert!(weak.upgrade().is_some());
    drop(held);
    completed.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(weak.upgrade().is_none());
    assert_eq!(
        block_on(shutdown.run(async { panic!("must not restart") })),
        Ok(())
    );
}

#[test]
fn spawn_failure_is_explicit_and_repeated_without_a_retry_or_false_ack() {
    let shutdown = Shutdown::default();
    let result = block_on(
        shutdown.run_with(async { panic!("failed spawn cannot run work") }, |_, _| {
            Err(io::Error::other("controlled thread spawn failure"))
        }),
    );
    assert_eq!(result, Err(tower_lsp::jsonrpc::Error::internal_error()));
    assert_eq!(block_on(shutdown.run(async { Ok(()) })), result);
}

#[test]
fn a_driver_panic_is_an_explicit_shared_failure() {
    let shutdown = Shutdown::default();
    let result = block_on(shutdown.run(async { panic!("controlled driver panic") }));
    assert_eq!(result, Err(tower_lsp::jsonrpc::Error::internal_error()));
    assert_eq!(block_on(shutdown.run(async { Ok(()) })), result);
}
