//! Retire changed owners and retain open documents needing fresh diagnostics.
use super::{Arc, Ordering, ServerState, Url, is_config_marker, paths::physical_path, registry};
use tower_lsp::lsp_types::FileEvent;

impl ServerState {
    pub(crate) fn observe_project_config_events(
        self: &Arc<Self>,
        changes: &[FileEvent],
    ) -> Vec<(Url, i32)> {
        if self.project_contexts.owner.read().is_some() {
            return Vec::new();
        }
        let mut roots = self.workspace_root_paths();
        if let Some(boundary) = self.project_contexts.boundary.read().clone()
            && !roots.contains(&boundary)
        {
            roots.push(boundary);
        }
        let physical_roots = roots
            .iter()
            .map(|root| physical_path(root))
            .collect::<Vec<_>>();
        let changed_paths = changes
            .iter()
            .filter_map(|event| event.uri.to_file_path().ok())
            .map(|path| physical_path(&path))
            .collect::<Vec<_>>();
        let marker_paths = changes
            .iter()
            .filter_map(|event| event.uri.to_file_path().ok())
            .filter(|path| !excluded_config_path(path, &roots))
            .map(|path| physical_path(&path))
            .filter(|path| is_config_marker(path) && !excluded_config_path(path, &physical_roots))
            .collect::<Vec<_>>();
        let config_paths = marker_paths.iter().collect::<Vec<_>>();
        let dependency_changed = |state: &ServerState| {
            state
                .project_contexts
                .config_dependencies
                .read()
                .iter()
                .any(|dependency| changed_paths.contains(&physical_path(dependency)))
        };
        let mut tracked = self
            .project_contexts
            .config_dependencies
            .read()
            .iter()
            .map(|path| physical_path(path))
            .collect::<Vec<_>>();
        for state in self.cached_project_states() {
            tracked.extend(
                state
                    .project_contexts
                    .config_dependencies
                    .read()
                    .iter()
                    .map(|path| physical_path(path)),
            );
        }
        let untracked_markers = config_paths
            .iter()
            .filter(|path| !tracked.contains(path))
            .copied()
            .collect::<Vec<_>>();
        let dependency_affected = self
            .cached_project_states()
            .iter()
            .any(|state| dependency_changed(state))
            || dependency_changed(self);
        if config_paths.is_empty() && !dependency_affected {
            return Vec::new();
        }
        let _change = self.project_routing_change();
        *self.project_contexts.reference_ownership.lock() = Default::default();
        let mut contexts = self.project_contexts.contexts.lock();
        let mut retired = Vec::new();
        let mut affected = contexts
            .iter()
            .filter(|(root, context)| {
                context
                    .initialized_state()
                    .is_some_and(|state| dependency_changed(&state))
                    || untracked_markers
                        .iter()
                        .any(|path| path.starts_with(physical_path(&root.root)))
            })
            .map(|(root, _)| root.clone())
            .collect::<Vec<_>>();
        let mut primary_retired = false;
        if let Some(root) = self
            .project_contexts
            .boundary
            .read()
            .clone()
            .or_else(|| self.get_workspace_root())
            && (dependency_changed(self)
                || config_paths
                    .iter()
                    .any(|path| path.parent() == Some(physical_path(&root).as_path())))
        {
            self.project_contexts.retired.store(true, Ordering::Release);
            primary_retired = true;
            if !affected.contains(&root.clone().into()) {
                affected.push(root.into());
            }
        }
        let base_roots = affected
            .iter()
            .filter(|key| key.tsconfig.is_none())
            .map(|key| key.root.clone())
            .collect::<Vec<_>>();
        let pending = contexts
            .iter()
            .filter(|(key, context)| {
                key.tsconfig.is_some()
                    && base_roots.contains(&key.root)
                    && context.initialized_state().is_none()
            })
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        affected.extend(pending);
        for root in &affected {
            let replacement = contexts
                .get(root)
                .map(|context| context.replacement())
                .unwrap_or_else(|| {
                    registry::ProjectContext::with_previous(
                        root.clone(),
                        self.retained_configuration(),
                    )
                });
            if let Some(previous) = contexts.insert(root.clone(), Arc::new(replacement)) {
                previous.mark_retired();
                retired.push(previous);
            }
        }
        drop(contexts);
        let affected_documents = self
            .documents
            .iter()
            .filter_map(|document| {
                document
                    .key()
                    .to_file_path()
                    .ok()
                    .map(|path| physical_path(&path))
                    .filter(|path| {
                        affected
                            .iter()
                            .any(|root| path.starts_with(physical_path(&root.root)))
                            || config_paths.iter().any(|config| {
                                config
                                    .parent()
                                    .is_some_and(|parent| path.starts_with(parent))
                            })
                    })
                    .map(|_| (document.key().clone(), document.value().version))
            })
            .collect::<Vec<_>>();
        // A newly created marker can move a document out of the primary
        // owner before any child context exists. Drop that old virtual view;
        // the watcher prepares the current buffer in the new owner below.
        #[cfg(test)]
        super::lock_order_tests::before_route_changes();
        let moved_primary_documents = affected_documents
            .iter()
            .filter(|(uri, _)| {
                self.project_contexts
                    .routes
                    .get(uri)
                    .is_some_and(|previous| {
                        previous.tsconfig.is_none()
                            && self.is_primary_boundary(&previous.root)
                            && uri
                                .to_file_path()
                                .ok()
                                .and_then(|path| self.document_context_root(&path))
                                .is_some_and(|current| current != previous.root)
                    })
            })
            .map(|(uri, _)| uri.clone())
            .collect::<Vec<_>>();
        self.project_contexts.routes.clear();
        if primary_retired {
            self.retire_project_owner();
        }
        for previous in retired {
            previous.retire();
        }
        for uri in moved_primary_documents {
            self.remove_virtual_docs(&uri);
        }
        affected_documents
    }
}

fn excluded_config_path(path: &std::path::Path, roots: &[std::path::PathBuf]) -> bool {
    let relative = roots
        .iter()
        .filter_map(|root| {
            path.strip_prefix(root)
                .ok()
                .map(|relative| (root, relative))
        })
        .max_by_key(|(root, _)| root.components().count())
        .map(|(_, relative)| relative);
    relative.is_some_and(|relative| {
        relative.components().any(|part| {
            matches!(part, std::path::Component::Normal(name) if matches!(
                name.to_str(),
                Some("node_modules" | ".git" | "target" | "dist" | "build")
            ))
        })
    })
}
