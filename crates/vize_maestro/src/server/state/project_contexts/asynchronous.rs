//! Foreground dispatch awaits one package-local initialization future.
use super::{Arc, ServerState, Url, Weak};

impl ServerState {
    pub(crate) async fn project_background<T: Send + 'static>(
        work: impl FnOnce() -> T + Send + 'static,
    ) -> Option<T> {
        super::pool::run(work).await
    }
    pub(crate) async fn document_project_state_async(
        self: &Arc<Self>,
        uri: &Url,
    ) -> Option<Arc<Self>> {
        let owner = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(Weak::upgrade);
        let root = owner.as_ref().unwrap_or(self);
        loop {
            let Some(context) = root.document_project_context(uri) else {
                return owner;
            };
            if let Some(state) = context.initialize_async(root).await {
                return Some(state);
            }
        }
    }

    pub(crate) async fn current_primary_project_state_async(self: &Arc<Self>) -> Arc<Self> {
        let root = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(Weak::upgrade)
            .unwrap_or_else(|| self.clone());
        if root.project_routing_shutting_down() {
            return root;
        }
        loop {
            let boundary = root
                .project_contexts
                .boundary
                .read()
                .clone()
                .or_else(|| root.get_workspace_root());
            let context = boundary.and_then(|boundary| {
                root.project_contexts
                    .contexts
                    .lock()
                    .get(&boundary)
                    .cloned()
            });
            let Some(context) = context else {
                return root;
            };
            if let Some(state) = context.initialize_async(&root).await {
                return state;
            }
            if root.project_routing_shutting_down() {
                return root;
            }
        }
    }

    pub(crate) async fn current_project_states_async(self: &Arc<Self>) -> Vec<Arc<Self>> {
        let root = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(Weak::upgrade)
            .unwrap_or_else(|| self.clone());
        if root.project_routing_shutting_down() {
            return Vec::new();
        }
        let primary = root.current_primary_project_state_async().await;
        let mut states = root.cached_project_states();
        if !states.iter().any(|state| Arc::ptr_eq(state, &primary)) {
            states.push(primary);
        }
        states.retain(|state| !state.project_context_retired());
        states
    }
}
