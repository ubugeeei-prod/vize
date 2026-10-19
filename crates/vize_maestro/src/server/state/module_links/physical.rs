//! Same-session ordering of received events, never complete watcher coverage.
use super::{
    Arc, ModuleLinkContext, ModuleLinkContextError, ModuleLinkRetirement, Phase, ServerState,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModuleTargetGateError {
    Context(ModuleLinkContextError),
    EventSuperseded,
}

pub(crate) struct ModuleTargetStamp {
    identity: Arc<()>,
    generation: u64,
    observed_file_epoch: u64,
    root: PathBuf,
}
impl ModuleTargetStamp {
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
}

impl ServerState {
    pub(crate) fn capture_module_target_stamp(
        &self,
        context: &ModuleLinkContext,
    ) -> Result<ModuleTargetStamp, ModuleLinkContextError> {
        let gate = self.module_links.read();
        if !Arc::ptr_eq(&gate.identity, &context.identity) {
            return Err(ModuleLinkContextError::ForeignSession);
        }
        gate.live()?;
        if gate.generation != context.generation {
            return Err(ModuleLinkContextError::Superseded);
        }
        Ok(ModuleTargetStamp {
            identity: Arc::clone(&gate.identity),
            generation: gate.generation,
            observed_file_epoch: gate.observed_file_epoch,
            root: context.root.clone(),
        })
    }

    /// Only inside the real source publication guard. No IO, await or reentry.
    pub(crate) fn with_current_module_target<T>(
        &self,
        stamp: &ModuleTargetStamp,
        publish: impl FnOnce() -> T,
    ) -> Result<T, ModuleTargetGateError> {
        let gate = self.module_links.read();
        if !Arc::ptr_eq(&gate.identity, &stamp.identity) {
            return Err(ModuleTargetGateError::Context(
                ModuleLinkContextError::ForeignSession,
            ));
        }
        gate.live().map_err(ModuleTargetGateError::Context)?;
        if gate.generation != stamp.generation {
            return Err(ModuleTargetGateError::Context(
                ModuleLinkContextError::Superseded,
            ));
        }
        if gate.observed_file_epoch != stamp.observed_file_epoch {
            return Err(ModuleTargetGateError::EventSuperseded);
        }
        Ok(publish())
    }

    /// Observe original nonempty batches before every legacy filter or await.
    pub(crate) fn observe_module_target_file_events(&self, nonempty: bool) {
        if !nonempty {
            return;
        }
        let mut gate = self.module_links.write();
        if !matches!(gate.phase, Phase::Live) {
            return;
        }
        match gate.observed_file_epoch.checked_add(1) {
            Some(next) => gate.observed_file_epoch = next,
            None => gate.retire(ModuleLinkRetirement::EventEpochExhausted),
        }
    }
}

#[cfg(test)]
mod tests {
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
    fn module_link_target_empty_batches_preserve_actual_context_equal_batches_supersede_only_events()
     {
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
}
