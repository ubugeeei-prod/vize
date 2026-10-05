//! Test-only full vectors from the actual retained materialized session.

use super::super::super::CorsaProjectClient;
use serde_json::{Value, json};
use vize_l0::String;

impl CorsaProjectClient {
    pub(crate) fn qualify_native_bulk_for_test(&mut self, uris: &[String]) -> Value {
        self.begin_batch_test_receipt(true);
        let actual = self.request_diagnostics_batch(uris);
        let observed = self.batch_test_receipt(uris.first().unwrap());
        let complete = uris
            .iter()
            .map(|uri| json!(self.diagnostics.get(uri.as_str())))
            .collect::<Vec<_>>();
        // Use the original full per-file pull path on the same acknowledged
        // generation; it includes related information omitted by public maps.
        let original = self.request_diagnostics_batch_via_lsp(uris);
        let original_complete = uris
            .iter()
            .map(|uri| json!(self.diagnostics.get(uri.as_str())))
            .collect::<Vec<_>>();
        json!({
            "observed":observed, "requestedUris":uris,
            "actual":actual.as_ref().ok(), "actualError":actual.as_ref().err(),
            "completeNative":complete,
            "original":original.as_ref().ok(), "originalError":original.as_ref().err(),
            "completeOriginalLsp":original_complete,
        })
    }
}
