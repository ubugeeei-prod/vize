//! Native checks attest normalized options through the diagnosing process's API.

use super::EditorLspSession;
use crate::corsa_bridge::DiagnosingSnapshot;
use corsa::{api::ApiClient, runtime::block_on};
#[cfg(unix)]
use corsa_lsp::InitializeApiSessionRequest;
use corsa_lsp::InitializeApiSessionResult;
use serde_json::Value;
use std::path::Path;
use vize_l0::{String, cstr};

pub(super) enum ConfigurationError {
    Communication(String),
    Unconfigured,
    Changed,
    #[cfg(not(unix))]
    Unsupported,
}

pub(super) struct DiagnosingApi {
    client: ApiClient,
    pub(super) session: InitializeApiSessionResult,
}

impl EditorLspSession {
    #[cfg(unix)]
    pub(super) fn diagnosing_api(
        &mut self,
        uri: &str,
    ) -> Result<DiagnosingApi, ConfigurationError> {
        self.ready_document_uri(uri)
            .map_err(ConfigurationError::Communication)?;
        let session = block_on(
            self.client
                .request::<InitializeApiSessionRequest>(Default::default()),
        )
        .map_err(|error| {
            ConfigurationError::Communication(cstr!("cannot attach diagnosing API: {error}"))
        })?;
        let client = block_on(ApiClient::connect_pipe(session.pipe.as_str())).map_err(|error| {
            ConfigurationError::Communication(cstr!("cannot connect diagnosing API: {error}"))
        })?;
        Ok(DiagnosingApi { client, session })
    }

    #[cfg(not(unix))]
    pub(super) fn diagnosing_api(
        &mut self,
        _uri: &str,
    ) -> Result<DiagnosingApi, ConfigurationError> {
        // The pinned SDK exposes no real named-pipe attachment on this target.
        Err(ConfigurationError::Unsupported)
    }
}

impl DiagnosingApi {
    pub(super) fn observe(
        &self,
        uri: &str,
        config: &Path,
        admitted_options: &Value,
    ) -> Result<DiagnosingSnapshot, ConfigurationError> {
        let communication = |error| {
            ConfigurationError::Communication(cstr!("cannot observe diagnosing options: {error}"))
        };
        let snapshot =
            block_on(self.client.update_snapshot(Default::default())).map_err(communication)?;
        let result = (|| {
            let project =
                block_on(snapshot.get_default_project_for_file(
                    super::super::session::uri_document_identifier(uri),
                ))
                .map_err(communication)?
                .ok_or(ConfigurationError::Unconfigured)?;
            if Path::new(&project.config_file_name) != config {
                return Err(ConfigurationError::Unconfigured);
            }
            // Both endpoints serialize the actual core.CompilerOptions value.
            // Compare the complete value, including inherited paths/options.
            if &project.compiler_options != admitted_options {
                return Err(ConfigurationError::Changed);
            }
            Ok(DiagnosingSnapshot {
                handle: snapshot.handle.clone(),
                projects: snapshot.projects.clone(),
                changes: snapshot.changes.clone(),
                project,
            })
        })();
        let cleanup = block_on(snapshot.release()).map_err(communication);
        match (result, cleanup) {
            (Ok(observed), Ok(())) => Ok(observed),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }

    pub(super) fn close(&self, editor: &mut EditorLspSession) -> Result<(), ConfigurationError> {
        // The attached SDK reader keeps a full-duplex socket clone. Reap the
        // owning LSP first so the peer closes, then drain and join that reader.
        // observe() has already released each snapshot while the backend lived.
        let owner = editor.shutdown().map_err(ConfigurationError::Communication);
        let attachment = block_on(self.client.close()).map_err(|error| {
            ConfigurationError::Communication(cstr!("cannot close diagnosing API: {error}"))
        });
        match (owner, attachment) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }
}

#[cfg(all(test, unix))]
mod tests;
