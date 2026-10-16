//! Final acknowledgment must wait for entered startup even after its caller left.
use crate::corsa_bridge::{CorsaBridge, CorsaBridgeConfig};
use futures::FutureExt;
use std::{
    sync::{Arc, atomic::Ordering, mpsc},
    time::Duration,
};

fn bridge(timeout_ms: u64) -> CorsaBridge {
    CorsaBridge::with_config(CorsaBridgeConfig {
        timeout_ms,
        ..Default::default()
    })
}

#[test]
fn exit_shutdown_waits_for_unfinished_abandoned_startup_before_acknowledging() {
    let bridge = bridge(1000);
    let (release, held) = mpsc::channel();
    let (entered, started) = mpsc::channel();
    let initialized = Arc::clone(&bridge.initialized);
    let mut startup = Box::pin(
        bridge
            .worker
            .submit_async(Duration::from_secs(5), move |_| {
                entered.send(()).unwrap();
                held.recv().unwrap();
                // This is the actual spawn completion flag written on the owner lane.
                initialized.store(true, Ordering::SeqCst);
            }),
    );
    assert!(startup.as_mut().now_or_never().is_none());
    started.recv_timeout(Duration::from_secs(1)).unwrap();
    drop(startup);
    assert!(!bridge.is_initialized());
    assert!(bridge.is_draining());
    let mut shutdown = Box::pin(bridge.shutdown_for_exit());
    assert!(
        shutdown.as_mut().now_or_never().is_none(),
        "not initialized does not mean no pending owner"
    );
    release.send(()).unwrap();
    assert!(futures::executor::block_on(shutdown).is_ok());
    assert!(
        !bridge.is_initialized(),
        "cleanup entered after startup completed"
    );
    assert!(!bridge.is_draining());
}

#[test]
fn exit_shutdown_deadline_is_an_error_until_queued_cleanup_really_completes() {
    let bridge = bridge(100);
    let (release, held) = mpsc::channel();
    let (entered, started) = mpsc::channel();
    let mut startup = Box::pin(
        bridge
            .worker
            .submit_async(Duration::from_secs(5), move |_| {
                entered.send(()).unwrap();
                held.recv().unwrap();
            }),
    );
    assert!(startup.as_mut().now_or_never().is_none());
    started.recv_timeout(Duration::from_secs(1)).unwrap();
    drop(startup);
    assert!(matches!(
        futures::executor::block_on(bridge.shutdown_for_exit()),
        Err(crate::corsa_bridge::CorsaBridgeError::Timeout)
    ));
    release.send(()).unwrap();
    // A later acknowledged cleanup still traverses the owner queue.
    assert!(futures::executor::block_on(bridge.shutdown_for_exit()).is_ok());
    assert!(!bridge.is_draining());
}

#[test]
fn exit_shutdown_closes_admission_for_already_queued_and_future_startups() {
    let bridge = bridge(1000);
    let (release, held) = mpsc::channel();
    let (entered, started) = mpsc::channel();
    let mut occupied = Box::pin(
        bridge
            .worker
            .submit_async(Duration::from_secs(5), move |_| {
                entered.send(()).unwrap();
                held.recv().unwrap();
            }),
    );
    assert!(occupied.as_mut().now_or_never().is_none());
    started.recv_timeout(Duration::from_secs(1)).unwrap();
    let mut startup = Box::pin(bridge.spawn());
    assert!(startup.as_mut().now_or_never().is_none());
    let mut shutdown = Box::pin(bridge.shutdown_for_exit());
    assert!(shutdown.as_mut().now_or_never().is_none());
    release.send(()).unwrap();
    assert!(futures::executor::block_on(occupied).is_ok());
    assert!(matches!(
        futures::executor::block_on(startup),
        Err(crate::corsa_bridge::CorsaBridgeError::ProcessTerminated)
    ));
    assert!(futures::executor::block_on(shutdown).is_ok());
    assert!(matches!(
        futures::executor::block_on(bridge.spawn()),
        Err(crate::corsa_bridge::CorsaBridgeError::ProcessTerminated)
    ));
}
