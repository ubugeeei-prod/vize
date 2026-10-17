//! Package-local once initialization and retirement cover in-flight discovery.
#![expect(
    clippy::disallowed_types,
    reason = "registry cells retain their isolated native owner"
)]
use super::{
    Arc, AtomicBool, LoadedLspConfig, Ordering, ProjectContextKey, RetainedConfiguration,
    ServerState,
};
#[cfg(feature = "experimental-source-navigation")]
use crate::source_project::{SourceQueryProject, navigation::NativeNavigationProject};
use futures::{
    FutureExt,
    future::{BoxFuture, Shared},
};
use std::sync::OnceLock;
type Initialization = Shared<BoxFuture<'static, Option<Arc<ServerState>>>>;

pub(super) struct ProjectContext {
    key: ProjectContextKey,
    snapshot: OnceLock<(LoadedLspConfig, bool)>,
    previous: Option<RetainedConfiguration>,
    state: OnceLock<Arc<ServerState>>,
    #[cfg(feature = "experimental-source-navigation")]
    navigation: OnceLock<Arc<NativeNavigationProject<'static>>>,
    retired: AtomicBool,
    initialization: super::Mutex<Option<Initialization>>,
}

impl ProjectContext {
    pub(super) fn new(key: impl Into<ProjectContextKey>) -> Self {
        Self {
            key: key.into(),
            snapshot: OnceLock::new(),
            previous: None,
            state: OnceLock::new(),
            #[cfg(feature = "experimental-source-navigation")]
            navigation: OnceLock::new(),
            retired: AtomicBool::new(false),
            initialization: super::Mutex::new(None),
        }
    }

    pub(super) fn provide_snapshot(&self, snapshot: LoadedLspConfig, failed: bool) {
        let _ = self.snapshot.set((snapshot, failed));
    }

    pub(super) fn replacement(&self) -> Self {
        let previous = self
            .initialized_state()
            .and_then(|state| state.retained_configuration())
            .or_else(|| self.previous.clone());
        let mut replacement = Self::new(self.key.clone());
        replacement.previous = previous;
        replacement
    }

    pub(super) fn with_previous(
        key: ProjectContextKey,
        previous: Option<RetainedConfiguration>,
    ) -> Self {
        let mut context = Self::new(key);
        context.previous = previous;
        context
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
                    self.previous.as_ref(),
                    self.snapshot.get().is_some_and(|(_, failed)| *failed),
                )
            })
            .clone();
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
        #[cfg(feature = "experimental-source-navigation")]
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
        #[cfg(feature = "experimental-source-navigation")]
        if let Some(navigation) = self.navigation.get() {
            navigation.retire();
        }
        if let Some(state) = self.state.get() {
            state.retire_project_owner();
            #[cfg(feature = "experimental-source-navigation")]
            state.retire_module_links(super::super::ModuleLinkRetirement::Shutdown);
        }
    }

    #[cfg(feature = "experimental-source-navigation")]
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
        if self.retired.load(Ordering::Acquire) {
            navigation.retire();
            return None;
        }
        Some(navigation)
    }
}

#[cfg(feature = "experimental-source-navigation")]
impl Drop for ProjectContext {
    fn drop(&mut self) {
        self.mark_retired();
        if let Some(navigation) = self.navigation.get() {
            navigation.retire();
        }
    }
}
