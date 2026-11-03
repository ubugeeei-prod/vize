//! Sticky unknown-owner storage follows the existing SessionStorage law.

use super::{EditorMirrorSession, GcLock, RETAINED_FAILURE, open_existing_session_lock};

fn generated(session: &EditorMirrorSession) -> std::path::PathBuf {
    let file = session.root().unwrap().join("projects/current/App.vue.ts");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "export const generated = 42;\n").unwrap();
    file
}

#[test]
fn successful_owner_removes_its_generated_files_normally() {
    let session = EditorMirrorSession::new();
    let file = generated(&session);
    let root = session.root().unwrap().to_path_buf();
    session.arm_retention_before_startup().unwrap();
    assert!(root.join(RETAINED_FAILURE).is_file());
    drop(session);
    assert!(!file.exists());
    assert!(!root.exists());
}

#[test]
fn failed_owner_keeps_exact_generated_bytes_through_clear_drop_and_later_sweep() {
    let session = EditorMirrorSession::new();
    let file = generated(&session);
    let root = session.root().unwrap().to_path_buf();
    session.arm_retention_before_startup().unwrap();
    session.retain_after_failed_shutdown();
    session.clear();
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "export const generated = 42;\n"
    );
    drop(session);
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "export const generated = 42;\n"
    );
    assert!(root.join(RETAINED_FAILURE).is_file());
    GcLock::acquire(root.parent().unwrap())
        .unwrap()
        .reap_abandoned_sessions();
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "export const generated = 42;\n"
    );
    // This fixture has no native reader; only the test removes its own owner.
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn actual_marker_creation_failure_is_unarmed_and_rollback_restores_normal_cleanup() {
    let session = EditorMirrorSession::new();
    let file = generated(&session);
    let root = session.root().unwrap().to_path_buf();
    std::fs::create_dir(root.join(RETAINED_FAILURE)).unwrap();
    assert!(session.arm_retention_before_startup().is_err());
    let storage = session.storage.as_ref().unwrap();
    assert!(!storage.retention_armed.load(super::Ordering::Acquire));
    session.retain_after_failed_shutdown();
    assert!(!storage.retain_after_failure.load(super::Ordering::Acquire));
    session.clear();
    assert!(!file.exists());
    assert!(!root.join(RETAINED_FAILURE).exists());
    session.arm_retention_before_startup().unwrap();
    assert!(root.join(RETAINED_FAILURE).is_file());
    drop(session);
    assert!(!root.exists());
}

#[test]
fn generic_unstarted_session_keeps_its_original_cleanup_without_sticky_claims() {
    let session = EditorMirrorSession::new();
    let file = generated(&session);
    let root = session.root().unwrap().to_path_buf();
    assert!(!root.join(RETAINED_FAILURE).exists());
    session.retain_after_failed_shutdown();
    assert!(
        !session
            .storage
            .as_ref()
            .unwrap()
            .retain_after_failure
            .load(super::Ordering::Acquire)
    );
    drop(session);
    assert!(!file.exists());
    assert!(!root.exists());
}

#[cfg(unix)]
#[test]
fn stale_reaper_observation_never_recreates_an_unlinked_lifetime_lock() {
    let session = EditorMirrorSession::new();
    let root = session.root().unwrap().to_path_buf();
    let lock_path = root.join(".session.lock");
    // The reaper has already observed this path before owner cleanup unlinks it.
    assert!(lock_path.is_file());
    std::fs::remove_file(&lock_path).unwrap();
    let error = open_existing_session_lock(&lock_path).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    assert!(!lock_path.exists());
    drop(session);
    assert!(!root.exists());
}

#[test]
fn reaper_open_respects_the_current_owner_lock_and_never_creates_missing_storage() {
    let session = EditorMirrorSession::new();
    let root = session.root().unwrap().to_path_buf();
    let lock = open_existing_session_lock(&root.join(".session.lock")).unwrap();
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    drop(lock);
    let missing = root.join("removed-session/.session.lock");
    let error = open_existing_session_lock(&missing).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    assert!(!missing.parent().unwrap().exists());
    drop(session);
    assert!(!root.exists());
}

/// A sweep racing session creation must never remove a session being born (#11500).
#[test]
fn concurrent_sweeps_never_remove_a_session_being_created() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let base = super::sessions_base();
    std::fs::create_dir_all(&base).unwrap();
    let stop = std::sync::Arc::new(AtomicBool::new(false));
    let sweeper = {
        let stop = std::sync::Arc::clone(&stop);
        let base = base.clone();
        std::thread::spawn(move || {
            while !stop.load(Ordering::Acquire) {
                GcLock::acquire(&base).unwrap().reap_abandoned_sessions();
            }
        })
    };
    let mut failures = 0;
    for _ in 0..2000 {
        let session = EditorMirrorSession::new();
        match session.root() {
            Ok(root) if root.join(".session.lock").is_file() => {}
            _ => failures += 1,
        }
    }
    stop.store(true, Ordering::Release);
    sweeper.join().unwrap();
    assert_eq!(failures, 0);
}
