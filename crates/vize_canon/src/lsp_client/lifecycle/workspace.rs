//! Workspace-rooted native project startup.

use std::path::Path;
use vize_l0::String;

use super::{CorsaProjectClient, resolve_corsa_executable};

impl CorsaProjectClient {
    /// Start the first editor owner directly on its complete Canon snapshot.
    /// Provider discovery and private storage retain the authored workspace's
    /// authority; only the native project's root and configuration change.
    pub(crate) fn new_for_editor_materialized_workspace(
        corsa_path: Option<&str>,
        authored_root: &Path,
        canon_root: &Path,
        config_path: &Path,
        changes: &crate::batch::virtual_project::MaterializedFileDelta,
    ) -> Result<Self, String> {
        Self::validate_initial_editor_project_delta(changes)?;
        let authored_root = authored_root
            .canonicalize()
            .unwrap_or_else(|_| authored_root.to_path_buf());
        let working_dir = authored_root.to_string_lossy();
        let executable = resolve_corsa_executable(corsa_path, Some(working_dir.as_ref()))?;
        let storage = std::sync::Arc::new(super::super::session_storage::SessionStorage::new(
            &authored_root,
        )?);
        let canon_root = canon_root
            .canonicalize()
            .unwrap_or_else(|_| canon_root.to_path_buf());
        let mut client = Self::spawn_initialized_client_with_transport(
            executable.as_str(),
            canon_root.clone(),
            Some(canon_root),
            Some(storage),
            Some(config_path),
            true,
        )?;
        client.finish_initial_materialized_project(changes)?;
        Ok(client)
    }

    /// Start a Corsa project session rooted at an on-disk workspace.
    pub fn new_for_workspace(
        corsa_path: Option<&str>,
        workspace_root: &Path,
    ) -> Result<Self, String> {
        Self::new_workspace_with_transport(corsa_path, workspace_root, false)
    }

    pub(crate) fn new_for_editor_workspace(
        corsa_path: Option<&str>,
        workspace_root: &Path,
    ) -> Result<Self, String> {
        Self::new_workspace_with_transport(corsa_path, workspace_root, true)
    }

    fn new_workspace_with_transport(
        corsa_path: Option<&str>,
        workspace_root: &Path,
        shared_editor_transport: bool,
    ) -> Result<Self, String> {
        let workspace_root = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());
        let working_dir = workspace_root.to_string_lossy();
        let executable = resolve_corsa_executable(corsa_path, Some(working_dir.as_ref()))?;

        if shared_editor_transport {
            Self::spawn_initialized_client_with_transport(
                executable.as_str(),
                workspace_root.clone(),
                Some(workspace_root),
                None,
                None,
                true,
            )
        } else {
            Self::spawn_initialized_client(
                executable.as_str(),
                workspace_root.clone(),
                Some(workspace_root),
                None,
            )
        }
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

#[cfg(test)]
#[cfg(target_os = "linux")]
mod initial_project_tests;
