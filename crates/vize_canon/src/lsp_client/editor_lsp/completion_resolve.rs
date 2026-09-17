//! Deferred completion metadata on the existing editor transport.

use corsa::runtime::block_on;
use serde_json::Value;
use vize_s0::{String, cstr};

use super::{CorsaProjectClient, EditorLspSession};

struct CompletionResolveRequest;

impl lsp_types::request::Request for CompletionResolveRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "completionItem/resolve";
}

impl CorsaProjectClient {
    pub(crate) fn completion_resolve_via_editor_lsp(
        &mut self,
        uri: &str,
        item: Value,
    ) -> Result<Option<Value>, String> {
        // Opaque data belongs to the existing process. Do not replay it on a
        // newly spawned session after transport failure.
        if self.editor_lsp.is_none() || !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        // Diagnostics may have temporarily synchronized a different document
        // set. Restore the editor view before resolving the candidate.
        self.editor_lsp_session()?.completion_resolve(uri, item)
    }
}

impl EditorLspSession {
    fn completion_resolve(&mut self, uri: &str, item: Value) -> Result<Option<Value>, String> {
        self.ready_document_uri(uri)?;
        block_on(self.client.request::<CompletionResolveRequest>(item))
            .map_err(|error| cstr!("Failed to resolve editor LSP completion: {error}"))
    }
}
