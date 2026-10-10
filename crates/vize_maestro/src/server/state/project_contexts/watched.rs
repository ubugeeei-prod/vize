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
        self.project_contexts.routes.clear();
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
                    .filter(|path| affected.iter().any(|root| path.starts_with(root)))
                    .map(|_| (document.key().clone(), document.value().version))
            })
            .collect();
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
        affected_documents
    }
}
