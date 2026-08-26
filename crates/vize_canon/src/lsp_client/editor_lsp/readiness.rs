//! Transport-generation readiness barrier for editor LSP documents.

use lsp_types::Uri;
use std::str::FromStr;
use vize_s0::{FxHashMap, FxHashSet, String, cstr};

use super::EditorLspSession;

impl EditorLspSession {
    pub(super) fn ready_document_uri(&mut self, document_uri: &str) -> Result<Uri, String> {
        let uri = self.document_uri(document_uri)?;
        self.ready_generation_barrier(Some(document_uri))?;
        Ok(uri)
    }

    pub(super) fn ready_workspace_request(&mut self) -> Result<(), String> {
        self.ready_generation_barrier(None)
    }

    fn ready_generation_barrier(&mut self, query_document: Option<&str>) -> Result<(), String> {
        if self.ready_generation == Some(self.document_generation) {
            return Ok(());
        }

        // didOpen/didChange/didClose are notifications: a successful write
        // only proves transport delivery. A diagnostic response for one query
        // document does not prove that the server has installed the other
        // changed documents in the same semantic project, so acknowledge every
        // dirty identity before accepting the generation. A close has no live
        // identity to diagnose and instead uses the current query URI, or a
        // stable live project document for workspace requests, as the
        // response-backed topology barrier.
        let readiness_documents = readiness_documents(
            &self.dirty_documents,
            self.query_barrier_required,
            query_document,
            &self.documents,
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
        self.unacknowledged_notifications = 0;
        self.ready_generation = Some(self.document_generation);
        Ok(())
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

    pub(super) fn has_documents(&self, documents: &FxHashMap<String, String>) -> bool {
        self.documents.len() == documents.len()
            && documents.iter().all(|(uri, text)| {
                self.documents
                    .get(uri.as_str())
                    .is_some_and(|current| current == text)
            })
    }
}

fn readiness_documents(
    dirty_documents: &FxHashSet<String>,
    query_barrier_required: bool,
    query_document: Option<&str>,
    live_documents: &FxHashMap<String, String>,
) -> Vec<String> {
    let mut documents = dirty_documents.iter().cloned().collect::<Vec<_>>();
    if query_barrier_required {
        let barrier = query_document
            .filter(|uri| live_documents.contains_key(*uri))
            .map(String::from)
            .or_else(|| live_documents.keys().min().cloned());
        if let Some(barrier) = barrier
            && !dirty_documents.contains(barrier.as_str())
        {
            documents.push(barrier);
        }
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
        let live = FxHashMap::default();

        assert_eq!(
            readiness_documents(&dirty, false, Some("file:///workspace/query.ts"), &live),
            ["file:///workspace/a.ts", "file:///workspace/z.ts"]
        );
    }

    #[test]
    fn close_requires_the_query_document_as_a_barrier() {
        let live = FxHashMap::from_iter([("file:///workspace/query.ts".into(), "".into())]);
        assert_eq!(
            readiness_documents(
                &FxHashSet::default(),
                true,
                Some("file:///workspace/query.ts"),
                &live
            ),
            ["file:///workspace/query.ts"]
        );
    }

    #[test]
    fn workspace_request_uses_a_stable_live_document_as_a_close_barrier() {
        let live = FxHashMap::from_iter([
            ("file:///workspace/z.ts".into(), "".into()),
            ("file:///workspace/a.ts".into(), "".into()),
        ]);

        assert_eq!(
            readiness_documents(&FxHashSet::default(), true, None, &live),
            ["file:///workspace/a.ts"]
        );
    }
}
