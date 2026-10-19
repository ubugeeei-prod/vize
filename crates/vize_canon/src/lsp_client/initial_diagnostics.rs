//! An owned, read-only API request detached from the mutable client lane.
#![expect(clippy::disallowed_types, reason = "SDK leases use std Arc")]

use std::{path::PathBuf, sync::Arc};

use corsa::{
    CorsaError,
    api::{ApiClient, CapabilitiesResponse, ManagedSnapshot, ProjectResponse},
    runtime::block_on,
};
use lsp_types::Diagnostic;
use vize_l0::{FxHashMap, String, cstr};

use super::{
    CorsaProjectClient,
    diagnostics_api::{document_identifier_uri, flatten_file_diagnostics},
    session::uri_document_identifier,
    utils::remap_serialized_uris,
};

/// The Arc owns the actual remote snapshot, not merely its opaque handle.
/// Project metadata and URI translations belong to that same captured revision.
pub(crate) struct InitialDiagnosticsRead {
    client: ApiClient,
    snapshot: Arc<ManagedSnapshot>,
    project: ProjectResponse,
    capabilities: Arc<CapabilitiesResponse>,
    project_root: PathBuf,
    explicit_config: Option<PathBuf>,
    uri: String,
    document_uri: String,
    text: Option<String>,
    external_document_uris: FxHashMap<String, String>,
    file_supported: bool,
    project_supported: bool,
}

impl CorsaProjectClient {
    pub(crate) fn prepare_initial_diagnostics_read(
        &mut self,
        uri: &str,
    ) -> Result<Option<InitialDiagnosticsRead>, String> {
        super::diagnostics::virtual_overlay_diagnostics::ensure_materialized_project(
            self,
            std::iter::once(uri),
        )?;
        if !self.can_use_api_for_uri(uri)
            || (!self.supports_file_diagnostics_api() && !self.supports_project_diagnostics_api())
        {
            return Ok(None);
        }
        let document_uri = self.session_document_uri(uri);
        let session = self.project_session()?;
        let read = InitialDiagnosticsRead {
            client: session.client().clone(),
            snapshot: session.snapshot_lease(),
            project: session.project().clone(),
            capabilities: Arc::clone(&self.capabilities),
            project_root: self.project_root.clone(),
            explicit_config: self.explicit_project_config.clone(),
            uri: uri.into(),
            document_uri,
            text: self.document_texts.get(uri).cloned(),
            external_document_uris: self.external_document_uris.clone(),
            file_supported: self.supports_file_diagnostics_api(),
            project_supported: self.supports_project_diagnostics_api(),
        };
        read.trace_route("capture", "prepared", None);
        Ok(Some(read))
    }

    pub(crate) fn replace_initial_diagnostics_cache(
        &mut self,
        uri: &str,
        replacement: Option<Vec<Diagnostic>>,
    ) -> Option<Vec<Diagnostic>> {
        match replacement {
            Some(diagnostics) => self.diagnostics.insert(uri.into(), diagnostics),
            None => self.diagnostics.remove(uri),
        }
    }

    /// This check performs no transport initialization, source read or retarget.
    pub(crate) fn initial_diagnostics_read_is_current(
        &self,
        read: &InitialDiagnosticsRead,
    ) -> bool {
        !self.closed
            && self.session.as_ref().is_some_and(|session| {
                Arc::ptr_eq(&read.snapshot, &session.snapshot_lease())
                    && session.project() == &read.project
            })
            && Arc::ptr_eq(&self.capabilities, &read.capabilities)
            && self.project_root == read.project_root
            && self.explicit_project_config == read.explicit_config
            && self.document_texts.get(read.uri.as_str()) == read.text.as_ref()
            && self.external_document_uris == read.external_document_uris
            && self.session_document_uris.get(read.uri.as_str()) == Some(&read.document_uri)
    }
}

impl InitialDiagnosticsRead {
    pub(crate) fn uri(&self) -> &str {
        &self.uri
    }

    /// The worker owns `self` until every entered RPC returns, including when
    /// its awaiting caller cancels. Results never touch the mutable client.
    pub(crate) fn run(&self) -> Result<Option<Vec<Diagnostic>>, String> {
        if self.file_supported {
            self.trace_route("file", "entered", None);
            match block_on(self.client.get_diagnostics_for_file(
                self.snapshot.handle.clone(),
                self.project.id.clone(),
                uri_document_identifier(self.document_uri.as_str()),
            )) {
                Ok(response) => {
                    self.trace_route("file", "complete", None);
                    return Ok(Some(self.remap(flatten_file_diagnostics(&response))));
                }
                Err(error @ CorsaError::Unsupported(_)) => {
                    self.trace_route("file", "unsupported", Some(&error));
                }
                Err(error) => {
                    self.trace_route("file", "failed", None);
                    return Err(cstr!("Failed to request Corsa file diagnostics: {error}"));
                }
            }
        }
        // A cancelled file RPC has drained. Do not start a second request on
        // behalf of the retired caller, or switch it to an editor transport.
        crate::corsa_bridge::native_operation::checkpoint()?;
        if self.project_supported {
            self.trace_route("project", "entered", None);
            match block_on(
                self.client.get_diagnostics_for_project(
                    self.snapshot.handle.clone(),
                    self.project.id.clone(),
                ),
            ) {
                Ok(response) => {
                    self.trace_route("project", "complete", None);
                    let diagnostics = response
                        .files
                        .iter()
                        .find(|file| document_identifier_uri(&file.file) == self.document_uri)
                        .map(flatten_file_diagnostics)
                        .unwrap_or_default();
                    return Ok(Some(self.remap(diagnostics)));
                }
                Err(error @ CorsaError::Unsupported(_)) => {
                    self.trace_route("project", "unsupported", Some(&error));
                }
                Err(error) => {
                    self.trace_route("project", "failed", None);
                    return Err(cstr!(
                        "Failed to request Corsa project diagnostics: {error}"
                    ));
                }
            }
        }
        self.trace_route("read", "unsupported", None);
        Ok(None)
    }

    fn trace_route(&self, route: &str, event: &str, error: Option<&CorsaError>) {
        if crate::corsa_bridge::preparation_trace::enabled() {
            tracing::info!(target: "vize_editor_preparation", phase = "initial_api_read",
                route, event, logical_uri = %self.uri, request_uri = %self.document_uri,
                project = self.project.id.as_str(), config = self.project.config_file_name.as_str(),
                snapshot = self.snapshot.handle.as_str(),
                capability_endpoint = self.capabilities.runtime.capability_endpoint,
                raw_file = self.capabilities.diagnostics.file,
                raw_project = self.capabilities.diagnostics.project,
                raw_snapshot = self.capabilities.diagnostics.snapshot,
                raw_overlay = self.capabilities.overlay.update_snapshot_overlay_changes,
                effective_file = self.file_supported, effective_project = self.project_supported,
                sdk_error = ?error);
        }
    }

    fn remap(&self, diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
        remap_serialized_uris(diagnostics.clone(), &self.external_document_uris)
            .unwrap_or(diagnostics)
    }
}
