//! Join source and applied context using this result's own retained host.
#![allow(
    dead_code,
    reason = "private context prerequisite; target/DocumentLinks remain unfinished"
)]
use super::ProjectQueryResult;
use crate::{
    server::{ModuleLinkContext, ModuleLinkContextError},
    source_project::SnapshotRefusal,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModuleLinkPublicationError {
    Source(SnapshotRefusal),
    Context(ModuleLinkContextError),
}
impl<T> ProjectQueryResult<'_, T> {
    /// Qualify even context refusal against this original source/cancellation.
    /// DocumentStore read -> module read; no IO, await or alternate host.
    pub(crate) fn capture_current_module_link_context(
        &self,
    ) -> Result<ModuleLinkContext, ModuleLinkPublicationError> {
        self.result
            .with_current(self.host.documents(), || {
                self.host
                    .capture_module_link_context()
                    .map_err(ModuleLinkPublicationError::Context)
            })
            .map_err(ModuleLinkPublicationError::Source)?
    }

    /// DocumentStore read -> module read -> synchronous owned publication.
    /// Never reenter a store, lifecycle/map/Names gate, retire/wake workers,
    /// call user code or await. No caller-supplied host can authenticate this.
    pub(crate) fn publish_with_module_link_context<R>(
        self,
        context: &ModuleLinkContext,
        publish: impl FnOnce(T) -> R,
    ) -> Result<R, ModuleLinkPublicationError> {
        let Self {
            result,
            host,
            _lease,
        } = self;
        result
            .publish(host.documents(), |ready| {
                host.with_current_module_link_context(context, || publish(ready))
                    .map_err(ModuleLinkPublicationError::Context)
            })
            .map_err(ModuleLinkPublicationError::Source)?
    }
}
