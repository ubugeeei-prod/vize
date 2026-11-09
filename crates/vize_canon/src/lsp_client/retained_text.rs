//! Keep byte-identical retained buffers in their original owned allocation.
#![expect(
    clippy::disallowed_types,
    reason = "batch payloads are shared with the retained document map"
)]

use std::sync::Arc;

use super::CorsaProjectClient;
use super::shared_text::RetainedText;
use vize_l0::{FxHashMap, String};

/// The caller retains a replaced value through its existing insertion scope.
#[must_use]
pub(super) struct RetainedTextUpdate {
    unchanged: bool,
    previous: Option<RetainedText>,
}

impl RetainedTextUpdate {
    pub(super) fn changed(&self) -> bool {
        !self.unchanged
    }

    pub(super) fn was_present(&self) -> bool {
        self.unchanged || self.previous.is_some()
    }

    #[cfg(test)]
    pub(super) fn previous_text(&self) -> Option<&str> {
        self.previous.as_deref()
    }
}

/// Immutable batch payloads by the address of their text, so a borrowed row
/// of that exact allocation is retained by sharing it.
#[derive(Default)]
pub(super) struct SharedPayloads(FxHashMap<usize, Arc<(String, String)>>);

impl SharedPayloads {
    pub(super) fn register(&mut self, payload: &Arc<(String, String)>) {
        self.0
            .insert(payload.1.as_ptr() as usize, Arc::clone(payload));
    }

    pub(super) fn clear(&mut self) {
        self.0.clear();
    }

    fn payload(&self, uri: &str, text: &str) -> Option<&Arc<(String, String)>> {
        self.0
            .get(&(text.as_ptr() as usize))
            .filter(|payload| payload.1.len() == text.len() && payload.0 == uri)
    }

    fn text(&self, uri: &str, text: &str) -> RetainedText {
        self.payload(uri, text)
            .map_or_else(|| text.into(), RetainedText::shared)
    }
}

impl CorsaProjectClient {
    /// Update only this owner's text. URI mapping and native synchronization
    /// remain the caller's responsibility, including when the text is equal.
    pub(super) fn retain_document_text(&mut self, uri: &str, text: &str) -> RetainedTextUpdate {
        if self
            .document_texts
            .get(uri)
            .is_some_and(|current| current.as_str() == text)
        {
            // Equal bytes from a batch payload replace an owned copy in place.
            if let Some(payload) = self.shared_payloads.payload(uri, text) {
                self.document_texts
                    .share_equal(uri, RetainedText::shared(payload));
            }
            return RetainedTextUpdate {
                unchanged: true,
                previous: None,
            };
        }
        let text = self.shared_payloads.text(uri, text);
        RetainedTextUpdate {
            unchanged: false,
            previous: self.document_texts.insert(uri.into(), text),
        }
    }
}
