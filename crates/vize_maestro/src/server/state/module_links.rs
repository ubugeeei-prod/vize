//! Same-server applied metadata, independent of syntax and Names admission.
#![expect(
    clippy::disallowed_types,
    reason = "session tickets retain their real Arc identity and Weak server owner"
)]
#![allow(
    dead_code,
    reason = "private context/publication prerequisite; target and module-link consumer are not implemented"
)]

use super::ServerState;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Weak},
};
use vize_carton::config::ProjectModel;
use vize_l0::config::TypeCheckerConfig;
mod mutation;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModuleLinkRetirement {
    Shutdown,
    ForegroundDropped,
    TransportEnded,
    InputEof,
    InputError,
    GenerationExhausted,
    MutationUnwound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModuleLinkContextError {
    HostUnavailable,
    NativeUnavailable,
    MissingRoot,
    ForeignSession,
    Superseded,
    Updating,
    Retired(ModuleLinkRetirement),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Live,
    Updating,
    Retired(ModuleLinkRetirement),
}

pub(super) struct Session {
    identity: Arc<()>,
    generation: u64,
    phase: Phase,
    load_origin: Option<PathBuf>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            identity: Arc::new(()),
            generation: 0,
            phase: Phase::Live,
            load_origin: None,
        }
    }
}

impl Session {
    fn live(&self) -> Result<(), ModuleLinkContextError> {
        match self.phase {
            Phase::Live => Ok(()),
            Phase::Updating => Err(ModuleLinkContextError::Updating),
            Phase::Retired(reason) => Err(ModuleLinkContextError::Retired(reason)),
        }
    }
    fn retire(&mut self, reason: ModuleLinkRetirement) {
        if !matches!(self.phase, Phase::Retired(_)) {
            self.phase = Phase::Retired(reason);
        }
    }
}

/// Owned actual metadata. Construction/session/generation stay private.
/// This grants no File, resolver, target existence, or filesystem freshness.
pub(crate) struct ModuleLinkContext {
    identity: Arc<()>,
    generation: u64,
    root: PathBuf,
    checker: TypeCheckerConfig,
    timeout_ms: u64,
    load_origin: Option<PathBuf>,
    project: ProjectModel,
}

impl ModuleLinkContext {
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
    pub(crate) fn checker(&self) -> &TypeCheckerConfig {
        &self.checker
    }
    pub(crate) fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }
    pub(crate) fn load_origin(&self) -> Option<&Path> {
        self.load_origin.as_deref()
    }
    pub(crate) fn project(&self) -> &ProjectModel {
        &self.project
    }
}

impl ServerState {
    pub(crate) fn capture_module_link_context(
        &self,
    ) -> Result<ModuleLinkContext, ModuleLinkContextError> {
        let gate = self.module_links.read();
        gate.live()?;
        #[cfg(not(feature = "native"))]
        {
            Err(ModuleLinkContextError::NativeUnavailable)
        }
        #[cfg(feature = "native")]
        {
            // No field-held caller can enter the module gate. Read only actual
            // fields while this gate prevents either authoritative assignment.
            let root = self
                .workspace_root
                .read()
                .clone()
                .ok_or(ModuleLinkContextError::MissingRoot)?;
            let (checker, timeout_ms) = self.type_checker_config.read().clone();
            let project = ProjectModel::new(Some(&root), None, &checker);
            Ok(ModuleLinkContext {
                identity: Arc::clone(&gate.identity),
                generation: gate.generation,
                root,
                checker,
                timeout_ms,
                project,
                load_origin: gate.load_origin.clone(),
            })
        }
    }

    /// Called inside original DocumentStore read publication only. Keep this
    /// guard through owned synchronous publication; no store/map/Names/await,
    /// reentry, worker retirement/wake, or arbitrary user callback is allowed.
    pub(crate) fn with_current_module_link_context<T>(
        &self,
        context: &ModuleLinkContext,
        publish: impl FnOnce() -> T,
    ) -> Result<T, ModuleLinkContextError> {
        let gate = self.module_links.read();
        if !Arc::ptr_eq(&gate.identity, &context.identity) {
            return Err(ModuleLinkContextError::ForeignSession);
        }
        gate.live()?;
        if gate.generation != context.generation {
            return Err(ModuleLinkContextError::Superseded);
        }
        Ok(publish())
    }

    pub(crate) fn retire_module_links(&self, reason: ModuleLinkRetirement) {
        self.module_links.write().retire(reason);
    }

    // Sole writers call this around ONLY relevant authoritative assignments.
    // Prepare owned paths before entering; release before every other effect.
    pub(super) fn update_module_link_context(&self, origin: Option<PathBuf>, apply: impl FnOnce()) {
        let mut gate = self.module_links.write();
        let mutation = mutation::Mutation::begin(&mut gate);
        apply();
        mutation.commit(origin);
    }
}

/// Foreground/transport only. Diagnostic workers never own one. Weak avoids a
/// cycle and retained state Arcs cannot delay synchronous session retirement.
pub(crate) struct ModuleLinkTerminationLease {
    state: Weak<ServerState>,
    reason: ModuleLinkRetirement,
}

impl ModuleLinkTerminationLease {
    pub(crate) fn new(state: &Arc<ServerState>, reason: ModuleLinkRetirement) -> Self {
        Self {
            state: Arc::downgrade(state),
            reason,
        }
    }
    pub(crate) fn for_transport(&self) -> Self {
        Self {
            state: self.state.clone(),
            reason: ModuleLinkRetirement::TransportEnded,
        }
    }
    pub(crate) fn retire(&self, reason: ModuleLinkRetirement) {
        if let Some(state) = self.state.upgrade() {
            state.retire_module_links(reason);
        }
    }
}

impl Drop for ModuleLinkTerminationLease {
    fn drop(&mut self) {
        self.retire(self.reason);
    }
}
