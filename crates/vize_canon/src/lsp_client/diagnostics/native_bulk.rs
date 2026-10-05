//! A bounded explicit-project batch on the diagnosing process's native API.

use super::super::editor_lsp::bulk_diagnostics::BulkDiagnostics;
use super::{CorsaProjectClient, DiagnosticBatch, String, convert_diagnostics};
use vize_l0::cstr;

impl CorsaProjectClient {
    pub(super) fn request_diagnostics_batch_via_native_bulk(
        &mut self,
        uris: &[String],
    ) -> Result<Option<DiagnosticBatch>, String> {
        #[cfg(test)]
        super::super::editor_lsp::bulk_diagnostics::reset_receipt();
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
        let response = match response {
            BulkDiagnostics::Refused => return Ok(None),
            BulkDiagnostics::Complete(response) => response,
            BulkDiagnostics::RetireOwner {
                request_error,
                release_error,
            } => {
                let cause = cstr!(
                    "native bulk request: {request_error:?}; snapshot release: {release_error:?}"
                );
                return self.original_batch_after_failed_attachment(uris, &cause);
            }
        };
        let mut results = Vec::with_capacity(pairs.len());
        for ((external, _), diagnostics) in pairs.into_iter().zip(response) {
            let diagnostics = self.remap_diagnostics(diagnostics);
            self.diagnostics
                .insert(external.clone(), diagnostics.clone());
            results.push((external, convert_diagnostics(&diagnostics)));
        }
        #[cfg(test)]
        super::test_route::record(super::test_route::Route::NativeBulk);
        Ok(Some(results))
    }

    fn original_batch_after_failed_attachment(
        &mut self,
        uris: &[String],
        cause: &str,
    ) -> Result<Option<DiagnosticBatch>, String> {
        // The SDK reader can close on a legal, deeply nested native chain.
        // Its flattened LSP counterpart need not exceed that JSON envelope.
        // Snapshot release was attempted before arriving here; retire the
        // failed attachment's owner using the existing lifecycle helper.
        let retirement = self.retire_editor_lsp();
        let fallback = match self.request_diagnostics_batch_via_lsp(uris) {
            Ok(Some(results)) => {
                #[cfg(test)]
                super::test_route::record(super::test_route::Route::Editor);
                Ok(Some(results))
            }
            Ok(None) => uris
                .iter()
                .map(|uri| {
                    self.request_diagnostics(uri)
                        .map(|diagnostics| (uri.clone(), diagnostics))
                })
                .collect::<Result<DiagnosticBatch, String>>()
                .map(Some),
            Err(error) => Err(error),
        };
        #[cfg(test)]
        super::super::editor_lsp::bulk_diagnostics::fallback_receipt(
            cause,
            retirement.as_ref().err(),
            fallback.as_ref().err(),
        );
        fallback.map_err(|error| cstr!(
            "{cause}; owner retirement: {retirement:?}; complete original diagnostic fallback: {error}"
        ))
    }
}
