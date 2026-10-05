//! Complete batch diagnostic transport selection.

use super::{CorsaProjectClient, LspDiagnostic, String, virtual_overlay_diagnostics};

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
