//! Ownership controls do not spawn a native executable.
#![expect(
    clippy::disallowed_types,
    reason = "worker control uses owned channels"
)]

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::task::{Context, Poll};
use std::time::Duration;

use futures::task::noop_waker;

use super::{CorsaBridge, CorsaBridgeError, EditorMirrorSession, StartupState};
use crate::corsa_bridge::worker::BoundedWorker;

const SETTLE: Duration = Duration::from_secs(10);

fn marker(session: &EditorMirrorSession) -> std::path::PathBuf {
    let path = session.root().unwrap().join("owned-revision");
    std::fs::write(&path, "exact current bytes").unwrap();
    path
}

#[test]
fn exclusive_seed_drop_rolls_back_only_its_unstarted_editor_revision() {
    let state = Arc::new(StartupState::default());
    let session = Arc::new(EditorMirrorSession::new());
    let other = Arc::new(EditorMirrorSession::new());
    let path = marker(&session);
    let other_path = marker(&other);
    let lease = state.reserve(Arc::clone(&session), true, false).unwrap();
    assert_eq!(state.pending(false), 1);
    assert!(state.reserve(Arc::clone(&session), true, false).is_none());
    assert!(state.reserve(Arc::clone(&session), false, false).is_none());
    assert!(!lease.belongs_to(&Arc::new(StartupState::default()), &session));
    assert!(!lease.belongs_to(&state, &other));
    drop(lease);
    assert!(!path.exists());
    assert_eq!(
        std::fs::read_to_string(other_path).unwrap(),
        "exact current bytes"
    );
    assert_eq!(state.pending(false), 0);
}

#[test]
fn stopped_unentered_seed_retains_files_until_its_owned_guard_is_dropped() {
    let state = Arc::new(StartupState::default());
    let session = Arc::new(EditorMirrorSession::new());
    let path = marker(&session);
    let mut lease = state.reserve(Arc::clone(&session), true, false).unwrap();
    let initialized = AtomicBool::new(false);
    assert!(state.stop(&initialized));
    state.stopped(&session);
    assert!(path.exists(), "live seed still leases its exact mirror");
    assert!(!lease.enter());
    assert!(matches!(
        lease.publish(&initialized, true),
        Err(CorsaBridgeError::ProcessTerminated)
    ));
    drop(lease);
    assert!(!path.exists());
    assert!(state.reserve(session, false, false).is_some());
}

#[test]
fn late_entered_owner_is_not_ready_and_requires_shutdown_after_caller_finishes() {
    let state = Arc::new(StartupState::default());
    let session = Arc::new(EditorMirrorSession::new());
    let path = marker(&session);
    let mut lease = state.reserve(Arc::clone(&session), true, false).unwrap();
    let initialized = AtomicBool::new(false);
    assert!(lease.enter());
    lease.retain();
    assert!(matches!(
        lease.publish(&initialized, false),
        Err(CorsaBridgeError::ProcessTerminated)
    ));
    drop(lease);
    assert!(!initialized.load(Ordering::SeqCst));
    assert_eq!(state.pending(false), 1, "unpublished owner needs cleanup");
    assert!(path.exists(), "an entered owner must keep its mirror");
    assert!(state.stop(&initialized));
    state.stopped(&session);
    assert_eq!(state.pending(false), 0);
    assert!(!path.exists());
}

#[test]
fn shutdown_error_keeps_the_exact_owner_and_mirror_until_successful_cleanup() {
    let state = Arc::new(StartupState::default());
    let session = Arc::new(EditorMirrorSession::new());
    let path = marker(&session);
    let mut lease = state.reserve(Arc::clone(&session), false, false).unwrap();
    let initialized = AtomicBool::new(false);
    assert!(lease.enter());
    lease.retain();
    lease.publish(&initialized, true).unwrap();
    drop(lease);
    let mut slot = Some(41);
    assert!(state.stop(&initialized));
    let failed = state.finish_shutdown(&mut slot, &session, |value| {
        *value += 1;
        Err(CorsaBridgeError::CommunicationError("not joined".into()))
    });
    assert!(
        matches!(failed, Err(CorsaBridgeError::CommunicationError(ref e)) if e == "not joined")
    );
    assert_eq!(slot, Some(42));
    assert!(path.exists());
    assert_eq!(state.pending(false), 1);
    assert!(state.reserve(Arc::clone(&session), false, false).is_none());
    state
        .finish_shutdown(&mut slot, &session, |_| Ok(()))
        .unwrap();
    assert_eq!(slot, None);
    assert!(!path.exists());
    assert_eq!(state.pending(false), 0);
}

#[test]
fn queued_cancellation_drops_the_seed_and_pending_count_without_entering_it() {
    let worker = BoundedWorker::new("vize-seed-cancel", ());
    let (entered_tx, entered) = mpsc::channel();
    let (release_tx, release) = mpsc::channel();
    let mut blocker = Box::pin(worker.submit_ready_async(SETTLE, move |_| {
        entered_tx.send(()).unwrap();
        release.recv_timeout(SETTLE).unwrap();
    }));
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    assert!(matches!(blocker.as_mut().poll(&mut cx), Poll::Pending));
    entered.recv_timeout(SETTLE).unwrap();

    let state = Arc::new(StartupState::default());
    let session = Arc::new(EditorMirrorSession::new());
    let path = marker(&session);
    let mut lease = state.reserve(Arc::clone(&session), true, false).unwrap();
    let (ran_tx, ran) = mpsc::channel();
    let mut queued = Box::pin(worker.submit_ready_async(SETTLE, move |_| {
        assert!(lease.enter());
        ran_tx.send(()).unwrap();
    }));
    assert!(matches!(
        Pin::as_mut(&mut queued).poll(&mut cx),
        Poll::Pending
    ));
    drop(queued);
    release_tx.send(()).unwrap();
    corsa::runtime::block_on(blocker).unwrap();
    corsa::runtime::block_on(worker.submit_ready_async(SETTLE, |_| ())).unwrap();
    assert!(ran.try_recv().is_err());
    assert!(!path.exists());
    assert_eq!(state.pending(false), 0);
}

#[test]
fn stop_between_entered_startup_and_publish_fences_the_original_epoch() {
    let state = Arc::new(StartupState::default());
    let session = Arc::new(EditorMirrorSession::new());
    let mut lease = state.reserve(Arc::clone(&session), false, false).unwrap();
    let initialized = Arc::new(AtomicBool::new(false));
    let (entered_tx, entered) = mpsc::channel();
    let (release_tx, release) = mpsc::channel();
    let target = Arc::clone(&initialized);
    let worker = std::thread::spawn(move || {
        assert!(lease.enter());
        entered_tx.send(()).unwrap();
        release.recv_timeout(SETTLE).unwrap();
        lease.retain();
        lease.publish(&target, true)
    });
    entered.recv_timeout(SETTLE).unwrap();
    assert!(state.stop(&initialized));
    release_tx.send(()).unwrap();
    assert!(matches!(
        worker.join().unwrap(),
        Err(CorsaBridgeError::ProcessTerminated)
    ));
    assert!(!initialized.load(Ordering::SeqCst));
    assert_eq!(state.pending(false), 1);
    state.stopped(&session);
}

#[test]
fn pending_outer_stop_queues_cleanup_without_waiting_for_entered_or_cancelled_work() {
    let bridge = CorsaBridge::new();
    let path = marker(&bridge.editor_session);
    let mut lease = bridge
        .startup
        .reserve(Arc::clone(&bridge.editor_session), false, false)
        .unwrap();
    let initialized = Arc::clone(&bridge.initialized);
    let (entered_tx, entered) = mpsc::channel();
    let (release_tx, release) = mpsc::channel();
    let mut startup = Box::pin(bridge.submit_ready_async(move |_| {
        assert!(lease.enter());
        entered_tx.send(()).unwrap();
        release.recv_timeout(SETTLE).unwrap();
        lease.publish(&initialized, true)
    }));
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    assert!(matches!(startup.as_mut().poll(&mut cx), Poll::Pending));
    entered.recv_timeout(SETTLE).unwrap();
    assert!(bridge.stop_pending_startup());
    assert!(path.exists(), "cleanup waits behind the still-entered job");
    drop(startup);
    release_tx.send(()).unwrap();
    corsa::runtime::block_on(bridge.submit_ready_async(|_| Ok(()))).unwrap();
    assert!(!bridge.is_initialized());
    assert_eq!(bridge.pending_startups(), 0);
    assert!(!path.exists());
}

#[test]
fn entered_factory_error_without_returned_owner_never_admits_a_second_startup() {
    let state = Arc::new(StartupState::default());
    let session = Arc::new(EditorMirrorSession::new());
    let path = marker(&session);
    let root = session.root().unwrap().to_path_buf();
    let mut lease = state.reserve(Arc::clone(&session), true, false).unwrap();
    assert!(lease.enter());
    lease.retain_after_failure();
    drop(lease);
    assert_eq!(state.pending(false), 1);
    assert!(state.reserve(Arc::clone(&session), false, false).is_none());
    assert!(state.stop(&AtomicBool::new(false)));
    let mut slot: Option<()> = None;
    assert!(matches!(
        state.finish_shutdown(&mut slot, &session, |_| Ok(())),
        Err(CorsaBridgeError::CommunicationError(ref error))
            if error == "failed native startup returned no owner to join"
    ));
    assert_eq!(state.pending(false), 1);
    drop(session);
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "exact current bytes"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn mutable_worker_epoch_covers_sync_async_ready_error_and_queued_cleanup() {
    let bridge = CorsaBridge::new();
    let observed = Arc::clone(&bridge.client_operation_epoch);
    assert_eq!(observed.load(Ordering::SeqCst), 0);
    let inside = Arc::clone(&observed);
    assert_eq!(
        bridge
            .submit(move |_| Ok(inside.load(Ordering::SeqCst)))
            .unwrap(),
        1
    );
    let inside = Arc::clone(&observed);
    let failed: Result<(), _> = corsa::runtime::block_on(bridge.submit_async(move |_| {
        assert_eq!(inside.load(Ordering::SeqCst), 2);
        Err(CorsaBridgeError::CommunicationError(
            "metadata failed".into(),
        ))
    }));
    assert!(matches!(
        failed,
        Err(CorsaBridgeError::CommunicationError(_))
    ));
    let inside = Arc::clone(&observed);
    assert_eq!(
        corsa::runtime::block_on(
            bridge.submit_ready_async(move |_| Ok(inside.load(Ordering::SeqCst)))
        )
        .unwrap(),
        3
    );
    bridge.clear_cache();
    assert_eq!(observed.load(Ordering::SeqCst), 4);
    let lease = bridge
        .startup
        .reserve(Arc::clone(&bridge.editor_session), false, false)
        .unwrap();
    assert!(bridge.stop_pending_startup());
    drop(lease);
    // The retirement itself increments once on the worker, then this barrier
    // increments once. No callback, cancelled job or waiter invents an epoch.
    let inside = Arc::clone(&observed);
    assert_eq!(
        corsa::runtime::block_on(
            bridge.submit_ready_async(move |_| Ok(inside.load(Ordering::SeqCst)))
        )
        .unwrap(),
        6
    );
}
