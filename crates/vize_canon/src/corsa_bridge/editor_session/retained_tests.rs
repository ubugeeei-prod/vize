//! Sticky unknown-owner storage follows the existing SessionStorage law.

use super::{EditorMirrorSession, RETAINED_FAILURE, reap_abandoned_sessions};

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
    reap_abandoned_sessions(root.parent().unwrap());
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
