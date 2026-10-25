//! Package-local once initialization and retirement cover in-flight discovery.
#![expect(
    clippy::disallowed_types,
    reason = "registry cells retain their isolated native owner"
)]
use super::{
    Arc, AtomicBool, ObservedLspConfig, Ordering, ProjectContextKey, RetainedConfiguration,
    ServerState,
};
#[cfg(feature = "retained-program")]
use crate::source_project::{SourceQueryProject, navigation::NativeNavigationProject};
use futures::{
    FutureExt,
    future::{BoxFuture, Shared},
};
use std::sync::OnceLock;
type Initialization = Shared<BoxFuture<'static, Option<Arc<ServerState>>>>;

pub(super) struct ProjectContext {
    key: ProjectContextKey,
    snapshot: OnceLock<(ObservedLspConfig, bool)>,
    prepared: OnceLock<RetainedConfiguration>,
    state: OnceLock<Arc<ServerState>>,
    #[cfg(feature = "retained-program")]
    navigation: OnceLock<Arc<NativeNavigationProject<'static>>>,
    retired: AtomicBool,
    reload_failed: AtomicBool,
    initialization: super::Mutex<Option<Initialization>>,
}

impl ProjectContext {
    pub(super) fn new(key: impl Into<ProjectContextKey>) -> Self {
        Self {
            key: key.into(),
            snapshot: OnceLock::new(),
            prepared: OnceLock::new(),
            state: OnceLock::new(),
            #[cfg(feature = "retained-program")]
            navigation: OnceLock::new(),
            retired: AtomicBool::new(false),
            reload_failed: AtomicBool::new(false),
            initialization: super::Mutex::new(None),
        }
    }

    pub(super) fn with_prepared(key: ProjectContextKey, prepared: RetainedConfiguration) -> Self {
        let context = Self::new(key);
        let _ = context.prepared.set(prepared);
        context
    }

    pub(super) fn config_dependencies(&self) -> Vec<std::path::PathBuf> {
        if let Some(state) = self.initialized_state() {
            return state.project_contexts.config_dependencies.read().clone();
        }
        if let Some(prepared) = self.prepared.get() {
            return prepared.applied.project.dependencies.clone();
        }
        self.snapshot
            .get()
            .map(|(snapshot, _)| snapshot.loaded.project.dependencies.clone())
            .unwrap_or_default()
    }

    pub(super) fn configuration_observation(
        &self,
    ) -> Option<super::freshness::ConfigurationObservation> {
        if let Some(state) = self.initialized_state() {
            return state
                .project_contexts
                .configuration_observation
                .read()
                .clone();
        }
        self.prepared
            .get()
            .map(|prepared| prepared.observation.clone())
    }

    pub(super) fn provide_snapshot(&self, snapshot: ObservedLspConfig, failed: bool) {
        let _ = self.snapshot.set((snapshot, failed));
    }

    pub(super) fn same_authority(&self, next: &RetainedConfiguration) -> bool {
        if let Some(state) = self.initialized_state() {
            return state.retains_configuration_authority(next);
        }
        self.prepared
            .get()
            .is_some_and(|previous| next.same_authority(&previous.applied, &previous.compiler))
    }

    pub(super) fn confirm_observation(&self, next: &RetainedConfiguration) {
        if let Some(state) = self.initialized_state() {
            *state.project_contexts.configuration_observation.write() =
                Some(next.observation.clone());
        }
    }

    pub(super) fn record_reload_failure(&self) {
        self.reload_failed.store(true, Ordering::Release);
        if let Some(state) = self.state.get() {
            state
                .project_contexts
                .config_reload_failed
                .store(true, Ordering::Release);
        }
    }

    pub(super) async fn initialize_async(
        self: &Arc<Self>,
        owner: &Arc<ServerState>,
    ) -> Option<Arc<ServerState>> {
        if self.retired.load(Ordering::Acquire) {
            return None;
        }
        if let Some(state) = self.initialized_state() {
            return Some(state);
        }
        let initialization = {
            let mut pending = self.initialization.lock();
            pending.get_or_insert_with(|| {
                let context = Arc::downgrade(self);
                let owner = Arc::downgrade(owner);
                async move {
                    let worker_context = context.clone();
                    let worker_owner = owner.clone();
                    match super::pool::run(move || worker_context.upgrade()?.initialize(&worker_owner.upgrade()?)).await {
                        Some(state) => state,
                        None => {
                            tracing::warn!("project config worker unavailable; resolving configuration inline");
                            context.upgrade()?.initialize(&owner.upgrade()?)
                        }
                    }
                }.boxed().shared()
            }).clone()
        };
        let state = initialization.await;
        if self.retired.load(Ordering::Acquire) {
            if let Some(state) = &state {
                state.retire_project_owner();
            }
            return None;
        }
        state
    }

    pub(super) fn initialize(&self, owner: &Arc<ServerState>) -> Option<Arc<ServerState>> {
        if self.retired.load(Ordering::Acquire) {
            return None;
        }
        let state = self
            .state
            .get_or_init(|| {
                owner.new_project_state(
                    &self.key,
                    self.snapshot.get().map(|(snapshot, _)| snapshot),
                    self.snapshot.get().is_some_and(|(_, failed)| *failed),
                    self.prepared.get(),
                )
            })
            .clone();
        // Recheck after publishing initialization: a rejection can occur after
        // new_project_state captured its snapshot but before the cell was set.
        if self.reload_failed.load(Ordering::Acquire) {
            state
                .project_contexts
                .config_reload_failed
                .store(true, Ordering::Release);
        }
        if self.retired.load(Ordering::Acquire) {
            state.retire_project_owner();
            return None;
        }
        Some(state)
    }

    pub(super) fn initialized_state(&self) -> Option<Arc<ServerState>> {
        (!self.retired.load(Ordering::Acquire))
            .then(|| self.state.get().cloned())
            .flatten()
    }

    pub(super) fn mark_retired(&self) {
        self.retired.store(true, Ordering::Release);
        #[cfg(feature = "retained-program")]
        if let Some(navigation) = self.navigation.get() {
            navigation.mark_retired();
        }
        if let Some(state) = self.state.get() {
            state
                .project_contexts
                .retired
                .store(true, Ordering::Release);
        }
    }

    pub(super) fn retire(&self) {
        self.mark_retired();
        #[cfg(feature = "retained-program")]
        if let Some(navigation) = self.navigation.get() {
            navigation.retire();
        }
        if let Some(state) = self.state.get() {
            state.retire_project_owner();
            #[cfg(feature = "experimental-source-navigation")]
            state.retire_module_links(super::super::ModuleLinkRetirement::Shutdown);
        }
    }

    #[cfg(feature = "retained-program")]
    pub(super) fn navigation_for_state(
        &self,
        state: &Arc<ServerState>,
    ) -> Option<Arc<NativeNavigationProject<'static>>> {
        if self.retired.load(Ordering::Acquire)
            || !self
                .state
                .get()
                .is_some_and(|current| Arc::ptr_eq(current, state))
        {
            return None;
        }
        let navigation = self
            .navigation
            .get_or_init(|| {
                Arc::new(NativeNavigationProject::new(
                    SourceQueryProject::new_server(state.clone()),
                ))
            })
            .clone();
        // Recheck after publishing initialization: a rejection can occur after
        // new_project_state captured its snapshot but before the cell was set.
        if self.reload_failed.load(Ordering::Acquire) {
            state
                .project_contexts
                .config_reload_failed
                .store(true, Ordering::Release);
        }
        if self.retired.load(Ordering::Acquire) {
            navigation.retire();
            return None;
        }
        Some(navigation)
    }
}

#[cfg(feature = "retained-program")]
impl Drop for ProjectContext {
    fn drop(&mut self) {
        self.mark_retired();
        if let Some(navigation) = self.navigation.get() {
            navigation.retire();
        }
    }
}
