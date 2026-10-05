//! Workspace-rooted native project startup.

use std::path::Path;

use super::{CorsaProjectClient, resolve_corsa_executable};

impl CorsaProjectClient {
    /// Start a Corsa project session rooted at an on-disk workspace.
    pub fn new_for_workspace(
        corsa_path: Option<&str>,
        workspace_root: &Path,
    ) -> Result<Self, String> {
        let workspace_root = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());
        let working_dir = workspace_root.to_string_lossy();
        let executable = resolve_corsa_executable(corsa_path, Some(working_dir.as_ref()))?;

        Self::spawn_initialized_client(
            executable.as_str(),
            workspace_root.clone(),
            Some(workspace_root),
            None,
        )
    }

    /// Use the generated configuration while retaining the complete workspace.
    pub(crate) fn new_for_workspace_config(
        corsa_path: Option<&str>,
        workspace_root: &Path,
        config_path: &Path,
    ) -> Result<Self, String> {
        let workspace_root = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());
        let working_dir = workspace_root.to_string_lossy();
        let executable = resolve_corsa_executable(corsa_path, Some(working_dir.as_ref()))?;
        Self::spawn_initialized_client_with_config(
            executable.as_str(),
            workspace_root.clone(),
            Some(workspace_root),
            None,
            Some(config_path),
        )
    }
}
