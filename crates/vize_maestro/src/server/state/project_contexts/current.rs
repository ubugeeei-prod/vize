//! Global requests target current owners; idle packages stay uninitialized.
use super::{Arc, Ordering, ProjectContexts, ServerState, Weak};

pub(crate) struct RoutingChange<'a>(&'a ProjectContexts);

impl Drop for RoutingChange<'_> {
    fn drop(&mut self) {
        self.0.generation.fetch_add(1, Ordering::AcqRel);
        self.0.routing_changes.fetch_sub(1, Ordering::AcqRel);
    }
}

impl ServerState {
    pub(crate) fn project_routing_change(&self) -> RoutingChange<'_> {
        self.project_contexts
            .routing_changes
            .fetch_add(1, Ordering::AcqRel);
        self.project_contexts
            .generation
            .fetch_add(1, Ordering::AcqRel);
        RoutingChange(&self.project_contexts)
    }

    pub(crate) fn stable_project_routing_generation(&self) -> Option<u64> {
        if let Some(owner) = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(Weak::upgrade)
        {
            return owner.stable_project_routing_generation();
        }
        if self
            .project_contexts
            .routing_changes
            .load(Ordering::Acquire)
            != 0
        {
            return None;
        }
        let generation = self.project_routing_generation();
        (self
            .project_contexts
            .routing_changes
            .load(Ordering::Acquire)
            == 0)
            .then_some(generation)
    }

    pub(crate) fn project_routing_shutting_down(&self) -> bool {
        if let Some(owner) = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(Weak::upgrade)
        {
            return owner.project_routing_shutting_down();
        }
        self.project_contexts.shutting_down.load(Ordering::Acquire)
    }

    pub(crate) fn project_routing_generation(&self) -> u64 {
        if let Some(owner) = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(Weak::upgrade)
        {
            return owner.project_routing_generation();
        }
        self.project_contexts.generation.load(Ordering::Acquire)
    }

    pub(crate) fn current_primary_project_state(self: &Arc<Self>) -> Arc<Self> {
        if self.project_routing_shutting_down() {
            return self.clone();
        }
        if let Some(owner) = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(Weak::upgrade)
        {
            return owner.current_primary_project_state();
        }
        loop {
            let root = self
                .project_contexts
                .boundary
                .read()
                .clone()
                .or_else(|| self.get_workspace_root());
            let context =
                root.and_then(|root| self.project_contexts.contexts.lock().get(&root).cloned());
            let Some(context) = context else {
                return self.clone();
            };
            if let Some(state) = context.initialize(self) {
                return state;
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn current_project_states(self: &Arc<Self>) -> Vec<Arc<Self>> {
        if let Some(owner) = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(Weak::upgrade)
        {
            return owner.current_project_states();
        }
        let primary = self.current_primary_project_state();
        let mut states = self.cached_project_states();
        if !states.iter().any(|state| Arc::ptr_eq(state, &primary)) {
            states.push(primary);
        }
        states.retain(|state| !state.project_context_retired());
        states
    }
}

#[cfg(test)]
mod tests {
    use super::{Arc, ServerState};
    use tower_lsp::lsp_types::{FileChangeType, FileEvent, Url};

    #[test]
    fn primary_replacement_is_lazy_unique_and_respects_explicit_options() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("vize.config.json");
        std::fs::write(&config, r#"{"lsp":{"fileRename":false}}"#).unwrap();
        let owner = Arc::new(ServerState::new());
        owner.set_workspace_root(root.path().to_path_buf());
        owner.load_workspace_config(root.path());
        owner
            .apply_lsp_initialization_options(Some(&serde_json::json!({"workspaceSymbols":false})));
        assert!(Arc::ptr_eq(&owner, &owner.current_primary_project_state()));
        let generation = owner.project_routing_generation();
        std::fs::write(
            &config,
            r#"{"lsp":{"fileRename":true,"workspaceSymbols":true}}"#,
        )
        .unwrap();
        owner.observe_project_config_events(&[FileEvent {
            uri: Url::from_file_path(&config).unwrap(),
            typ: FileChangeType::CHANGED,
        }]);
        assert!(owner.project_routing_generation() > generation);
        assert!(owner.cached_project_states().is_empty());
        let current = owner.current_primary_project_state();
        assert!(!Arc::ptr_eq(&owner, &current));
        assert!(!current.project_context_retired());
        assert!(current.lsp_features().file_rename);
        assert!(!current.lsp_features().workspace_symbols);
        assert!(Arc::ptr_eq(
            &current,
            &owner.current_primary_project_state()
        ));
        let states = owner.current_project_states();
        assert_eq!(states.len(), 1);
        assert!(Arc::ptr_eq(&states[0], &current));
        owner.retire_project_contexts();
        assert!(owner.cached_project_states().is_empty());
        assert!(Arc::ptr_eq(&owner, &owner.current_primary_project_state()));
    }
}
