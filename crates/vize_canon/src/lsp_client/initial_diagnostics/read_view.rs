//! Borrow the current source world during synchronous native diagnostics.
#![expect(clippy::disallowed_types, reason = "SDK leases use std Arc")]

use super::super::{
    PhasedDiagnostic,
    diagnostics_api::{document_identifier_uri, phased_file_diagnostics},
    session::uri_document_identifier,
};
use super::{InitialDiagnosticsRead, native_diagnostics};
use corsa::{
    CorsaError,
    api::{ApiClient, CapabilitiesResponse, ManagedSnapshot, ProjectResponse},
    runtime::block_on,
};
use std::{path::PathBuf, sync::Arc};
use vize_l0::{FxHashMap, String, cstr};

type Storage = super::super::session_storage::SessionStorage;

pub(in crate::lsp_client) struct DiagnosticReadView<'a> {
    pub(super) client: &'a ApiClient,
    pub(super) snapshot: Arc<ManagedSnapshot>,
    pub(super) project: &'a ProjectResponse,
    pub(super) capabilities: &'a Arc<CapabilitiesResponse>,
    pub(super) project_root: &'a PathBuf,
    pub(super) explicit_config: &'a Option<PathBuf>,
    pub(super) uri: &'a str,
    pub(super) document_uri: &'a str,
    pub(super) documents: &'a FxHashMap<String, String>,
    pub(super) external_document_uris: &'a FxHashMap<String, String>,
    pub(super) file_supported: bool,
    pub(super) project_supported: bool,
    pub(super) storage: &'a Arc<Storage>,
}

/// Retain the same physical mirror and SDK snapshot until commit/fallback ends.
pub(in crate::lsp_client) struct DiagnosticReadLease {
    _snapshot: Arc<ManagedSnapshot>,
    _storage: Arc<Storage>,
}

impl<'a> DiagnosticReadView<'a> {
    pub(super) fn from_owned(read: &'a InitialDiagnosticsRead) -> Self {
        Self {
            client: &read.client,
            snapshot: Arc::clone(&read.snapshot),
            project: &read.project,
            capabilities: &read.capabilities,
            project_root: &read.project_root,
            explicit_config: &read.explicit_config,
            uri: read.uri.as_str(),
            document_uri: read.document_uri.as_str(),
            documents: &read.documents,
            external_document_uris: &read.external_document_uris,
            file_supported: read.file_supported,
            project_supported: read.project_supported,
            storage: &read._storage,
        }
    }

    pub(super) fn into_owned(self) -> InitialDiagnosticsRead {
        InitialDiagnosticsRead {
            client: self.client.clone(),
            snapshot: self.snapshot,
            project: self.project.clone(),
            capabilities: Arc::clone(self.capabilities),
            project_root: self.project_root.clone(),
            explicit_config: self.explicit_config.clone(),
            uri: self.uri.into(),
            document_uri: self.document_uri.into(),
            documents: self.documents.clone(),
            external_document_uris: self.external_document_uris.clone(),
            file_supported: self.file_supported,
            project_supported: self.project_supported,
            _storage: Arc::clone(self.storage),
        }
    }

    pub(in crate::lsp_client) fn into_lease(self) -> DiagnosticReadLease {
        DiagnosticReadLease {
            _snapshot: self.snapshot,
            _storage: Arc::clone(self.storage),
        }
    }
}

impl DiagnosticReadView<'_> {
    /// The caller retains this source world until every entered RPC returns.
    /// Results never touch the mutable client during execution.
    pub(crate) fn run_with_provenance(&self) -> Result<Option<Vec<PhasedDiagnostic>>, String> {
        if self.file_supported {
            self.trace_route("file", "entered", None);
            match block_on(self.client.get_diagnostics_for_file(
                self.snapshot.handle.clone(),
                self.project.id.clone(),
                uri_document_identifier(self.document_uri),
            )) {
                Ok(response) => {
                    self.trace_route("file", "complete", None);
                    return Ok(Some(self.remap(phased_file_diagnostics(&response))));
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
                        .find(|file| {
                            document_identifier_uri(&file.file).as_str() == self.document_uri
                        })
                        .map(phased_file_diagnostics)
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
        if !self.file_supported {
            self.trace_route("read", "unsupported", None);
            return Ok(None);
        }
        crate::corsa_bridge::native_operation::checkpoint()?;
        // Standard native 7.0.2 has compiler-kind endpoints, not the optional
        // aggregate SDK methods. This read retains this same snapshot lease;
        // no project selection, overlay update or live disk read occurs here.
        self.trace_route("compiler_kinds", "entered", None);
        let result =
            native_diagnostics::read(self.client, &self.snapshot, self.project, self.document_uri)?;
        self.trace_route(
            "compiler_kinds",
            if result.is_some() {
                "complete"
            } else {
                "unsupported"
            },
            None,
        );
        Ok(result.map(|diagnostics| self.remap(diagnostics)))
    }

    pub(super) fn trace_route(&self, route: &str, event: &str, error: Option<&CorsaError>) {
        if crate::corsa_bridge::preparation_trace::enabled() {
            if route == "capture" {
                tracing::info!(target: "vize_editor_preparation", phase="initial_api_input",
                    snapshot=self.snapshot.handle.as_str(), project=?self.project,
                    logical_uri=%self.uri, request_uri=%self.document_uri, query_text=?self.documents.get(self.uri));
            }
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

    fn remap(&self, diagnostics: Vec<PhasedDiagnostic>) -> Vec<PhasedDiagnostic> {
        super::super::diagnostic_phase::remap_diagnostics(diagnostics, self.external_document_uris)
    }
}
