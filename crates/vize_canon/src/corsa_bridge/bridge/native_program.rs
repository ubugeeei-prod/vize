//! Complete typed diagnostics without the compatibility DTO conversion.

use super::CorsaBridge;
use crate::CorsaBridgeError;
use lsp_types::RelatedFullDocumentDiagnosticReport;

impl CorsaBridge {
    pub(crate) async fn get_native_program_diagnostics(
        &self,
        uri: &str,
    ) -> Result<RelatedFullDocumentDiagnosticReport, CorsaBridgeError> {
        let uri = uri.to_owned();
        let (cache_len, report) = self
            .with_client(move |client| {
                let report = client
                    .request_native_program_diagnostics_full(&uri)
                    .map_err(CorsaBridgeError::CommunicationError)?;
                Ok((client.diagnostics_cache_len(), report))
            })
            .await?;
        self.cache_stats.set_entries(cache_len as u64);
        self.cache_stats.miss();
        Ok(report)
    }
}
