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
mod tests;
