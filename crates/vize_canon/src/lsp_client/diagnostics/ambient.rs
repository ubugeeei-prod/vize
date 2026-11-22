//! Reuse only answers attached to a native immutable snapshot still acknowledged by its owner.
use crate::lsp_client::{
    CorsaProjectClient, DiagnosticFetch, InitialDiagnosticsRead, PhasedDiagnostic,
};
use vize_l0::{String, cstr};

/// One successful compiler read and its exact native source world. A new
/// caller cannot construct a receipt or install one into another session.
pub struct AmbientDiagnosticSnapshot {
    read: InitialDiagnosticsRead,
    diagnostics: Vec<PhasedDiagnostic>,
}

impl CorsaProjectClient {
    pub(crate) fn read_ambient_diagnostic_snapshot(
        &mut self,
        uri: &str,
        previous: Option<AmbientDiagnosticSnapshot>,
    ) -> Result<(DiagnosticFetch, Option<AmbientDiagnosticSnapshot>), String> {
        // This also refreshes an attached editor's acknowledged native world.
        let Some(document_uri) = self.prepare_diagnostics_read_document(uri)? else {
            return self
                .request_diagnostics_full(uri)
                .map(|fetch| (fetch, None));
        };
        if let Some(previous) = previous
            && self.initial_diagnostics_read_is_current(&previous.read)
            && previous.read.confirms_ambient_project(uri, &document_uri)?
        {
            let fetch = DiagnosticFetch {
                diagnostics: previous.diagnostics.clone(),
                used_cache: true,
            };
            return Ok((fetch, Some(previous)));
        }
        let read = self
            .borrow_diagnostics_read_view(uri, &document_uri)?
            .into_owned();
        let rows = read.run_with_provenance()?;
        if !self.initial_diagnostics_read_is_current(&read) {
            return Err(cstr!(
                "Corsa diagnostic snapshot is no longer current for {uri}"
            ));
        }
        let Some(rows) = rows else {
            return self
                .request_lsp_diagnostics_or_cache(uri)
                .map(|fetch| (fetch, None));
        };
        let receipt = AmbientDiagnosticSnapshot {
            read,
            diagnostics: rows.clone(),
        };
        Ok((self.store_file_diagnostics(uri, rows), Some(receipt)))
    }
}
