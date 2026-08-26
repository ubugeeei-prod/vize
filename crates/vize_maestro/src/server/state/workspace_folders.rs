//! Per-workspace-folder configuration contexts for multi-root sessions.
//!
//! `initialize` may carry several `workspaceFolders` (#3240). The primary
//! root (rootUri, or the first folder) keeps driving the process-wide
//! configuration — LSP feature flags, the type-checker/Corsa session root,
//! formatting — matching the historical single-root behavior. Per-document
//! lint diagnostics however must not leak one folder's lint policy into
//! another folder's files, so each folder gets its own linter context here:
//! a folder that ships its own `vize.config.*` uses that config, and a
//! folder without one uses the built-in defaults (never a sibling folder's
//! config, which would make behavior depend on folder order). Documents
//! outside every folder fall back to the process-wide settings.

use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::{InitializeParams, Url, WorkspaceFolder, WorkspaceFoldersChangeEvent};
use vize_s0::config::{LintRuleOptions, LinterConfig};

use super::ServerState;

/// Linter context resolved for one workspace folder at registration time.
pub(super) struct WorkspaceFolderConfig {
    root: PathBuf,
    linter: LinterConfig,
    rule_options: LintRuleOptions,
}

impl WorkspaceFolderConfig {
    /// Load the folder's own `vize.config.*`; a folder without a config file
    /// gets the built-in defaults so contexts stay order-independent.
    fn load(root: PathBuf) -> Self {
        let (loaded, linter) = vize_s0::config::load_config_and_linter_with_source(Some(&root));
        if loaded.source_path.is_some() {
            let rule_options = vize_s0::config::load_linter_rule_options(Some(&root));
            Self {
                root,
                linter,
                rule_options,
            }
        } else {
            Self {
                root,
                linter: LinterConfig::default(),
                rule_options: LintRuleOptions::default(),
            }
        }
    }
}

impl ServerState {
    /// Resolve the primary workspace root from `initialize`: `rootUri` when
    /// present, otherwise the first workspace folder. This root keeps driving
    /// process-wide config, the type-checker/Corsa session, and formatting.
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
    pub(crate) fn set_workspace_folders(&self, roots: Vec<PathBuf>) {
        let contexts = roots.into_iter().map(WorkspaceFolderConfig::load).collect();
        *self.workspace_folder_configs.write() = contexts;
    }

    /// Load a context for every folder carried by `initialize` (#3240),
    /// keeping the wire types out of the request handler.
    pub(crate) fn apply_initialize_workspace_folders(&self, folders: Option<&[WorkspaceFolder]>) {
        self.set_workspace_folders(folder_roots(folders.unwrap_or_default()));
    }

    /// Apply a `workspace/didChangeWorkspaceFolders` event: removed roots
    /// drop their contexts, added roots load theirs.
    pub(crate) fn update_workspace_folders(&self, added: Vec<PathBuf>, removed: &[PathBuf]) {
        let mut contexts = self.workspace_folder_configs.write();
        contexts.retain(|context| !removed.contains(&context.root));
        contexts.extend(added.into_iter().map(WorkspaceFolderConfig::load));
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
    pub(crate) fn linter_settings_for_uri(&self, uri: &Url) -> (LinterConfig, LintRuleOptions) {
        if let Ok(path) = uri.to_file_path() {
            let contexts = self.workspace_folder_configs.read();
            if let Some(context) = deepest_enclosing_folder(&contexts, &path) {
                return (context.linter.clone(), context.rule_options.clone());
            }
        }
        (self.get_linter_config(), self.get_linter_rule_options())
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
mod tests {
    use tower_lsp::lsp_types::{Url, WorkspaceFolder, WorkspaceFoldersChangeEvent};
    use vize_s0::{config::LintRuleSeverity, cstr};

    use crate::server::ServerState;

    fn folder_with_config(
        parent: &std::path::Path,
        name: &str,
        config: &str,
    ) -> std::path::PathBuf {
        let dir = parent.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("vize.config.json"), config).unwrap();
        dir
    }

    #[test]
    fn documents_resolve_their_own_folder_config_regardless_of_order() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let parent =
            std::env::temp_dir().join(cstr!("vize-folder-configs-{}-{nonce}", std::process::id()));
        let strict = parent.join("strict-root");
        std::fs::create_dir_all(&strict).unwrap();
        let relaxed = folder_with_config(
            &parent,
            "relaxed-root",
            r#"{ "linter": { "rules": { "vue/require-v-for-key": "off" } } }"#,
        );

        for roots in [
            vec![strict.clone(), relaxed.clone()],
            vec![relaxed.clone(), strict.clone()],
        ] {
            let state = ServerState::new();
            state.set_workspace_folders(roots);

            let strict_uri = Url::from_file_path(strict.join("List.vue")).unwrap();
            let (strict_config, _) = state.linter_settings_for_uri(&strict_uri);
            assert_eq!(strict_config.rules.get("vue/require-v-for-key"), None);

            let relaxed_uri = Url::from_file_path(relaxed.join("List.vue")).unwrap();
            let (relaxed_config, _) = state.linter_settings_for_uri(&relaxed_uri);
            assert_eq!(
                relaxed_config.rules.get("vue/require-v-for-key"),
                Some(&LintRuleSeverity::Off),
            );
        }

        let _ = std::fs::remove_dir_all(parent);
    }

    #[test]
    fn removed_folders_drop_their_context_and_outside_documents_use_globals() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let parent =
            std::env::temp_dir().join(cstr!("vize-folder-removal-{}-{nonce}", std::process::id()));
        let relaxed = folder_with_config(
            &parent,
            "relaxed-root",
            r#"{ "linter": { "rules": { "vue/require-v-for-key": "off" } } }"#,
        );

        let state = ServerState::new();
        state.set_workspace_folders(vec![relaxed.clone()]);
        let uri = Url::from_file_path(relaxed.join("List.vue")).unwrap();
        let (config, _) = state.linter_settings_for_uri(&uri);
        assert_eq!(
            config.rules.get("vue/require-v-for-key"),
            Some(&LintRuleSeverity::Off),
        );

        state.update_workspace_folders(Vec::new(), std::slice::from_ref(&relaxed));
        let (config, _) = state.linter_settings_for_uri(&uri);
        assert_eq!(config.rules.get("vue/require-v-for-key"), None);

        let _ = std::fs::remove_dir_all(parent);
    }

    #[test]
    fn workspace_folder_changes_select_only_open_documents_under_changed_roots() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let parent = std::env::temp_dir().join(cstr!(
            "vize-folder-revalidation-{}-{nonce}",
            std::process::id()
        ));
        let added_root = parent.join("added");
        let untouched_root = parent.join("untouched");
        std::fs::create_dir_all(&added_root).unwrap();
        std::fs::create_dir_all(&untouched_root).unwrap();

        let affected = Url::from_file_path(added_root.join("Affected.vue")).unwrap();
        let untouched = Url::from_file_path(untouched_root.join("Untouched.vue")).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(affected.clone(), "<template />".into(), 1, "vue".into());
        state
            .documents
            .open(untouched, "<template />".into(), 1, "vue".into());

        let selected = state.apply_workspace_folders_change(&WorkspaceFoldersChangeEvent {
            added: vec![WorkspaceFolder {
                uri: Url::from_file_path(&added_root).unwrap(),
                name: "added".into(),
            }],
            removed: Vec::new(),
        });

        assert_eq!(selected, vec![affected]);
        let _ = std::fs::remove_dir_all(parent);
    }
}
