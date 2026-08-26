//! Bounded editor-overlay synchronization for large project surfaces.

use lsp_types::Uri;
use std::str::FromStr;
use vize_s0::{FxHashMap, String, cstr};

use super::EditorLspSession;

/// Stay comfortably below the transport's bounded outbound notification queue.
/// A response-backed diagnostic drains every notification sent before it.
const NOTIFICATIONS_PER_BARRIER: usize = 128;

impl EditorLspSession {
    /// Bring the reusable editor transport to the same virtual-project view as
    /// the project-session transport, including dependency removals.
    pub(super) fn synchronize(
        &mut self,
        documents: &FxHashMap<String, String>,
    ) -> Result<(), String> {
        let mut desired = documents.iter().collect::<Vec<_>>();
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

        let mut removed = self
            .documents
            .keys()
            .filter(|uri| !documents.contains_key(uri.as_str()))
            .cloned()
            .collect::<Vec<_>>();
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
        Ok(())
    }

    fn close_mirrored_document(&mut self, document_uri: &str) -> Result<(), String> {
        let uri = parse_document_uri(document_uri)?;
        self.overlay.close(&uri).map_err(|error| {
            cstr!("Failed to close editor LSP overlay for {document_uri}: {error}")
        })?;
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
        super::super::diagnostics_lsp::request_lsp_document_diagnostic_ack(&self.client, uri)
            .map_err(|error| cstr!("Failed to drain editor LSP overlay notifications: {error}"))?;
        self.unacknowledged_notifications = 0;
        Ok(())
    }
}

fn parse_document_uri(document_uri: &str) -> Result<Uri, String> {
    Uri::from_str(document_uri)
        .map_err(|error| cstr!("Invalid LSP document URI {document_uri}: {error}"))
}
