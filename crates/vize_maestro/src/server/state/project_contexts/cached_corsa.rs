//! Read-only lookup of already-routed native type providers.

use super::{ServerState, Weak};
use tower_lsp::lsp_types::Url;

impl ServerState {
    /// Preserve native DOM quick info without discovering a root, loading
    /// configuration, or starting a provider on the cold markup path.
    pub(crate) fn has_cached_corsa_bridge_for_document(&self, uri: &Url) -> bool {
        if self.has_corsa_bridge() {
            return true;
        }
        if let Some(owner) = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(Weak::upgrade)
        {
            return owner.has_cached_corsa_bridge_for_document(uri);
        }
        let Some(key) = self
            .project_contexts
            .routes
            .get(uri)
            .map(|route| route.value().clone())
        else {
            return false;
        };
        let context = self.project_contexts.contexts.lock().get(&key).cloned();
        context
            .and_then(|context| context.initialized_state())
            .is_some_and(|state| !state.project_context_retired() && state.has_corsa_bridge())
    }
}
