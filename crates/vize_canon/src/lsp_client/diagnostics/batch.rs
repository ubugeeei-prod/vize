//! Complete batch diagnostic transport selection.

use super::{
    CorsaProjectClient, DiagnosticFetch, LspDiagnostic, PhasedDiagnostic, String, lsp_report,
    virtual_overlay_diagnostics,
};

impl CorsaProjectClient {
    /// Request diagnostics for multiple URIs in batch.
    pub fn request_diagnostics_batch(
        &mut self,
        uris: &[String],
    ) -> Result<Vec<(String, Vec<LspDiagnostic>)>, String> {
        virtual_overlay_diagnostics::ensure_materialized_project(
            self,
            uris.iter().map(|uri| uri.as_str()),
        )?;

        if self.has_materialized_documents(uris)
            && self.can_use_attached_semantic_api()
            && let Some(results) = self.request_diagnostics_batch_via_materialized_files(uris)?
        {
            #[cfg(test)]
            super::test_route::record(super::test_route::Route::Api);
            return Ok(results);
        }

        if self.can_batch_with_project_diagnostics(uris)
            && let Some(results) = self.request_diagnostics_batch_via_project_api(uris)?
        {
            #[cfg(test)]
            super::test_route::record(super::test_route::Route::Api);
            return Ok(results);
        }

        if let Some(results) = self.request_diagnostics_batch_via_lsp(uris)? {
            #[cfg(test)]
            super::test_route::record(super::test_route::Route::Editor);
            return Ok(results);
        }

        uris.iter()
            .map(|uri| {
                let diagnostics = self.request_diagnostics(uri.as_str())?;
                Ok((uri.clone(), diagnostics))
            })
            .collect()
    }
}

impl CorsaProjectClient {
    pub(super) fn request_diagnostics_full_via_lsp(
        &mut self,
        uri: &str,
    ) -> Result<Option<DiagnosticFetch>, String> {
        let Some(mut results) = self.request_diagnostics_batch_via_lsp(&[uri.into()])? else {
            return Ok(None);
        };
        let Some((_, diagnostics)) = results.pop() else {
            return Ok(None);
        };
        let diagnostics = diagnostics
            .into_iter()
            .map(lsp_report::lsp_diagnostic_to_native)
            .map(PhasedDiagnostic::unknown)
            .collect::<Vec<_>>();
        Ok(Some(self.store_file_diagnostics(uri, diagnostics)))
    }
}
