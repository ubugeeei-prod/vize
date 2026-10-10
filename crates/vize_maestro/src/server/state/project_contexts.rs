//! Project ownership is independent of request order; no process-wide config swaps.
#![expect(
    clippy::disallowed_types,
    reason = "isolated native owners share the authoritative editor document store"
)]

use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Weak,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
};
use tower_lsp::lsp_types::{ClientCapabilities, Url};
use vize_l0::FxHashMap;

use super::{LspFeatureConfig, ServerState};

#[derive(Default)]
pub(super) struct ProjectContexts {
    owner: RwLock<Option<Weak<ServerState>>>,
    retired: AtomicBool,
    shutting_down: AtomicBool,
    generation: AtomicU64,
    routing_changes: AtomicUsize,
    boundary: RwLock<Option<PathBuf>>,
    contexts: Mutex<FxHashMap<PathBuf, Arc<registry::ProjectContext>>>,
    routes: DashMap<Url, PathBuf>,
    initialization_options: RwLock<Option<serde_json::Value>>,
    client_capabilities: RwLock<ClientCapabilities>,
}

impl ServerState {
    pub(super) fn apply_project_path_identity(
        &self,
        directory: &Path,
        loaded: &vize_carton::config::LoadedProjectConfig,
    ) {
        *self.project_contexts.boundary.write() = Some(directory.to_path_buf());
        if let Some(root) = &loaded.project_root {
            self.set_workspace_root(root.clone());
        }
    }

    pub(super) fn retain_project_initialization_options(
        &self,
        options: Option<&serde_json::Value>,
    ) {
        *self.project_contexts.initialization_options.write() = options.cloned();
    }

    pub(super) fn retain_project_client_capabilities(&self, capabilities: &ClientCapabilities) {
        *self.project_contexts.client_capabilities.write() = capabilities.clone();
    }

    pub(crate) fn document_project_state(self: &Arc<Self>, uri: &Url) -> Option<Arc<ServerState>> {
        if self.project_contexts.shutting_down.load(Ordering::Acquire) {
            return None;
        }
        if let Some(owner) = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(Weak::upgrade)
        {
            return owner.document_project_state(uri).or(Some(owner));
        }
        loop {
            let context = self.document_project_context(uri)?;
            if let Some(state) = context.initialize(self) {
                return Some(state);
            }
        }
    }

    fn document_project_context(&self, uri: &Url) -> Option<Arc<registry::ProjectContext>> {
        if self.project_contexts.shutting_down.load(Ordering::Acquire) {
            return None;
        }
        let root = match self.project_contexts.routes.get(uri) {
            Some(root) => {
                let cached = self
                    .project_contexts
                    .contexts
                    .lock()
                    .get(root.value())
                    .cloned();
                if let Some(context) = cached {
                    drop(root);
                    return Some(context);
                }
                if self.is_primary_boundary(&root) {
                    return None;
                }
                root.clone()
            }
            None => {
                let path = uri.to_file_path().ok()?;
                let root = self.document_context_root(&path)?;
                self.project_contexts
                    .routes
                    .insert(uri.clone(), root.clone());
                root
            }
        };
        let context = {
            let mut contexts = self.project_contexts.contexts.lock();
            if !contexts.contains_key(&root) && self.is_primary_boundary(&root) {
                return None;
            }
            contexts
                .entry(root.clone())
                .or_insert_with(|| Arc::new(registry::ProjectContext::new(root)))
                .clone()
        };
        Some(context)
    }

    fn is_primary_boundary(&self, root: &Path) -> bool {
        if let Some(boundary) = self.project_contexts.boundary.read().as_deref() {
            boundary == root
        } else {
            self.workspace_root.read().as_deref() == Some(root)
        }
    }

    pub(super) fn project_routing_initialized(&self) -> bool {
        self.project_contexts.boundary.read().is_some()
    }

    fn document_context_root(&self, path: &Path) -> Option<PathBuf> {
        let roots = self.workspace_root_paths();
        paths::document_context_root(path, &roots)
            .or_else(|| self.project_contexts.boundary.read().clone())
    }

    fn new_project_state(self: &Arc<Self>, root: &Path) -> Arc<ServerState> {
        let mut state = ServerState::new();
        state.documents = self.documents.clone();
        *state.project_contexts.owner.write() = Some(Arc::downgrade(self));
        state.record_client_capabilities(&self.project_contexts.client_capabilities.read());
        state.set_workspace_root(root.to_path_buf());
        state.load_workspace_config(root);
        state.apply_lsp_initialization_options(
            self.project_contexts.initialization_options.read().as_ref(),
        );
        // Rebuild only editor-owned sources belonging to this context. Later
        // didOpen/didChange notifications update the same owner directly.
        for uri in state.documents.uris() {
            if uri
                .to_file_path()
                .is_ok_and(|path| self.document_context_root(&path).as_deref() == Some(root))
                && let Some(source) = state.documents.text(&uri)
            {
                state.update_virtual_docs(&uri, &source);
            }
        }
        Arc::new(state)
    }

    pub(crate) fn project_open_typecheck_dependents(self: &Arc<Self>, uri: &Url) -> Vec<Url> {
        if let Some(owner) = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(Weak::upgrade)
        {
            return owner.project_open_typecheck_dependents(uri);
        }
        let mut states = self.cached_project_states();
        states.push(self.clone());
        let mut dependents = states
            .iter()
            .filter(|state| !state.project_context_retired())
            .flat_map(|state| super::super::importers::open_typecheck_dependents(state, uri))
            .collect::<Vec<_>>();
        dependents.sort();
        dependents.dedup();
        dependents
    }

    pub(crate) fn cached_project_states(&self) -> Vec<Arc<ServerState>> {
        self.project_contexts
            .contexts
            .lock()
            .values()
            .filter_map(|context| context.initialized_state())
            .collect()
    }

    pub(crate) fn project_context_retired(&self) -> bool {
        self.project_contexts.retired.load(Ordering::Acquire)
    }

    fn retire_project_owner(&self) {
        self.project_contexts.retired.store(true, Ordering::Release);
        self.invalidate_corsa_request_environment();
        self.retire_corsa_configuration();
    }

    pub(crate) fn refresh_project_routes(&self) {
        let _change = self.project_routing_change();
        self.project_contexts.routes.clear();
        let roots = self.workspace_root_paths();
        let mut retired = Vec::new();
        self.project_contexts.contexts.lock().retain(|root, state| {
            let retained = roots.iter().any(|folder| root.starts_with(folder));
            if !retained {
                state.mark_retired();
                retired.push(state.clone());
            }
            retained
        });
        for previous in retired {
            previous.retire();
        }
    }

    pub(crate) fn retire_project_contexts(&self) {
        self.project_contexts
            .shutting_down
            .store(true, Ordering::Release);
        self.retire_project_owner();
        let contexts = std::mem::take(&mut *self.project_contexts.contexts.lock());
        for context in contexts.into_values() {
            context.retire();
        }
        self.project_contexts.routes.clear();
    }

    pub(crate) fn project_jsx_typecheck_enabled(&self) -> bool {
        let roots = self.workspace_root_paths();
        self.jsx_typecheck_enabled()
            || roots.len() > 1
            || roots
                .iter()
                .any(|root| capabilities::is_composite_project(root))
    }

    pub(crate) fn project_capability_features(&self) -> LspFeatureConfig {
        let mut features = self.lsp_features();
        let roots = self.workspace_root_paths();
        if roots.len() > 1
            || roots
                .iter()
                .any(|root| capabilities::is_composite_project(root))
        {
            let defaults = LspFeatureConfig {
                formatting: true,
                ..LspFeatureConfig::default()
            };
            capabilities::merge(&mut features, defaults);
        }
        if let Some(options) = self.project_contexts.initialization_options.read().clone()
            && let Ok(section) =
                serde_json::from_value::<super::features::LspConfigSection>(options)
        {
            section.apply_to(&mut features);
        }
        features
    }
}

fn document_root(path: &Path, workspace: &Path) -> PathBuf {
    let mut directory = path.parent().unwrap_or(workspace);
    while directory != workspace && directory.starts_with(workspace) {
        if directory.join("package.json").is_file()
            || directory.join("tsconfig.json").is_file()
            || directory.join("jsconfig.json").is_file()
            || std::fs::read_dir(directory).is_ok_and(|entries| {
                entries
                    .flatten()
                    .any(|entry| is_config_marker(&entry.path()))
            })
        {
            return directory.to_path_buf();
        }
        let Some(parent) = directory.parent() else {
            break;
        };
        directory = parent;
    }
    workspace.to_path_buf()
}

fn is_config_marker(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.starts_with("vize.config.")
                || name.starts_with("vite.config.")
                || (name.starts_with("tsconfig") && name.ends_with(".json"))
                || name == "package.json"
                || name == "jsconfig.json"
        })
}

#[cfg(test)]
mod async_tests;
mod asynchronous;
mod capabilities;
mod current;
mod paths;
mod pool;
mod registry;
#[cfg(test)]
mod tests;
mod watched;
