//! Read provenance while borrowing the current client world.

use super::*;

impl CorsaProjectClient {
    pub(crate) fn request_diagnostics_full_with_provenance(
        &mut self,
        uri: &str,
    ) -> Result<DiagnosticFetch, String> {
        let Some(document_uri) = self.prepare_diagnostics_read_document(uri)? else {
            return self.request_diagnostics_full(uri);
        };
        let (result, _lease) = {
            let read = self.borrow_diagnostics_read_view(uri, &document_uri)?;
            let result = read.run_with_provenance();
            if !self.borrowed_diagnostics_read_is_current(&read) {
                return Err(cstr!(
                    "Corsa diagnostic snapshot is no longer current for {uri}"
                ));
            }
            (result?, read.into_lease())
        };
        // No await or transport mutation separates the checked view and commit.
        match result {
            Some(diagnostics) => Ok(self.store_file_diagnostics(uri, diagnostics)),
            None => self.request_lsp_diagnostics_or_cache(uri),
        }
    }
}

#[cfg(all(test, unix))]
mod tests;
