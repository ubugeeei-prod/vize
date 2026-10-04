#[cfg(feature = "native")]
use super::ModuleTargetGateError;
use super::{ModuleLinkRetirement, Phase, ServerState};

#[test]
fn module_link_target_session_adds_only_original_received_event_counter_storage() {
    struct OriginalSession {
        _identity: super::Arc<()>,
        _generation: u64,
        _phase: Phase,
        _load_origin: Option<std::path::PathBuf>,
    }
    assert_eq!(
        std::mem::size_of::<super::super::Session>(),
        std::mem::size_of::<OriginalSession>() + std::mem::size_of::<u64>()
    );
    assert_eq!(
        std::mem::align_of::<super::super::Session>(),
        std::mem::align_of::<OriginalSession>()
    );
}

#[test]
fn module_link_target_received_event_overflow_retires_without_wrapping_or_legacy_mutation() {
    let state = ServerState::new();
    state.module_links.write().observed_file_epoch = u64::MAX;
    state.observe_module_target_file_events(false);
    assert_eq!(state.module_links.read().phase, Phase::Live);
    state.observe_module_target_file_events(true);
    assert_eq!(state.module_links.read().observed_file_epoch, u64::MAX);
    assert_eq!(
        state.module_links.read().phase,
        Phase::Retired(ModuleLinkRetirement::EventEpochExhausted)
    );
    state.retire_module_links(ModuleLinkRetirement::Shutdown);
    assert!(matches!(
        state.capture_module_link_context(),
        Err(super::ModuleLinkContextError::Retired(
            ModuleLinkRetirement::EventEpochExhausted
        ))
    ));
    assert_eq!(state.lsp_request_timeout_ms(), 60_000);
}

#[cfg(feature = "native")]
#[test]
fn module_link_target_empty_batches_preserve_actual_context_equal_batches_supersede_only_events() {
    let state = ServerState::new();
    state.set_workspace_root("/original-target-root".into());
    let context = state.capture_module_link_context().unwrap();
    let stamp = state.capture_module_target_stamp(&context).unwrap();
    state.observe_module_target_file_events(false);
    assert_eq!(state.with_current_module_target(&stamp, || 7), Ok(7));
    state.observe_module_target_file_events(true);
    state.observe_module_target_file_events(true);
    assert_eq!(
        state.with_current_module_target(&stamp, || 7),
        Err(ModuleTargetGateError::EventSuperseded)
    );
    assert_eq!(
        state.with_current_module_link_context(&context, || 9),
        Ok(9)
    );
    assert_eq!(state.module_links.read().observed_file_epoch, 2);
}

#[cfg(feature = "native")]
#[test]
fn module_link_target_actual_gate_guard_excludes_event_write_through_entire_callback() {
    let state = ServerState::new();
    state.set_workspace_root("/original-target-root".into());
    let context = state.capture_module_link_context().unwrap();
    let stamp = state.capture_module_target_stamp(&context).unwrap();
    assert_eq!(
        state.with_current_module_target(&stamp, || {
            assert!(state.module_links.try_write().is_none());
            7
        }),
        Ok(7)
    );
    assert!(state.module_links.try_write().is_some());
    state.observe_module_target_file_events(true);
    assert_eq!(
        state.with_current_module_target(&stamp, || 9),
        Err(ModuleTargetGateError::EventSuperseded)
    );
}

#[cfg(not(feature = "native"))]
#[test]
fn module_link_target_minimal_events_keep_authentic_native_unavailable_and_retirement() {
    let state = ServerState::new();
    state.observe_module_target_file_events(true);
    assert!(matches!(
        state.capture_module_link_context(),
        Err(super::ModuleLinkContextError::NativeUnavailable)
    ));
    state.retire_module_links(ModuleLinkRetirement::InputEof);
    assert!(matches!(
        state.capture_module_link_context(),
        Err(super::ModuleLinkContextError::Retired(
            ModuleLinkRetirement::InputEof
        ))
    ));
}
