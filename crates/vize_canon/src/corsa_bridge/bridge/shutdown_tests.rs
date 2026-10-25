//! FIFO shutdown observes accepted startup; marker jobs grant no SDK credit.
#![expect(clippy::disallowed_types, reason = "worker ownership unit controls")]

use super::{CorsaBridge, CorsaBridgeConfig, CorsaBridgeError};
use futures::{FutureExt, executor::block_on};
use std::{sync::mpsc, time::Duration};

const SETTLE: Duration = Duration::from_secs(10);

fn abandoned_startup(bridge: &CorsaBridge) -> mpsc::Sender<()> {
    let (release, held) = mpsc::channel();
    let (entered, observed) = mpsc::channel();
    let mut startup = Box::pin(bridge.worker.submit_async(SETTLE, move |_| {
        entered.send(()).unwrap();
        held.recv().unwrap();
    }));
    assert!(startup.as_mut().now_or_never().is_none());
    observed.recv_timeout(SETTLE).unwrap();
    drop(startup);
    assert!(!bridge.is_initialized());
    assert!(bridge.is_draining());
    release
}

#[test]
fn shutdown_queues_behind_an_uninitialized_accepted_startup() {
    let bridge = CorsaBridge::new();
    let release = abandoned_startup(&bridge);
    let mut shutdown = Box::pin(bridge.shutdown());
    assert!(shutdown.as_mut().now_or_never().is_none());
    release.send(()).unwrap();
    assert!(block_on(shutdown).is_ok());
    assert!(!bridge.is_draining());
    assert!(!bridge.is_initialized());
}

#[test]
fn pending_startup_timeout_is_not_success_and_can_be_retried_after_drain() {
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        timeout_ms: 50,
        ..Default::default()
    });
    let release = abandoned_startup(&bridge);
    assert!(matches!(
        block_on(bridge.shutdown()),
        Err(CorsaBridgeError::Timeout)
    ));
    assert!(bridge.is_draining());
    release.send(()).unwrap();
    assert!(
        block_on(
            bridge
                .worker
                .submit_ready_async(SETTLE, |slot| slot.is_none())
        )
        .unwrap()
    );
    assert!(block_on(bridge.shutdown()).is_ok());
    assert!(!bridge.is_draining());
}

#[test]
fn empty_repeated_shutdown_never_spawns_a_client() {
    let bridge = CorsaBridge::new();
    assert!(block_on(bridge.shutdown()).is_ok());
    assert!(block_on(bridge.shutdown()).is_ok());
    assert!(
        block_on(
            bridge
                .worker
                .submit_ready_async(SETTLE, |slot| slot.is_none())
        )
        .unwrap()
    );
    assert!(!bridge.is_initialized());
}

#[test]
fn a_stopped_worker_is_not_a_successful_shutdown() {
    let bridge = CorsaBridge::new();
    assert!(
        block_on(
            bridge
                .worker
                .submit_async(SETTLE, |_| -> () { panic!("controlled worker panic") })
        )
        .is_err()
    );
    assert!(matches!(
        block_on(bridge.shutdown()),
        Err(CorsaBridgeError::ProcessTerminated)
    ));
}
