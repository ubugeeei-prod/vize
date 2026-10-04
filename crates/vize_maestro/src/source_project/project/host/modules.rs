//! The actual retained host alone supplies applied module context authority.
#![allow(
    dead_code,
    reason = "private metadata prerequisite, not a DocumentLinks implementation"
)]
use super::DocumentHost;
use crate::server::{ModuleLinkContext, ModuleLinkContextError};
use crate::source_project::SourceQueryProject;

impl DocumentHost<'_> {
    pub(in crate::source_project::project) fn capture_module_link_context(
        &self,
    ) -> Result<ModuleLinkContext, ModuleLinkContextError> {
        match self {
            Self::Server(state) => state.capture_module_link_context(),
            _ => Err(ModuleLinkContextError::HostUnavailable),
        }
    }
    pub(in crate::source_project::project) fn with_current_module_link_context<T>(
        &self,
        context: &ModuleLinkContext,
        publish: impl FnOnce() -> T,
    ) -> Result<T, ModuleLinkContextError> {
        match self {
            Self::Server(state) => state.with_current_module_link_context(context, publish),
            _ => Err(ModuleLinkContextError::HostUnavailable),
        }
    }
}
impl SourceQueryProject<'_> {
    /// Releases the module/field guards before returning; no source/map lock.
    pub(crate) fn capture_module_link_context(
        &self,
    ) -> Result<ModuleLinkContext, ModuleLinkContextError> {
        self.host.capture_module_link_context()
    }
}
