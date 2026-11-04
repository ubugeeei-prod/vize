//! An owned, read-only API request detached from the mutable client lane.
#![expect(clippy::disallowed_types, reason = "SDK leases use std Arc")]

use std::{path::PathBuf, sync::Arc};

use corsa::api::{ApiClient, CapabilitiesResponse, ManagedSnapshot, ProjectResponse};
use vize_l0::{FxHashMap, String, cstr};

use super::{CorsaProjectClient, PhasedDiagnostic};

mod diagnostic_response;
mod read_view;
pub(in crate::lsp_client) use read_view::DiagnosticReadView;
pub(super) mod native_diagnostics;

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
    documents: FxHashMap<String, String>,
    external_document_uris: FxHashMap<String, String>,
    file_supported: bool,
    project_supported: bool,
    // Detached workers keep physical materialization alive until their RPC and
    // SDK snapshot/client fields have been dropped, including cancellation.
    _storage: Arc<super::session_storage::SessionStorage>,
}

impl CorsaProjectClient {
    pub(crate) fn prepare_initial_diagnostics_read(
        &mut self,
        uri: &str,
    ) -> Result<Option<InitialDiagnosticsRead>, String> {
        let Some(document_uri) = self.prepare_diagnostics_read_document(uri)? else {
            return Ok(None);
        };
        let read = self
            .borrow_diagnostics_read_view(uri, &document_uri)?
            .into_owned();
        if crate::corsa_bridge::preparation_trace::enabled() {
            tracing::info!(target: "vize_editor_preparation", phase="initial_api_owner",
                snapshot=read.snapshot.handle.as_str(), uri=%read.uri,
                shared_editor=self.project_session_attached_to_editor,
                editor_documents_dirty=self.editor_lsp_documents_dirty,
                materialized=self.materialized_project_session);
        }
        DiagnosticReadView::from_owned(&read).trace_route("capture", "prepared", None);
        Ok(Some(read))
    }

    pub(in crate::lsp_client) fn prepare_diagnostics_read_document(
        &mut self,
        uri: &str,
    ) -> Result<Option<String>, String> {
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
        self.refresh_acknowledged_editor_snapshot(&document_uri)?;
        self.project_session()?;
        Ok(Some(document_uri))
    }

    pub(in crate::lsp_client) fn borrow_diagnostics_read_view<'a>(
        &'a self,
        uri: &'a str,
        document_uri: &'a str,
    ) -> Result<DiagnosticReadView<'a>, String> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| cstr!("Corsa diagnostics have no project session"))?;
        Ok(DiagnosticReadView {
            client: session.client(),
            snapshot: session.snapshot_lease(),
            project: session.project(),
            capabilities: &self.capabilities,
            project_root: &self.project_root,
            explicit_config: &self.explicit_project_config,
            uri,
            document_uri,
            documents: &self.document_texts,
            external_document_uris: &self.external_document_uris,
            file_supported: self.supports_file_diagnostics_api(),
            project_supported: self.supports_project_diagnostics_api(),
            storage: &self.overlay_storage,
        })
    }

    /// Shared borrowing fixes this world during execution; retain the runtime owner checks.
    pub(in crate::lsp_client) fn borrowed_diagnostics_read_is_current(
        &self,
        read: &DiagnosticReadView<'_>,
    ) -> bool {
        !self.closed
            && self.can_use_api_for_uri(read.uri)
            && self.session.as_ref().is_some_and(|session| {
                Arc::ptr_eq(&read.snapshot, &session.snapshot_lease())
                    && std::ptr::eq(read.project, session.project())
            })
            && Arc::ptr_eq(&self.capabilities, read.capabilities)
            && std::ptr::eq(read.project_root, &self.project_root)
            && std::ptr::eq(read.explicit_config, &self.explicit_project_config)
            && std::ptr::eq(read.documents, &*self.document_texts)
            && std::ptr::eq(read.external_document_uris, &self.external_document_uris)
            && self
                .session_document_uris
                .get(read.uri)
                .map(|uri| uri.as_str())
                == Some(read.document_uri)
    }

    pub(crate) fn replace_initial_diagnostics_cache(
        &mut self,
        uri: &str,
        replacement: Option<Vec<PhasedDiagnostic>>,
    ) -> Option<Vec<PhasedDiagnostic>> {
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
            && self.can_use_api_for_uri(read.uri.as_str())
            && self.session.as_ref().is_some_and(|session| {
                Arc::ptr_eq(&read.snapshot, &session.snapshot_lease())
                    && session.project() == &read.project
            })
            && Arc::ptr_eq(&self.capabilities, &read.capabilities)
            && self.project_root == read.project_root
            && self.explicit_project_config == read.explicit_config
            && self.document_texts == read.documents
            && self.external_document_uris == read.external_document_uris
            && self.session_document_uris.get(read.uri.as_str()) == Some(&read.document_uri)
    }
}

// Passive values already owned by the same read; no extra RPC/disk lookup.
fn trace_kind(params: &serde_json::Value, method: &str, response: &serde_json::Value) {
    if crate::corsa_bridge::preparation_trace::enabled() {
        tracing::info!(target: "vize_editor_preparation", phase="initial_compiler_kind",
            request=%params, method, response=%response);
    }
}
fn trace_source(params: &serde_json::Value, name: &str, text: &str) {
    if crate::corsa_bridge::preparation_trace::enabled() {
        tracing::info!(target: "vize_editor_preparation", phase="initial_compiler_source", request=%params, name, text);
    }
}

impl InitialDiagnosticsRead {
    pub(crate) fn uri(&self) -> &str {
        self.uri.as_str()
    }
    pub(crate) fn run_with_provenance(&self) -> Result<Option<Vec<PhasedDiagnostic>>, String> {
        DiagnosticReadView::from_owned(self).run_with_provenance()
    }
}
