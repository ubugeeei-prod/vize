//! An explicit last-close policy releases routing ownership, never live connections.

use super::{Arc, Ordering, ServerState};

impl ServerState {
    pub(in crate::server::state) fn set_closed_context_release(&self, selected: Option<bool>) {
        if let Some(selected) = selected {
            self.project_contexts
                .release_closed
                .store(selected, Ordering::Release);
        }
    }

    /// Called only after the existing close and importer restoration completed.
    /// Failed try-locks conservatively preserve cached owners for this attempt.
    pub(crate) fn release_closed_child_contexts(self: &Arc<Self>) -> usize {
        let root = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(super::Weak::upgrade)
            .unwrap_or_else(|| Arc::clone(self));
        if !root.project_contexts.release_closed.load(Ordering::Acquire)
            || root.project_routing_shutting_down()
            || !root.documents.is_empty()
        {
            return 0;
        }
        let removed = root
            .documents
            .try_while_closed(|| {
                // Match watched/config-worker lock order, keeping both guards
                // through removal so no in-flight reload loses its old authority.
                let mut contexts = root.project_contexts.contexts.try_lock()?;
                let pending = root.project_contexts.pending_config_reloads.try_lock()?;
                if !pending.is_empty() || root.project_routing_shutting_down() {
                    return None;
                }
                root.stable_project_routing_generation()?;
                if contexts
                    .keys()
                    .all(|key| root.is_primary_boundary(&key.root))
                {
                    return Some(Vec::new());
                }
                let _change = root.project_routing_change();
                let mut removed = Vec::new();
                contexts.retain(|key, context| {
                    if root.is_primary_boundary(&key.root) {
                        return true;
                    }
                    context.detach_closed();
                    removed.push(Arc::clone(context));
                    false
                });
                Some(removed)
            })
            .flatten();
        // Context/navigation/worker drop may wake or cancel work: all document
        // and registry guards have already been released at this boundary.
        removed.map_or(0, |removed| removed.len())
    }
}

#[cfg(test)]
mod tests;
