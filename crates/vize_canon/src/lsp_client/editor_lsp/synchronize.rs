//! Bounded editor-overlay synchronization for large project surfaces.

use lsp_types::notification::DidCloseTextDocument;
use lsp_types::{DidCloseTextDocumentParams, TextDocumentIdentifier, Uri};
use std::str::FromStr;
use vize_l0::{FxHashSet, String, cstr};

use super::EditorLspSession;
use crate::lsp_client::tracked_documents::TrackedDocumentTexts;
use crate::{
    file_uri::file_uri_to_path,
    lsp_client::shared_text::{DocumentTexts, RetainedText},
};
#[cfg(test)]
use vize_l0::FxHashMap;

/// Stay comfortably below the transport's bounded outbound notification queue.
/// A response-backed symbol request drains preceding notifications.
const NOTIFICATIONS_PER_BARRIER: usize = 128;
/// Preserve the established explicit-overlay contract for ordinary projects.
/// Disk-backed admission is only needed once a full replay would require a
/// transport drain; below that size, opening every retained source keeps wire
/// custody and watcher-independent functional evidence unchanged.
const DISK_DOCUMENT_MINIMUM: usize = NOTIFICATIONS_PER_BARRIER;

impl EditorLspSession {
    /// Returns whether disabling the disk view requires a complete overlay replay.
    pub(super) fn prefer_disk_documents(&mut self, enabled: bool) -> bool {
        self.prefer_disk_documents = enabled;
        let replay = !enabled && !self.disk_documents.is_empty();
        if replay {
            self.disk_documents.clear();
        }
        replay
    }

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
        let admit_disk_documents =
            self.prefer_disk_documents && documents.len() >= DISK_DOCUMENT_MINIMUM;
        desired.sort_unstable_by(|left, right| left.0.cmp(right.0));
        for (document_uri, text) in desired {
            if self.retain_exact_disk_document(admit_disk_documents, document_uri, text) {
                continue;
            }
            let changed = self
                .documents
                .get(document_uri.as_str())
                .is_none_or(|current| current != text);
            self.disk_documents.remove(document_uri.as_str());
            let uri = self.mirror(document_uri, text)?;
            if changed {
                self.flush_if_full(Some(&uri))?;
            }
        }

        let disk_removed = match changes {
            Some(changes) => changes
                .iter()
                .filter(|uri| !documents.contains_key(uri.as_str()))
                .filter(|uri| self.disk_documents.remove(uri.as_str()).is_some())
                .count(),
            None => {
                let before = self.disk_documents.len();
                self.disk_documents
                    .retain(|uri, _| documents.contains_key(uri.as_str()));
                before.saturating_sub(self.disk_documents.len())
            }
        };
        if disk_removed > 0 {
            self.query_barrier_required = true;
            self.advance_document_generation();
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

    fn retain_exact_disk_document(
        &mut self,
        admit_disk_documents: bool,
        document_uri: &str,
        text: &RetainedText,
    ) -> bool {
        if !admit_disk_documents || self.documents.contains_key(document_uri) {
            return false;
        }
        if let Some(matches) =
            existing_disk_document_matches(&self.disk_documents, document_uri, text)
        {
            // An admitted disk document is a read-only baseline. Once its
            // retained revision changes, the editor must observe that change
            // through didOpen/didChange even when materialization has already
            // written the same new bytes to disk. Otherwise the server can
            // keep its previous snapshot until an unreliable watcher event.
            return matches;
        }
        if !exact_disk_text(document_uri, text) {
            return false;
        }
        self.disk_documents
            .insert(document_uri.into(), text.clone());
        true
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
        self.disk_documents.remove(document_uri);
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

fn exact_disk_text(document_uri: &str, text: &RetainedText) -> bool {
    let Some(path) = file_uri_to_path(document_uri) else {
        return false;
    };
    std::fs::read(path).is_ok_and(|bytes| bytes == text.as_str().as_bytes())
}

fn existing_disk_document_matches(
    documents: &DocumentTexts,
    document_uri: &str,
    text: &RetainedText,
) -> Option<bool> {
    documents.get(document_uri).map(|current| current == text)
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
    fn exact_materialized_bytes_do_not_require_an_open_overlay() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("Member.vue.ts");
        std::fs::write(&path, "export const member = 1;").unwrap();
        let uri = crate::file_uri::path_to_file_uri(&path);

        assert!(exact_disk_text(
            &uri,
            &RetainedText::from("export const member = 1;")
        ));
        assert!(!exact_disk_text(
            &uri,
            &RetainedText::from("export const member = 2;")
        ));
        assert!(!exact_disk_text(
            "untitled:Member.vue.ts",
            &RetainedText::from("export const member = 1;")
        ));
    }

    #[test]
    fn changed_disk_baselines_require_an_editor_overlay() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("Member.vue.ts");
        let uri = crate::file_uri::path_to_file_uri(&path);
        let mut disk_documents = DocumentTexts::default();
        disk_documents.insert(uri.clone(), RetainedText::from("export const member = 1;"));
        let desired = RetainedText::from("export const member = 2;");
        std::fs::write(&path, desired.as_str()).unwrap();

        assert!(exact_disk_text(&uri, &desired));
        assert_eq!(
            existing_disk_document_matches(&disk_documents, &uri, &desired),
            Some(false)
        );
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
