//! The resident tier behind the request paths (Davinci P5-6a).
//!
//! Hover, completion and definition read the SFC descriptor of a document
//! through [`ResidentCache`] instead of calling `parse_sfc` per request: the
//! cache holds each document as an input of a `vize_resident` salsa database,
//! so every request between two keystrokes shares one parse. Components read
//! from disk go through the same cache keyed by their path.
//!
//! The lock is held only for the lookup — never across an `.await` — and the
//! descriptor handed out is an owned, shared snapshot.

use std::path::Path;

use parking_lot::Mutex;
use tower_lsp::lsp_types::Url;
use vize_resident::{ResidentDocuments, SharedDescriptor};

use super::ServerState;

/// Every document the request paths have read, one resident database.
#[derive(Default)]
pub(crate) struct ResidentCache(Mutex<ResidentDocuments>);

impl ResidentCache {
    /// The descriptor of document `key`'s `text`, parsed as `filename`.
    pub(crate) fn descriptor(
        &self,
        key: &str,
        filename: &str,
        text: &str,
    ) -> Option<SharedDescriptor> {
        self.0.lock().descriptor(key, filename, text)
    }

    /// Release document `key`'s buffer.
    pub(crate) fn close(&self, key: &str) {
        self.0.lock().close(key);
    }

    /// Lookups served and parses run since the last call.
    #[cfg(test)]
    pub(crate) fn take_stats(&self) -> vize_resident::DescriptorStats {
        self.0.lock().take_stats()
    }
}

impl ServerState {
    /// The memoized descriptor of open document `uri` whose text is `text`.
    pub(crate) fn sfc_descriptor(&self, uri: &Url, text: &str) -> Option<SharedDescriptor> {
        self.resident.descriptor(uri.as_str(), uri.path(), text)
    }

    /// The memoized descriptor of the component at `path` whose text was just
    /// read from disk as `text`.
    pub(crate) fn component_descriptor(&self, path: &Path, text: &str) -> Option<SharedDescriptor> {
        let path = path.to_string_lossy();
        self.resident.descriptor(&path, &path, text)
    }
}

#[cfg(test)]
mod tests;
