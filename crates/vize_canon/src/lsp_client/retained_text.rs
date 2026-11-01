//! Keep byte-identical retained buffers in their original owned allocation.

use super::CorsaProjectClient;
use vize_l0::String;

/// The caller retains a replaced value through its existing insertion scope.
#[must_use]
pub(super) struct RetainedTextUpdate {
    unchanged: bool,
    previous: Option<String>,
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

impl CorsaProjectClient {
    /// Update only this owner's text. URI mapping and native synchronization
    /// remain the caller's responsibility, including when the text is equal.
    pub(super) fn retain_document_text(&mut self, uri: &str, text: &str) -> RetainedTextUpdate {
        if self
            .document_texts
            .get(uri)
            .is_some_and(|current| current.as_str() == text)
        {
            return RetainedTextUpdate {
                unchanged: true,
                previous: None,
            };
        }
        RetainedTextUpdate {
            unchanged: false,
            previous: self.document_texts.insert(uri.into(), text.into()),
        }
    }
}
