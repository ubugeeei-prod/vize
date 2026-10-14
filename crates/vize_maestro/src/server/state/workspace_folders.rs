//! Per-workspace-folder configuration contexts for multi-root sessions.
//!
//! `initialize` may carry several `workspaceFolders` (#3240). The primary
//! root (rootUri, or the first folder) initializes the primary owner. Native
//! document requests use isolated lazy project owners, while capabilities
//! cover the session's composite roots. Each folder gets its own linter context:
//! a folder that ships its own `vize.config.*` uses that config, and a
//! folder without one uses the built-in defaults (never a sibling folder's
//! config, which would make behavior depend on folder order). Documents
//! outside every folder fall back to the process-wide settings.

use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::{InitializeParams, Url, WorkspaceFolder, WorkspaceFoldersChangeEvent};
use vize_carton::config::matcher::{LintPlanScope, ProjectIgnoreSet};
use vize_l0::config::{
    ConfigDocument, ConfigLintRuleOptions, LinterConfig, LinterConfigPlanWithConfigRuleOptions,
    LinterFeatureFlags,
};

use super::ServerState;
mod config;

/// Linter context resolved for one workspace folder at registration time.
pub(super) struct WorkspaceFolderConfig {
    root: PathBuf,
    plan: LinterConfigPlanWithConfigRuleOptions,
    scopes: Vec<LintPlanScope>,
    global_ignores: Vec<LintPlanScope>,
    project_ignores: Option<ProjectIgnoreSet>,
    features: LinterFeatureFlags,
}

impl WorkspaceFolderConfig {
    pub(super) fn load(root: PathBuf) -> Self {
        let loaded = vize_carton::config::load_project_config_with_source(Some(&root));
        Self::from_project(
            root,
            &loaded.document,
            loaded.source_path.as_deref(),
            loaded.project_root.as_deref(),
        )
    }

    fn linter_for_path(
        &self,
        path: &Path,
    ) -> Option<(LinterConfig, ConfigLintRuleOptions, LinterFeatureFlags)> {
        if self
            .project_ignores
            .as_ref()
            .is_some_and(|ignores| ignores.is_ignored(path))
            || self.global_ignores.iter().any(|scope| scope.ignores(path))
        {
            return None;
        }
        let matching = self
            .scopes
            .iter()
            .enumerate()
            .filter_map(|(index, scope)| scope.matches(path).then_some(index))
            .collect::<Vec<_>>();
        let resolved = self.plan.resolve_matching_entries(&matching);
        Some((resolved.config, resolved.rule_options, self.features))
    }
}

impl ServerState {
    /// Set the workspace root path.
    #[cfg(feature = "native")]
    pub fn set_workspace_root(&self, path: PathBuf) {
        let _change = self.corsa_environment_change();
        #[cfg(feature = "experimental-source-navigation")]
        self.update_module_link_context(None, || {
            *self.workspace_root.write() = Some(path);
        });
        #[cfg(not(feature = "experimental-source-navigation"))]
        {
            *self.workspace_root.write() = Some(path);
        }
        self.lint_hover_cache.clear();
        self.package_route_resolver.lock().clear();
        self.global_component_references.invalidate();
        // Invalidate batch cache when workspace changes
        self.batch_cache.invalidate();
        // Overlays shadow files resolved relative to the workspace root, so a
        // new root retargets them even though no document changed.
        self.corsa_overlays.invalidate();
        self.retire_corsa_configuration();
    }

    /// Get the workspace root path.
    #[cfg(feature = "native")]
    pub fn get_workspace_root(&self) -> Option<PathBuf> {
        self.workspace_root.read().clone()
    }

    /// Capture every registered editor folder without rescanning its sources.
    #[cfg(feature = "native")]
    pub(crate) fn workspace_root_paths(&self) -> Vec<PathBuf> {
        let mut roots = self
            .workspace_folder_configs
            .read()
            .iter()
            .map(|context| context.root.clone())
            .collect::<Vec<_>>();
        if let Some(primary) = self.get_workspace_root()
            && !roots.iter().any(|root| primary.starts_with(root))
        {
            roots.push(primary);
        }
        roots
    }

    /// Resolve the primary workspace root from `initialize`: `rootUri` when
    /// present, otherwise the first workspace folder.
    pub(crate) fn primary_workspace_path(&self, params: &InitializeParams) -> Option<PathBuf> {
        params
            .root_uri
            .as_ref()
            .and_then(|u| u.to_file_path().ok())
            .or_else(|| {
                params
                    .workspace_folders
                    .as_ref()
                    .and_then(|f| f.first())
                    .and_then(|f| f.uri.to_file_path().ok())
            })
    }

    /// Replace the workspace-folder contexts with the folders sent by
    /// `initialize`.
    #[cfg(test)]
    pub(crate) fn set_workspace_folders(&self, roots: Vec<PathBuf>) {
        self.replace_workspace_folders(roots, false);
    }

    fn replace_workspace_folders(&self, roots: Vec<PathBuf>, reuse_loaded: bool) {
        #[cfg(feature = "native")]
        let _change = self.corsa_environment_change();
        let mut current = self.workspace_folder_configs.write();
        let mut existing = std::mem::take(&mut *current);
        *current = roots
            .into_iter()
            .map(|root| {
                existing
                    .iter()
                    .position(|context| reuse_loaded && context.root == root)
                    .map_or_else(
                        || self.load_folder_context(root, reuse_loaded),
                        |index| existing.swap_remove(index),
                    )
            })
            .collect();
    }

    fn load_folder_context(&self, root: PathBuf, deferred: bool) -> WorkspaceFolderConfig {
        #[cfg(feature = "native")]
        if deferred && self.project_routing_initialized() {
            return WorkspaceFolderConfig::deferred(root);
        }
        #[cfg(not(feature = "native"))]
        let _ = deferred;
        WorkspaceFolderConfig::load(root)
    }

    /// Load a context for every folder carried by `initialize` (#3240),
    /// keeping the wire types out of the request handler.
    pub(crate) fn apply_initialize_workspace_folders(
        &self,
        folders: Option<&[WorkspaceFolder]>,
        primary: Option<&Path>,
    ) {
        let mut roots = folder_roots(folders.unwrap_or_default());
        if roots.is_empty()
            && let Some(primary) = primary
        {
            roots.push(primary.to_path_buf());
        }
        self.replace_workspace_folders(roots, true);
    }

    /// Apply a `workspace/didChangeWorkspaceFolders` event: removed roots
    /// drop their contexts, added roots load theirs.
    pub(crate) fn update_workspace_folders(&self, added: Vec<PathBuf>, removed: &[PathBuf]) {
        #[cfg(feature = "native")]
        let _change = self.corsa_environment_change();
        let mut contexts = self.workspace_folder_configs.write();
        contexts.retain(|context| !removed.contains(&context.root));
        contexts.extend(
            added
                .into_iter()
                .map(|root| self.load_folder_context(root, true)),
        );
    }

    /// Sync the contexts from a `didChangeWorkspaceFolders` event (#3240) and
    /// return the open documents whose enclosing folder context may have
    /// changed. The request handler republishes diagnostics for these URIs.
    pub(crate) fn apply_workspace_folders_change(
        &self,
        event: &WorkspaceFoldersChangeEvent,
    ) -> Vec<Url> {
        let added = folder_roots(&event.added);
        let removed = folder_roots(&event.removed);
        let mut changed_roots = added.clone();
        changed_roots.extend(removed.iter().cloned());

        self.update_workspace_folders(added, &removed);

        let mut affected = self
            .documents
            .uris()
            .into_iter()
            .filter(|uri| {
                uri.to_file_path()
                    .is_ok_and(|path| changed_roots.iter().any(|root| path.starts_with(root)))
            })
            .collect::<Vec<_>>();
        affected.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        affected
    }

    /// Linter settings for a document: its deepest enclosing workspace folder
    /// wins; documents outside every folder use the process-wide settings.
    pub(crate) fn linter_settings_for_uri(
        &self,
        uri: &Url,
    ) -> Option<(LinterConfig, ConfigLintRuleOptions, LinterFeatureFlags)> {
        if let Ok(path) = uri.to_file_path() {
            let contexts = self.workspace_folder_configs.read();
            if let Some(context) = deepest_enclosing_folder(&contexts, &path) {
                return context.linter_for_path(&path);
            }
        }
        let (config, features) = self.get_linter_config_and_features();
        Some((config, self.get_linter_rule_options(), features))
    }
}

/// Filesystem roots of the given workspace folders, dropping non-`file:` URIs.
fn folder_roots(folders: &[WorkspaceFolder]) -> Vec<PathBuf> {
    folders
        .iter()
        .filter_map(|folder| folder.uri.to_file_path().ok())
        .collect()
}

fn deepest_enclosing_folder<'a>(
    contexts: &'a [WorkspaceFolderConfig],
    path: &Path,
) -> Option<&'a WorkspaceFolderConfig> {
    contexts
        .iter()
        .filter(|context| path.starts_with(&context.root))
        .max_by_key(|context| context.root.components().count())
}

#[cfg(test)]
#[path = "workspace_folders/tests.rs"]
mod tests;
