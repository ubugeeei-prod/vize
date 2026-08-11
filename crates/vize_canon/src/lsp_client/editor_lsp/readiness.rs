//! Transport-generation readiness barrier for editor LSP documents.

use lsp_types::Uri;
use std::str::FromStr;
use vize_carton::{String, cstr};

use super::EditorLspSession;

impl EditorLspSession {
    pub(super) fn ready_document_uri(&mut self, document_uri: &str) -> Result<Uri, String> {
        let uri = self.document_uri(document_uri)?;
        if self.ready_generation == Some(self.document_generation) {
            return Ok(uri);
        }

        // didOpen/didChange are notifications: a successful write only proves
        // transport delivery, not that the server has installed this document
        // generation in its semantic project. A pull-diagnostic request is the
        // first request that consumes the transport's current project state
        // and therefore provides an ordered, response-backed readiness barrier
        // before editor queries, including after another document changed.
        super::super::diagnostics_lsp::request_lsp_document_diagnostics(&self.client, &uri)
            .map_err(|error| {
                cstr!("Failed to establish editor LSP readiness for {document_uri}: {error}")
            })?;
        self.ready_generation = Some(self.document_generation);
        Ok(uri)
    }

    pub(super) fn advance_document_generation(&mut self) {
        self.document_generation = self.document_generation.wrapping_add(1);
        self.ready_generation = None;
    }

    fn document_uri(&self, document_uri: &str) -> Result<Uri, String> {
        if !self.documents.contains_key(document_uri) {
            return Err(cstr!(
                "Editor LSP virtual project does not contain {document_uri}"
            ));
        }
        Uri::from_str(document_uri)
            .map_err(|error| cstr!("Invalid LSP document URI {document_uri}: {error}"))
    }
}
