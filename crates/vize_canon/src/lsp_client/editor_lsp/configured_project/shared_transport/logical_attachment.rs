//! Refresh this editor's API from its acknowledged logical revision.

use crate::lsp_client::{CorsaProjectClient, lifecycle_setup::workspace_config_path};
use corsa::{
    api::{ApiDialect, ProjectSession},
    runtime::block_on,
};
use std::path::Path;
use vize_carton::path::normalize_windows_verbatim_path;
use vize_l0::{String, cstr};

impl CorsaProjectClient {
    pub(in crate::lsp_client) fn refresh_logical_attached_project(&mut self) -> Result<(), String> {
        if !self.project_session_attached_to_editor
            || !self.session.as_ref().is_some_and(|session| {
                session.client().dialect() == Some(ApiDialect::DerivedSnapshots)
            })
        {
            return Ok(());
        }
        let uri = self
            .document_texts
            .keys()
            .min()
            .cloned()
            .ok_or_else(|| cstr!("Logical API refresh requires a retained document barrier"))?;
        let config = self
            .explicit_project_config
            .clone()
            .unwrap_or_else(|| workspace_config_path(&self.project_root));
        // This response acknowledges every dirty retained identity, including
        // partial closes, before a fresh API lineage acquires its initial head.
        self.editor_lsp_session()?.ready_document_uri(&uri)?;
        let fork = self
            .session
            .as_ref()
            .ok_or_else(|| cstr!("Attached editor snapshot owner is unavailable"))?
            .client()
            .fork_snapshot_lineage()
            .map_err(|error| cstr!("Cannot fork current editor snapshot lineage: {error}"))?;
        let replacement = block_on(ProjectSession::open(
            fork,
            crate::file_uri::path_to_typescript_name(&config).as_ref(),
            None,
        ))
        .map_err(|error| cstr!("Cannot open current logical editor snapshot: {error}"))?;
        let requested = normalize_windows_verbatim_path(
            config
                .canonicalize()
                .map_err(|error| cstr!("Cannot inspect authored editor config: {error}"))?,
        );
        let selected = normalize_windows_verbatim_path(
            Path::new(replacement.project().config_file_name.as_str())
                .canonicalize()
                .map_err(|error| cstr!("Cannot inspect selected logical editor config: {error}"))?,
        );
        if selected != requested {
            return Err(cstr!(
                "Current logical editor snapshot did not retain its exact authored config"
            ));
        }
        let api = self
            .editor_lsp
            .as_mut()
            .and_then(|editor| editor.configured_api.as_mut())
            .ok_or_else(|| cstr!("Attached editor API connection is unavailable"))?;
        // Nothing fallible remains: publish the session and its retained API
        // client together. Both forks use the same connection and native owner.
        let previous_client = std::mem::replace(&mut api.client, replacement.client().clone());
        let previous_session = self.session.replace(replacement);
        self.logical_api_reset_pending = false;
        // Ordinary drops preserve any independently retained old snapshot. An
        // explicit close would close every fork's shared transport and is wrong.
        drop(previous_session);
        drop(previous_client);
        Ok(())
    }
}
