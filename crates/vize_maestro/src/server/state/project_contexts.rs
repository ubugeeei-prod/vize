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
use vize_l0::{FxHashMap, FxHashSet};

use super::{
    LspFeatureConfig, ServerState,
    config::host::{LoadedLspConfig, ObservedLspConfig, load_observed_lsp_config_snapshot},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct ProjectContextKey {
    pub(super) root: PathBuf,
    pub(super) tsconfig: Option<PathBuf>,
}

impl From<PathBuf> for ProjectContextKey {
    fn from(root: PathBuf) -> Self {
        Self {
            root,
            tsconfig: None,
        }
    }
}

#[derive(Clone)]
pub(super) struct RetainedConfiguration {
    authored: LoadedLspConfig,
    applied: LoadedLspConfig,
    compiler: Arc<vize_canon::CompilerConfigurationSnapshot>,
    observation: freshness::ConfigurationObservation,
}

impl RetainedConfiguration {
    fn same_authority(
        &self,
        previous: &LoadedLspConfig,
        compiler: &vize_canon::CompilerConfigurationSnapshot,
    ) -> bool {
        let (Some(current), Some(previous_policy)) = (
            self.applied.project.document.evaluated_policy(),
            previous.project.document.evaluated_policy(),
        ) else {
            return false;
        };
        // Failure identity is host-only notification metadata, never policy or
        // compiler authority. The complete evaluated value remains authoritative.
        current == previous_policy
            && self.applied.source_path == previous.source_path
            && self.applied.project.source_path == previous.project.source_path
            && self.applied.project.project_root == previous.project.project_root
            && self.applied.project.dependencies == previous.project.dependencies
            && self.compiler.same_authority(compiler)
    }
}

#[derive(Default)]
pub(super) struct ProjectContexts {
    evaluation_session: Arc<super::config::host::ConfigEvaluationSession>,
    owner: RwLock<Option<Weak<ServerState>>>,
    retired: AtomicBool,
    shutting_down: AtomicBool,
    generation: AtomicU64,
    routing_changes: AtomicUsize,
    boundary: RwLock<Option<PathBuf>>,
    pub(super) config_dependencies: RwLock<Vec<PathBuf>>,
    pub(super) key: RwLock<Option<ProjectContextKey>>,
    pub(super) loaded_config: RwLock<Option<LoadedLspConfig>>,
    pub(super) applied_config: RwLock<Option<LoadedLspConfig>>,
    pub(super) compiler_configuration:
        RwLock<Option<Arc<vize_canon::CompilerConfigurationSnapshot>>>,
    pub(super) config_reload_failed: AtomicBool,
    pub(super) configuration_observation: RwLock<Option<freshness::ConfigurationObservation>>,
    configuration_attempts:
        Mutex<FxHashMap<ProjectContextKey, freshness::ConfigurationObservation>>,
    reference_ownership: Mutex<vize_canon::batch::TsconfigOwnershipCache>,
    contexts: Mutex<FxHashMap<ProjectContextKey, Arc<registry::ProjectContext>>>,
    reload_admission: Mutex<FxHashMap<PathBuf, Weak<futures::lock::Mutex<()>>>>,
    pending_config_reloads: Mutex<FxHashSet<ProjectContextKey>>,
    /// Saves and watched-file events so far, and the count each unobservable
    /// configuration failure was last evaluated at.
    retry_triggers: AtomicU64,
    unobservable_retries: Mutex<FxHashMap<ProjectContextKey, u64>>,
    routes: DashMap<Url, ProjectContextKey>,
    initialization_options: RwLock<Option<serde_json::Value>>,
    client_capabilities: RwLock<ClientCapabilities>,
}

impl ServerState {
    /// A save or watched-file event can repair a failure Vize cannot observe.
    pub(crate) fn observe_configuration_retry_trigger(&self) {
        let contexts = &self.project_contexts;
        contexts.retry_triggers.fetch_add(1, Ordering::AcqRel);
    }

    /// An unobservable failure is evaluated again once per save or watched-file
    /// event, never on ordinary requests (each attempt runs the Vite config).
    pub(super) fn unobservable_retry_due(&self, key: &ProjectContextKey) -> bool {
        let triggers = self.project_contexts.retry_triggers.load(Ordering::Acquire);
        let mut seen = self.project_contexts.unobservable_retries.lock();
        seen.insert(key.clone(), triggers).unwrap_or(0) < triggers
    }

    pub(super) fn configuration_evaluation_session(
        &self,
    ) -> Arc<super::config::host::ConfigEvaluationSession> {
        self.project_contexts.evaluation_session.clone()
    }

    pub(super) fn apply_project_path_identity(
        &self,
        directory: &Path,
        project_root: Option<&Path>,
    ) {
        *self.project_contexts.boundary.write() = Some(directory.to_path_buf());
        if let Some(root) = project_root {
            self.set_workspace_root(root.to_path_buf());
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
            if self.project_routing_shutting_down() {
                return None;
            }
            let context = self.document_project_context(uri);
            let state = match context {
                Some(context) => match context.initialize(self) {
                    Some(state) => state,
                    None => continue,
                },
                None => self.clone(),
            };
            if state.project_context_retired() {
                continue;
            }
            if let Some(reference) = self.document_reference_context(uri, &state) {
                if let Some(state) = reference.initialize(self) {
                    return Some(state);
                }
            } else {
                if state.project_context_retired() {
                    continue;
                }
                return (!Arc::ptr_eq(&state, self)).then_some(state);
            }
        }
    }

    fn document_project_context(&self, uri: &Url) -> Option<Arc<registry::ProjectContext>> {
        if self.project_contexts.shutting_down.load(Ordering::Acquire) {
            return None;
        }
        let root = match self
            .project_contexts
            .routes
            .get(uri)
            .map(|route| route.value().clone())
        {
            Some(root) => {
                #[cfg(test)]
                lock_order_tests::before_contexts_lock();
                let cached = self.project_contexts.contexts.lock().get(&root).cloned();
                if let Some(context) = cached {
                    return Some(context);
                }
                let root = ProjectContextKey::from(root.root);
                if self.is_primary_boundary(&root.root) {
                    return None;
                }
                root
            }
            None => {
                let path = uri.to_file_path().ok()?;
                let root = ProjectContextKey::from(self.document_context_root(&path)?);
                self.project_contexts
                    .routes
                    .insert(uri.clone(), root.clone());
                root
            }
        };
        let context = {
            let mut contexts = self.project_contexts.contexts.lock();
            if !contexts.contains_key(&root)
                && root.tsconfig.is_none()
                && self.is_primary_boundary(&root.root)
            {
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

    fn new_project_state(
        self: &Arc<Self>,
        key: &ProjectContextKey,
        snapshot: Option<&ObservedLspConfig>,
        snapshot_failed: bool,
        prepared: Option<&RetainedConfiguration>,
    ) -> Arc<ServerState> {
        let root = &key.root;
        let mut state = ServerState::new();
        state.project_contexts.evaluation_session = self.configuration_evaluation_session();
        state.documents = self.documents.clone();
        *state.project_contexts.owner.write() = Some(Arc::downgrade(self));
        *state.project_contexts.key.write() = Some(key.clone());
        state.record_client_capabilities(&self.project_contexts.client_capabilities.read());
        state.set_workspace_root(root.to_path_buf());
        if let Some(prepared) = prepared {
            // This pair was already validated for this complete context key.
            // Raw dependency files can change before lazy initialization.
            *state.project_contexts.compiler_configuration.write() =
                Some(prepared.compiler.clone());
            *state.project_contexts.configuration_observation.write() =
                Some(prepared.observation.clone());
            state.apply_retained_workspace_config(root, prepared.applied.clone());
            *state.project_contexts.loaded_config.write() = Some(prepared.authored.clone());
        } else {
            let loaded = snapshot.cloned().unwrap_or_else(|| {
                load_observed_lsp_config_snapshot(
                    Some(root),
                    &self.configuration_evaluation_session(),
                )
            });
            state.apply_observed_workspace_config_snapshot(root, loaded);
        }
        if snapshot_failed {
            state
                .project_contexts
                .config_reload_failed
                .store(true, Ordering::Release);
        }
        state.apply_lsp_initialization_options(
            self.project_contexts.initialization_options.read().as_ref(),
        );
        // Rebuild only editor-owned sources belonging to this context. Later
        // didOpen/didChange notifications update the same owner directly.
        for uri in state.documents.uris() {
            if uri
                .to_file_path()
                .is_ok_and(|path| self.document_belongs_to_context(&uri, &path, key))
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
        self.project_contexts
            .pending_config_reloads
            .lock()
            .retain(|key| roots.iter().any(|folder| key.root.starts_with(folder)));
        self.project_contexts
            .configuration_attempts
            .lock()
            .retain(|key, _| roots.iter().any(|folder| key.root.starts_with(folder)));
        let mut retired = Vec::new();
        self.project_contexts.contexts.lock().retain(|root, state| {
            let retained = roots.iter().any(|folder| root.root.starts_with(folder));
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
        let contexts = {
            let mut contexts = self.project_contexts.contexts.lock();
            self.project_contexts.pending_config_reloads.lock().clear();
            std::mem::take(&mut *contexts)
        };
        for context in contexts.into_values() {
            context.retire();
        }
        self.project_contexts.routes.clear();
        self.project_contexts.configuration_attempts.lock().clear();
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
    let mut package_root = None;
    let mut vite_root: Option<PathBuf> = None;
    while directory.starts_with(workspace) {
        let mut has_package = false;
        let mut has_vite = false;
        let mut has_project_config = false;
        if let Ok(entries) = std::fs::read_dir(directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                has_package |=
                    path.file_name().and_then(|name| name.to_str()) == Some("package.json");
                has_vite |= is_vite_config_marker(&path);
                has_project_config |= is_typescript_config_marker(&path);
            }
        }
        if has_project_config {
            if package_root.is_some()
                && let Some(vite_root) = vite_root.as_ref()
            {
                return vite_root.clone();
            }
            return directory.to_path_buf();
        }
        if vite_root.is_none() && package_root.is_none() && has_vite {
            vite_root = Some(directory.to_path_buf());
        }
        if package_root.is_none() && has_package {
            package_root = Some(directory.to_path_buf());
        }
        if directory == workspace {
            break;
        }
        let Some(parent) = directory.parent() else {
            break;
        };
        directory = parent;
    }
    vite_root
        .or(package_root)
        .unwrap_or_else(|| workspace.to_path_buf())
}

fn is_vite_config_marker(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("vite.config."))
}

fn is_typescript_config_marker(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            (name.starts_with("tsconfig") && name.ends_with(".json")) || name == "jsconfig.json"
        })
}

fn is_config_marker(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            is_vite_config_marker(path)
                || is_typescript_config_marker(path)
                || name == "package.json"
        })
}

#[cfg(test)]
mod async_tests;
mod asynchronous;
mod capabilities;
#[cfg(test)]
mod config_dependencies;
pub(super) mod current;
#[cfg(test)]
mod lock_order_tests;
#[cfg(feature = "experimental-source-navigation")]
mod navigation;
mod paths;
mod pool;
#[cfg(test)]
mod reference_tests;
mod references;
mod registry;
#[cfg(test)]
mod tests;
mod watched;

mod config_reload;
#[cfg(test)]
mod config_reload_tests;

pub(super) mod freshness;

#[cfg(test)]
mod freshness_tests;

#[cfg(test)]
mod config_post_read_tests;

mod topology;

#[cfg(test)]
mod topology_tests;
