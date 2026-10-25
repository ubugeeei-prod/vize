//! A real held startup is retained until the graceful caller owns its barrier.
//! The shell process is only a handshake stall, never SDK capability evidence.
use super::{Arc, BackendCleanup, await_condition, held_backend};
use futures::{FutureExt, executor::block_on};
use vize_canon::CorsaBridgeError;

#[test]
fn global_shutdown_preserves_a_startup_slot_after_the_original_waiter_finishes() {
    let (root, state) = held_backend(60_000);
    let state = Arc::new(state);
    let cleanup = BackendCleanup(root.path());
    let mut startup = Box::pin(state.get_corsa_bridge());
    assert!(startup.as_mut().now_or_never().is_none());
    await_condition(|| root.path().join("backend.pids").exists());
    let accepted = state.corsa_bridge.read().clone().unwrap();
    let mut shutdown = Box::pin(state.shutdown_project_contexts());
    assert!(shutdown.as_mut().now_or_never().is_none());
    assert!(state.project_routing_shutting_down());
    assert!(state.owns_corsa_bridge(&accepted));

    cleanup.stop();
    assert!(block_on(startup).is_none());
    // The post-spawn global-shutdown branch must not clear this slot before
    // the shutdown future can acquire the init gate and take ownership.
    assert!(state.owns_corsa_bridge(&accepted));
    assert!(block_on(shutdown).is_ok());
    assert!(state.corsa_bridge.read().is_none());
    assert!(!accepted.is_initialized());
    assert!(!accepted.is_draining());
}

#[test]
fn canceled_pending_startup_never_reports_a_false_successful_drain() {
    let (root, state) = held_backend(1_000);
    let state = Arc::new(state);
    let cleanup = BackendCleanup(root.path());
    let mut startup = Box::pin(state.get_corsa_bridge());
    assert!(startup.as_mut().now_or_never().is_none());
    await_condition(|| root.path().join("backend.pids").exists());
    let accepted = state.corsa_bridge.read().clone().unwrap();
    let mut shutdown = Box::pin(state.shutdown_project_contexts());
    assert!(shutdown.as_mut().now_or_never().is_none());
    drop(startup);
    assert!(accepted.is_draining());
    assert!(matches!(block_on(shutdown), Err(CorsaBridgeError::Timeout)));
    assert!(accepted.is_draining());
    cleanup.stop();
    await_condition(|| !accepted.is_draining());
}
