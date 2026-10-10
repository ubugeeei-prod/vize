//! Package-local once initialization and retirement cover in-flight discovery.
#![expect(
    clippy::disallowed_types,
    reason = "registry cells retain their isolated native owner"
)]
use super::{Arc, AtomicBool, Ordering, PathBuf, ServerState};
use futures::{
    FutureExt,
    future::{BoxFuture, Shared},
};
use std::sync::OnceLock;
type Initialization = Shared<BoxFuture<'static, Option<Arc<ServerState>>>>;

pub(super) struct ProjectContext {
    root: PathBuf,
    state: OnceLock<Arc<ServerState>>,
    retired: AtomicBool,
    initialization: super::Mutex<Option<Initialization>>,
}

impl ProjectContext {
    pub(super) fn new(root: PathBuf) -> Self {
        Self {
            root,
            state: OnceLock::new(),
            retired: AtomicBool::new(false),
            initialization: super::Mutex::new(None),
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
            .get_or_init(|| owner.new_project_state(&self.root))
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
        if let Some(state) = self.state.get() {
            state
                .project_contexts
                .retired
                .store(true, Ordering::Release);
        }
    }

    pub(super) fn retire(&self) {
        self.mark_retired();
        if let Some(state) = self.state.get() {
            state.retire_project_owner();
            #[cfg(feature = "experimental-source-navigation")]
            state.retire_module_links(super::super::ModuleLinkRetirement::Shutdown);
        }
    }
}
