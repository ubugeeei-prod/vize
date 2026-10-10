//! Retire changed owners and retain open documents needing fresh diagnostics.
use super::{Arc, Ordering, ServerState, Url, is_config_marker, registry};
use tower_lsp::lsp_types::FileEvent;

impl ServerState {
    pub(crate) fn observe_project_config_events(
        self: &Arc<Self>,
        changes: &[FileEvent],
    ) -> Vec<(Url, i32)> {
        if self.project_contexts.owner.read().is_some() {
            return Vec::new();
        }
        let config_paths = changes
            .iter()
            .filter_map(|event| event.uri.to_file_path().ok())
            .filter(|path| is_config_marker(path))
            .collect::<Vec<_>>();
        if config_paths.is_empty() {
            return Vec::new();
        }
        let _change = self.project_routing_change();
        let mut contexts = self.project_contexts.contexts.lock();
        let mut retired = Vec::new();
        let mut affected = contexts
            .keys()
            .filter(|root| config_paths.iter().any(|path| path.starts_with(root)))
            .cloned()
            .collect::<Vec<_>>();
        let mut primary_retired = false;
        if let Some(root) = self
            .project_contexts
            .boundary
            .read()
            .clone()
            .or_else(|| self.get_workspace_root())
            && config_paths
                .iter()
                .any(|path| path.parent() == Some(root.as_path()))
        {
            self.project_contexts.retired.store(true, Ordering::Release);
            primary_retired = true;
            if !affected.contains(&root) {
                affected.push(root);
            }
        }
        let affected_documents = self
            .documents
            .iter()
            .filter_map(|document| {
                document
                    .key()
                    .to_file_path()
                    .ok()
                    .filter(|path| {
                        affected.iter().any(|root| path.starts_with(root))
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
        let moved_primary_documents = affected_documents
            .iter()
            .filter(|(uri, _)| {
                self.project_contexts
                    .routes
                    .get(uri)
                    .is_some_and(|previous| {
                        self.is_primary_boundary(&previous)
                            && uri
                                .to_file_path()
                                .ok()
                                .and_then(|path| self.document_context_root(&path))
                                .is_some_and(|current| current != *previous)
                    })
            })
            .map(|(uri, _)| uri.clone())
            .collect::<Vec<_>>();
        self.project_contexts.routes.clear();
        for root in affected {
            let replacement = Arc::new(registry::ProjectContext::new(root.clone()));
            if let Some(previous) = contexts.insert(root, replacement) {
                previous.mark_retired();
                retired.push(previous);
            }
        }
        drop(contexts);
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
