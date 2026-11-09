//! Bounded editor-overlay synchronization for large project surfaces.

use lsp_types::notification::DidCloseTextDocument;
use lsp_types::{DidCloseTextDocumentParams, TextDocumentIdentifier, Uri};
use std::str::FromStr;
use vize_l0::{FxHashSet, String, cstr};

use super::EditorLspSession;
use crate::lsp_client::shared_text::{DocumentTexts, RetainedText};
use crate::lsp_client::tracked_documents::TrackedDocumentTexts;
#[cfg(test)]
use vize_l0::FxHashMap;

/// Stay comfortably below the transport's bounded outbound notification queue.
/// A response-backed symbol request drains preceding notifications.
const NOTIFICATIONS_PER_BARRIER: usize = 128;

impl EditorLspSession {
    /// Bring the reusable editor transport to the same virtual-project view as
    /// the project-session transport, including dependency removals.
    pub(super) fn synchronize(&mut self, documents: &DocumentTexts) -> Result<(), String> {
        self.tracked_documents = None;
        self.synchronize_selected(documents, None)
    }

    pub(super) fn synchronize_tracked(
        &mut self,
        documents: &mut TrackedDocumentTexts,
    ) -> Result<(), String> {
        let changes = documents.changes_since(self.tracked_documents);
        self.synchronize_selected(documents, changes)?;
        self.tracked_documents = Some(documents.acknowledge());
        Ok(())
    }

    fn synchronize_selected(
        &mut self,
        documents: &DocumentTexts,
        changes: Option<&FxHashSet<String>>,
    ) -> Result<(), String> {
        let phase =
            crate::corsa_bridge::preparation_trace::Phase::start("editor_mirror", documents.len());
        let (mut desired, mut removed) = selected_documents(documents, &self.documents, changes);
        desired.sort_unstable_by(|left, right| left.0.cmp(right.0));
        for (document_uri, text) in desired {
            let changed = self
                .documents
                .get(document_uri.as_str())
                .is_none_or(|current| current != text);
            let uri = self.mirror(document_uri, text)?;
            if changed {
                self.flush_if_full(Some(&uri))?;
            }
        }

        if removed.is_empty() {
            phase.finish();
            return Ok(());
        }
        removed.sort_unstable();
        let retained_barrier = documents
            .keys()
            .min()
            .cloned()
            .or_else(|| (removed.len() > 1).then(|| removed.pop()).flatten());
        let retained_uri = retained_barrier
            .as_deref()
            .map(parse_document_uri)
            .transpose()?;

        for document_uri in removed {
            self.close_mirrored_document(&document_uri)?;
            self.flush_if_full(retained_uri.as_ref())?;
        }
        if let Some(document_uri) = retained_barrier
            && !documents.contains_key(document_uri.as_str())
        {
            self.close_mirrored_document(&document_uri)?;
            self.flush_if_full(None)?;
        }
        phase.finish();
        Ok(())
    }

    fn close_mirrored_document(&mut self, document_uri: &str) -> Result<(), String> {
        let uri = parse_document_uri(document_uri)?;
        self.tracked_documents = None;
        if self.versions.remove(document_uri).is_some() {
            self.client
                .notify::<DidCloseTextDocument>(DidCloseTextDocumentParams {
                    text_document: TextDocumentIdentifier::new(uri),
                })
                .map_err(|error| {
                    cstr!("Failed to close editor LSP overlay for {document_uri}: {error}")
                })?;
        }
        self.documents.remove(document_uri);
        self.dirty_documents.remove(document_uri);
        self.query_barrier_required = true;
        self.advance_document_generation();
        Ok(())
    }

    fn flush_if_full(&mut self, barrier_uri: Option<&Uri>) -> Result<(), String> {
        self.unacknowledged_notifications = self.unacknowledged_notifications.saturating_add(1);
        if self.unacknowledged_notifications < NOTIFICATIONS_PER_BARRIER {
            return Ok(());
        }
        let Some(uri) = barrier_uri else {
            return Ok(());
        };
        let phase = crate::corsa_bridge::preparation_trace::Phase::start(
            "editor_notification_drain",
            self.unacknowledged_notifications,
        );
        super::super::diagnostics_lsp::request_lsp_document_readiness_ack(&self.client, uri)
            .map_err(|error| cstr!("Failed to drain editor LSP overlay notifications: {error}"))?;
        phase.finish();
        self.unacknowledged_notifications = 0;
        Ok(())
    }
}

fn selected_documents<'a>(
    documents: &'a DocumentTexts,
    mirrored: &DocumentTexts,
    changes: Option<&FxHashSet<String>>,
) -> (Vec<(&'a String, &'a RetainedText)>, Vec<String>) {
    match changes {
        Some(changes) => (
            changes
                .iter()
                .filter_map(|uri| documents.get_key_value(uri.as_str()))
                .collect(),
            changes
                .iter()
                .filter(|uri| {
                    !documents.contains_key(uri.as_str()) && mirrored.contains_key(uri.as_str())
                })
                .cloned()
                .collect(),
        ),
        None => (
            documents.iter().collect(),
            mirrored
                .keys()
                .filter(|uri| !documents.contains_key(uri.as_str()))
                .cloned()
                .collect(),
        ),
    }
}

fn parse_document_uri(document_uri: &str) -> Result<Uri, String> {
    Uri::from_str(document_uri)
        .map_err(|error| cstr!("Invalid LSP document URI {document_uri}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> DocumentTexts {
        FxHashMap::from_iter([
            ("file:///a.ts".into(), "a".into()),
            ("file:///b.ts".into(), "b".into()),
            ("file:///z.ts".into(), "z".into()),
        ])
    }

    #[test]
    fn certified_single_edit_does_not_select_unrelated_texts() {
        let mut documents = rows();
        let mirrored = documents.clone();
        documents.insert("file:///b.ts".into(), "changed".into());
        let changes = FxHashSet::from_iter(["file:///b.ts".into()]);
        let (desired, removed) = selected_documents(&documents, &mirrored, Some(&changes));
        assert_eq!(
            desired,
            vec![(
                documents.get_key_value("file:///b.ts").unwrap().0,
                documents.get("file:///b.ts").unwrap(),
            )]
        );
        assert!(removed.is_empty());
    }

    #[test]
    fn selection_preserves_new_and_removed_exact_uri_identities() {
        let mut documents = rows();
        let mirrored = documents.clone();
        documents.remove("file:///a.ts");
        documents.insert("file:///%61.ts".into(), "a".into());
        let changes = FxHashSet::from_iter(["file:///a.ts".into(), "file:///%61.ts".into()]);
        let (desired, removed) = selected_documents(&documents, &mirrored, Some(&changes));
        assert_eq!(desired.len(), 1);
        assert_eq!(desired[0].0.as_str(), "file:///%61.ts");
        assert_eq!(removed, vec![String::from("file:///a.ts")]);
    }

    #[test]
    fn unknown_generation_selects_the_complete_map_and_every_removal() {
        let documents = FxHashMap::from_iter([("file:///b.ts".into(), "changed".into())]);
        let mirrored = rows();
        let (desired, mut removed) = selected_documents(&documents, &mirrored, None);
        assert_eq!(desired.len(), 1);
        removed.sort_unstable();
        assert_eq!(
            removed,
            [String::from("file:///a.ts"), String::from("file:///z.ts")]
        );
    }
}
