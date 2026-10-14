//! Deferred completion metadata on the existing editor transport.

use corsa::runtime::block_on;
use serde_json::Value;
use vize_l0::{String, cstr};

use super::{CorsaProjectClient, EditorLspSession, requests::RawCompletionRequest};

mod issued;
pub(super) use issued::IssuedCompletions;

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
    pub(super) fn completion(
        &mut self,
        document_uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        let uri = self.ready_document_uri(document_uri)?;
        let response = block_on(
            self.client
                .request::<RawCompletionRequest>(serde_json::json!({
                    "textDocument": { "uri": uri },
                    "position": { "line": line, "character": character },
                    "context": { "triggerKind": 1 },
                })),
        )
        .map_err(|error| cstr!("Failed to request editor LSP completion: {error}"))?;
        self.issued_completions
            .record(self.document_generation, document_uri, response.as_ref());
        Ok(response)
    }

    fn completion_resolve(&mut self, uri: &str, item: Value) -> Result<Option<Value>, String> {
        if self.closed
            || !self
                .issued_completions
                .allows(self.document_generation, uri, &item)
        {
            return Ok(None);
        }
        self.ready_document_uri(uri)?;
        block_on(self.client.request::<CompletionResolveRequest>(item))
            .map_err(|error| cstr!("Failed to resolve editor LSP completion: {error}"))
    }
}
