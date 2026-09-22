//! The resident tier behind the request paths (Davinci P5-6a, diagnostics in
//! P5-6b).
//!
//! Hover, completion, definition and diagnostics read the SFC parse of a
//! document through [`ResidentCache`] instead of calling `parse_sfc` per
//! request: the cache holds each document as an input of a `vize_resident`
//! salsa database, so every request between two keystrokes shares one parse.
//! A rejected parse stays in that memo, error included. Components read from
//! disk go through the same cache keyed by their path.
//!
//! The lock is held only for the lookup — never across an `.await` — and the
//! descriptor handed out is an owned, shared snapshot.

use std::path::Path;

use parking_lot::Mutex;
use tower_lsp::lsp_types::Url;
use vize_resident::{ParsedSfc, ResidentDocuments, SharedDescriptor};

use super::ServerState;

/// Every document the request paths have read, one resident database.
#[derive(Default)]
pub(crate) struct ResidentCache(Mutex<ResidentDocuments>);

impl ResidentCache {
    /// The parse of document `key`'s `text`, including a rejection.
    pub(crate) fn parsed(&self, key: &str, filename: &str, text: &str) -> ParsedSfc {
        self.0.lock().parsed(key, filename, text)
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
        self.sfc_parsed(uri, text).into_descriptor()
    }

    /// The memoized parse of open document `uri` whose text is `text`. A
    /// rejection is stored with the error, so diagnostics can publish it
    /// without parsing the buffer again.
    pub(crate) fn sfc_parsed(&self, uri: &Url, text: &str) -> ParsedSfc {
        self.resident.parsed(uri.as_str(), uri.path(), text)
    }

    /// The memoized descriptor of the component at `path` whose text was just
    /// read from disk as `text`.
    pub(crate) fn component_descriptor(&self, path: &Path, text: &str) -> Option<SharedDescriptor> {
        let path = path.to_string_lossy();
        self.resident.parsed(&path, &path, text).into_descriptor()
    }
}

#[cfg(test)]
mod tests;
