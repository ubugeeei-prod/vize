//! A bounded explicit-project batch on the diagnosing process's native API.

use super::{CorsaProjectClient, DiagnosticBatch, String, convert_diagnostics};

impl CorsaProjectClient {
    pub(super) fn request_diagnostics_batch_via_native_bulk(
        &mut self,
        uris: &[String],
    ) -> Result<Option<DiagnosticBatch>, String> {
        let Some(config) = self.explicit_project_config.clone() else {
            return Ok(None);
        };
        if uris.is_empty() {
            return Ok(None);
        }
        let (documents, pairs) = self.editor_lsp_diagnostic_documents(uris)?;
        let requested = pairs.iter().map(|(_, uri)| uri.clone()).collect::<Vec<_>>();
        let response = self.request_with_editor_lsp_documents_recovery(&documents, |session| {
            session.bulk_diagnostics(&config, &requested)
        })?;
        let Some(response) = response else {
            return Ok(None);
        };
        let mut results = Vec::with_capacity(pairs.len());
        for ((external, _), diagnostics) in pairs.into_iter().zip(response) {
            let diagnostics = self.remap_diagnostics(diagnostics);
            self.diagnostics
                .insert(external.clone(), diagnostics.clone());
            results.push((external, convert_diagnostics(&diagnostics)));
        }
        Ok(Some(results))
    }
}
