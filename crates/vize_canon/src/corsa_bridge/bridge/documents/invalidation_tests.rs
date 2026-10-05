use std::sync::{atomic::Ordering, mpsc};
use std::time::{Duration, Instant};

use futures::{FutureExt, executor::block_on};

use super::super::CorsaBridge;

#[test]
fn cancelling_a_real_queued_invalidation_preserves_the_dirty_disk_state() {
    let bridge = CorsaBridge::new();
    bridge.initialized.store(true, Ordering::SeqCst);
    let (release, held) = mpsc::channel();
    let (entered, observed) = mpsc::channel();
    let settle = Duration::from_secs(10);
    let mut running = Box::pin(bridge.worker.submit_async(settle, move |_| {
        entered.send(()).unwrap();
        held.recv().unwrap();
    }));
    assert_eq!(running.as_mut().now_or_never(), None);
    observed.recv_timeout(settle).unwrap();
    bridge.mark_disk_project_state_dirty();
    let mut flush = Box::pin(bridge.flush_disk_project_state_if_dirty());
    assert!(flush.as_mut().now_or_never().is_none());
    assert!(!bridge.disk_project_state_dirty.load(Ordering::SeqCst));
    drop(flush);
    assert!(bridge.disk_project_state_dirty.load(Ordering::SeqCst));
    release.send(()).unwrap();
    assert_eq!(block_on(running), Ok(()));
    let started = Instant::now();
    while bridge.worker.submit(settle, |_| ()).is_err() {
        assert!(
            started.elapsed() < settle,
            "cancelled invalidation did not drain"
        );
        std::thread::yield_now();
    }
    // No client was installed in this fixture: a failed actual flush must
    // retain the pending invalidation rather than consuming its bit.
    assert!(block_on(bridge.flush_disk_project_state_if_dirty()).is_err());
    assert!(bridge.disk_project_state_dirty.load(Ordering::SeqCst));
}
