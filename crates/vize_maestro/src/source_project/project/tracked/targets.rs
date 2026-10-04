//! Genuine ready-query ownership, then bounded IO outside every host guard.
#![allow(
    dead_code,
    reason = "private physical provider; no native DocumentLinks consumer"
)]
use super::super::host::DocumentHost;
use super::{Arc, ProjectQueryResult, SourceSnapshot};
use crate::{
    server::{
        ModuleLinkContext, ModuleLinkContextError, ModuleTargetError, ModuleTargetStamp,
        PhysicalTargets, ServerState, TargetPolicy, WatcherCoverageError,
    },
    source_project::SnapshotRefusal,
};
use futures::future::AbortHandle;
use tower_lsp::lsp_types::Url;

pub(crate) struct ObservedModuleTargets {
    state: Arc<ServerState>,
    snapshot: Arc<SourceSnapshot>,
    cancel: AbortHandle,
    stamp: ModuleTargetStamp,
    physical: PhysicalTargets,
}
pub(crate) struct RecheckedModuleTargets(ObservedModuleTargets);
impl ObservedModuleTargets {
    pub(crate) fn uris(&self) -> &[Url] {
        self.physical.uris()
    }
    pub(crate) fn recheck(self) -> Result<RecheckedModuleTargets, ModuleTargetError> {
        self.current()?;
        let fresh = self.physical.recheck();
        self.current()?;
        let fresh = fresh?;
        Ok(RecheckedModuleTargets(Self {
            physical: fresh,
            ..self
        }))
    }
    fn current(&self) -> Result<(), ModuleTargetError> {
        self.snapshot
            .with_current(&self.state.documents, |_| {
                if self.cancel.is_aborted() {
                    return Err(ModuleTargetError::Source(SnapshotRefusal::Cancelled));
                }
                self.state
                    .with_current_module_target(&self.stamp, || ())
                    .map_err(ModuleTargetError::from)
            })
            .map_err(ModuleTargetError::Source)?
    }
}
impl RecheckedModuleTargets {
    pub(crate) fn uris(&self) -> &[Url] {
        self.0.physical.uris()
    }
    /// Point observations deliberately carry no acknowledged watcher claim.
    pub(crate) fn require_watcher_covered(&self) -> Result<(), WatcherCoverageError> {
        Err(WatcherCoverageError::UnknownCoverage)
    }
}
impl<T> ProjectQueryResult<'_, T> {
    /// Caller strings are physical requests, never static module/Occurrence proof.
    pub(crate) fn observe_module_targets(
        &self,
        context: &ModuleLinkContext,
        candidates: &[&str],
    ) -> Result<ObservedModuleTargets, ModuleTargetError> {
        let (state, stamp) = self
            .result
            .with_current(self.host.documents(), || {
                let DocumentHost::Server(state) = &self.host else {
                    return Err(ModuleTargetError::Context(
                        ModuleLinkContextError::HostUnavailable,
                    ));
                };
                let stamp = state
                    .capture_module_target_stamp(context)
                    .map_err(ModuleTargetError::Context)?;
                Ok((Arc::clone(state), stamp))
            })
            .map_err(ModuleTargetError::Source)??;
        if candidates.is_empty() {
            return Err(ModuleTargetError::Policy(TargetPolicy::EmptyBatch));
        }
        let physical =
            PhysicalTargets::observe(stamp.root(), self.result.snapshot().uri(), candidates);
        // Always revalidate owner facts after IO, even when IO itself refused.
        self.result
            .with_current(self.host.documents(), || {
                state
                    .with_current_module_target(&stamp, || ())
                    .map_err(ModuleTargetError::from)
            })
            .map_err(ModuleTargetError::Source)??;
        let physical = physical?;
        let observed = ObservedModuleTargets {
            state,
            stamp,
            physical,
            snapshot: Arc::clone(self.result.snapshot()),
            cancel: self.result.cancellation(),
        };
        observed.current()?;
        Ok(observed)
    }

    /// Original source -> context -> event ordering. No IO or await in callback.
    /// The entire batch is retained until this one synchronous publication ends.
    pub(crate) fn publish_with_module_targets<R>(
        self,
        targets: RecheckedModuleTargets,
        publish: impl FnOnce(T, &[Url]) -> R,
    ) -> Result<R, ModuleTargetError> {
        let Self {
            result,
            host,
            _lease,
        } = self;
        let original = Arc::ptr_eq(result.snapshot(), &targets.0.snapshot);
        result
            .publish(host.documents(), |ready| {
                if targets.0.cancel.is_aborted() {
                    return Err(ModuleTargetError::Source(SnapshotRefusal::Cancelled));
                }
                if !original {
                    return Err(ModuleTargetError::ForeignSnapshot);
                }
                let DocumentHost::Server(state) = &host else {
                    return Err(ModuleTargetError::Context(
                        ModuleLinkContextError::HostUnavailable,
                    ));
                };
                // Original source identity, then genuine context, then event epoch.
                state
                    .with_current_module_target(&targets.0.stamp, || {
                        Ok(publish(ready, targets.uris()))
                    })
                    .map_err(ModuleTargetError::from)?
            })
            .map_err(ModuleTargetError::Source)?
    }
}
