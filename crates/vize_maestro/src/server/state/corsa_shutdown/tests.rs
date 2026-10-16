use super::super::ServerState;
use std::sync::Arc;
use tower_lsp::lsp_types::Url;
use vize_canon::CorsaBridge;

#[test]
fn final_shutdown_drains_root_and_package_caches_before_retiring_contexts() {
    let root = tempfile::tempdir().unwrap();
    let package = root.path().join("packages/child");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(root.path().join("tsconfig.json"), "{}").unwrap();
    std::fs::write(package.join("tsconfig.json"), "{}").unwrap();
    let owner = Arc::new(ServerState::new());
    owner.set_workspace_root(root.path().to_path_buf());
    owner.apply_project_path_identity(root.path(), Some(root.path()));
    let uri = Url::from_file_path(package.join("App.vue")).unwrap();
    let child = owner.document_project_state(&uri).unwrap();
    assert!(!Arc::ptr_eq(&owner, &child));
    *owner.corsa_bridge.write() = Some(Arc::new(CorsaBridge::new()));
    *child.corsa_bridge.write() = Some(Arc::new(CorsaBridge::new()));
    assert!(futures::executor::block_on(owner.shutdown_corsa_project_contexts()).is_ok());
    assert!(owner.corsa_bridge.read().is_none());
    assert!(child.corsa_bridge.read().is_none());
    assert!(owner.project_context_retired());
    assert!(child.project_context_retired());
    assert!(owner.cached_project_states().is_empty());
    assert!(owner.document_project_state(&uri).is_none());
    assert!(futures::executor::block_on(owner.get_corsa_bridge()).is_none());
    assert!(futures::executor::block_on(child.get_corsa_bridge()).is_none());
    assert!(futures::executor::block_on(owner.shutdown_corsa_project_contexts()).is_ok());
}

#[test]
fn retry_retains_failed_owner_until_acknowledged_and_preserves_first_failure() {
    let owner = ServerState::new();
    let bridge = Arc::new(CorsaBridge::new());
    let weak = Arc::downgrade(&bridge);
    {
        // Seed the exact retained state left by an earlier deadline failure;
        // the retry below still executes the real bridge owner-queue cleanup.
        let mut state = owner.corsa_shutdown.try_lock().unwrap();
        state.pending.push(bridge);
        state.first_failure = Some("original native shutdown deadline".into());
    }
    assert!(weak.upgrade().is_some());
    let result = futures::executor::block_on(owner.shutdown_corsa_project_contexts());
    assert_eq!(
        result.unwrap_err().as_str(),
        "original native shutdown deadline"
    );
    assert!(owner.corsa_shutdown.try_lock().unwrap().pending.is_empty());
    assert!(
        weak.upgrade().is_none(),
        "retry released the retained bridge only after queue acknowledgment"
    );
    let repeated = futures::executor::block_on(owner.shutdown_corsa_project_contexts());
    assert_eq!(
        repeated.unwrap_err().as_str(),
        "original native shutdown deadline"
    );
}

#[test]
fn final_exit_retains_owner_across_config_callbacks_but_ordinary_retirement_does_not_wait() {
    let ordinary = ServerState::new();
    *ordinary.corsa_bridge.write() = Some(Arc::new(CorsaBridge::new()));
    ordinary.retire_corsa_configuration();
    assert!(ordinary.corsa_bridge.read().is_none());

    let closing = ServerState::new();
    let bridge = Arc::new(CorsaBridge::new());
    *closing.corsa_bridge.write() = Some(Arc::clone(&bridge));
    closing.freeze_corsa_owner_for_exit();
    closing.retire_corsa_configuration();
    closing.retire_corsa_bridge(&bridge);
    assert!(Arc::ptr_eq(
        closing.corsa_bridge.read().as_ref().unwrap(),
        &bridge
    ));
    assert!(futures::executor::block_on(closing.shutdown_corsa_project_contexts()).is_ok());
    assert!(closing.corsa_bridge.read().is_none());
    assert!(matches!(
        futures::executor::block_on(bridge.spawn()),
        Err(vize_canon::CorsaBridgeError::ProcessTerminated)
    ));
}
