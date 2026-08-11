//! Transport-generation readiness barrier for editor LSP documents.

use lsp_types::Uri;
use std::str::FromStr;
use vize_carton::{FxHashSet, String, cstr};

use super::EditorLspSession;

impl EditorLspSession {
    pub(super) fn ready_document_uri(&mut self, document_uri: &str) -> Result<Uri, String> {
        let uri = self.document_uri(document_uri)?;
        if self.ready_generation == Some(self.document_generation) {
            return Ok(uri);
        }

        // didOpen/didChange/didClose are notifications: a successful write
        // only proves transport delivery. A diagnostic response for one query
        // document does not prove that the server has installed the other
        // changed documents in the same semantic project, so acknowledge every
        // dirty identity before accepting the generation. A close has no live
        // identity to diagnose and instead requires the current query URI as a
        // response-backed topology barrier.
        let readiness_documents = readiness_documents(
            &self.dirty_documents,
            self.query_barrier_required,
            document_uri,
        );
        for readiness_document in &readiness_documents {
            let readiness_uri = Uri::from_str(readiness_document).map_err(|error| {
                cstr!("Invalid LSP readiness document URI {readiness_document}: {error}")
            })?;
            super::super::diagnostics_lsp::request_lsp_document_diagnostic_ack(
                &self.client,
                &readiness_uri,
            )
            .map_err(|error| {
                cstr!("Failed to establish editor LSP readiness for {readiness_document}: {error}")
            })?;
        }
        self.dirty_documents.clear();
        self.query_barrier_required = false;
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

fn readiness_documents(
    dirty_documents: &FxHashSet<String>,
    query_barrier_required: bool,
    query_document: &str,
) -> Vec<String> {
    let mut documents = dirty_documents.iter().cloned().collect::<Vec<_>>();
    if query_barrier_required && !dirty_documents.contains(query_document) {
        documents.push(query_document.into());
    }
    documents.sort_unstable();
    documents
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirty_documents_are_acknowledged_in_stable_order() {
        let dirty = FxHashSet::from_iter([
            "file:///workspace/z.ts".into(),
            "file:///workspace/a.ts".into(),
        ]);

        assert_eq!(
            readiness_documents(&dirty, false, "file:///workspace/query.ts"),
            ["file:///workspace/a.ts", "file:///workspace/z.ts"]
        );
    }

    #[test]
    fn close_requires_the_query_document_as_a_barrier() {
        assert_eq!(
            readiness_documents(&FxHashSet::default(), true, "file:///workspace/query.ts"),
            ["file:///workspace/query.ts"]
        );
    }
}
