use super::should_join_bridge;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[test]
fn publication_between_first_observation_and_pending_stop_is_joined() {
    let ready = AtomicBool::new(false);
    let observations = AtomicUsize::new(0);
    let join = should_join_bridge(
        || {
            observations.fetch_add(1, Ordering::SeqCst);
            ready.load(Ordering::SeqCst)
        },
        || {
            ready.store(true, Ordering::SeqCst);
            false
        },
    );
    assert!(join);
    assert_eq!(observations.load(Ordering::SeqCst), 2);
}

#[test]
fn genuinely_pending_owner_is_fenced_and_never_added_to_outer_wait() {
    let ready = AtomicBool::new(false);
    let stopped = AtomicBool::new(false);
    assert!(!should_join_bridge(
        || ready.load(Ordering::SeqCst),
        || {
            stopped.store(true, Ordering::SeqCst);
            true
        },
    ));
    assert!(stopped.load(Ordering::SeqCst));
}

#[test]
fn already_initialized_owner_keeps_original_join_without_pending_stop() {
    assert!(should_join_bridge(
        || true,
        || panic!("initialized owners use their actual shutdown join")
    ));
}

#[test]
fn absent_owner_is_not_made_ready_by_a_failed_stop_attempt() {
    assert!(!should_join_bridge(|| false, || false));
}
