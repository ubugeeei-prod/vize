//! Refresh this editor's API from its acknowledged logical revision.

use crate::lsp_client::{CorsaProjectClient, lifecycle_setup::workspace_config_path};
use corsa::{
    api::{ApiDialect, ProjectSession},
    runtime::block_on,
};
use std::path::Path;
use vize_l0::{String, cstr};

impl CorsaProjectClient {
    pub(in crate::lsp_client) fn refresh_logical_attached_project(&mut self) -> Result<(), String> {
        if !self.project_session_attached_to_editor {
            return Ok(());
        }
        self.logical_api_reset_pending = true;
        let dialect = self
            .session
            .as_ref()
            .and_then(|session| session.client().dialect())
            .ok_or_else(|| cstr!("Attached editor snapshot dialect is unavailable"))?;
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
        #[cfg(test)]
        record_logical_refresh(self, "before-ack", dialect, &config);
        // Acknowledge every dirty retained identity and partial close before
        // acquiring the replacement snapshot on this same connection.
        self.editor_lsp_session()?.ready_document_uri(&uri)?;
        let previous = self
            .session
            .as_ref()
            .ok_or_else(|| cstr!("Attached editor snapshot owner is unavailable"))?
            .client();
        let client = match dialect {
            ApiDialect::SessionSnapshots => previous.clone(),
            ApiDialect::DerivedSnapshots => previous
                .fork_snapshot_lineage()
                .map_err(|error| cstr!("Cannot fork current editor snapshot lineage: {error}"))?,
        };
        let replacement = block_on(ProjectSession::open(
            client,
            crate::file_uri::path_to_typescript_name(&config).as_str(),
            None,
        ))
        .map_err(|error| cstr!("Cannot open current logical editor snapshot: {error}"))?;
        let requested = config
            .canonicalize()
            .map_err(|error| cstr!("Cannot inspect authored editor config: {error}"))?;
        let selected = Path::new(replacement.project().config_file_name.as_str())
            .canonicalize()
            .map_err(|error| cstr!("Cannot inspect selected logical editor config: {error}"))?;
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
        #[cfg(test)]
        record_logical_refresh(self, "published", dialect, &config);
        Ok(())
    }
}

#[cfg(test)]
fn record_logical_refresh(
    client: &CorsaProjectClient,
    phase: &str,
    dialect: ApiDialect,
    config: &Path,
) {
    use std::io::Write;

    let Some(capture) = std::env::var_os("VIZE_PRIVATE_OVERLAY_CAPTURE") else {
        return;
    };
    if std::env::var("VIZE_TEST_REQUIRE_TSGO").as_deref() != Ok("1") {
        return;
    }
    let packet = serde_json::json!({
        "phase":phase,"dialect":dialect.as_str(),"config":config,
        "processId":std::process::id(),"testThread":std::thread::current().name(),
        "documents":client.document_texts,
        "snapshot":client.session.as_ref().map(|session| &session.snapshot().handle),
        "pending":client.logical_api_reset_pending,
    });
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(Path::new(&capture).join("shared-api-revisions.ndjson"))
        .expect("open the existing private native capture");
    writeln!(file, "{packet}").expect("retain the whole native API revision before assertions");
    file.flush().expect("flush the native API revision capture");
}
