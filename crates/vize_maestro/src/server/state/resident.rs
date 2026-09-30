//! The resident tier behind the request paths (Davinci P5-6a, diagnostics in
//! P5-6b, annotations and document structure in P5-6c).
//!
//! Hover, completion, definition, diagnostics, semantic tokens, inlay hints
//! document links, lenses, colours, symbols and folding read the SFC parse
//! through [`ResidentCache`] instead of
//! calling `parse_sfc` per request. The cache holds each document as an input
//! of a `vize_resident` salsa database, so every request between two
//! keystrokes shares one parse.
//! A rejected parse stays in that memo, error included. Components read from
//! disk go through the same cache keyed by their path.
//!
//! The lock is held only for the lookup — never across an `.await` — and the
//! descriptor handed out is an owned, shared snapshot.

use std::{path::Path, sync::Arc};

use parking_lot::Mutex;
use tower_lsp::lsp_types::Url;
use vize_atelier_sfc::script::TypeSourceSnapshot;
use vize_l2::summary::AlphaPages;
use vize_resident::{ComponentSurface, ParsedSfc, ResidentDocuments, SharedDescriptor};

use super::ServerState;

mod sources;

/// Every document the request paths have read, one resident database.
#[derive(Default)]
pub(crate) struct ResidentCache(Mutex<ResidentDocuments>, Mutex<sources::SourceState>);

impl ResidentCache {
    /// The parse of document `key`'s `text`, including a rejection.
    pub(crate) fn parsed(&self, key: &str, filename: &str, text: &str) -> ParsedSfc {
        self.0.lock().parsed(key, filename, text)
    }

    /// Release document `key`'s buffer.
    pub(crate) fn close(&self, key: &str) {
        self.0.lock().close(key);
    }

    pub(crate) fn close_document(&self, uri: &Url) {
        self.close(uri.as_str());
        if let Ok(path) = uri.to_file_path() {
            self.close(&path.to_string_lossy());
            if let Ok(canonical) = path.canonicalize()
                && canonical != path
            {
                self.close(&canonical.to_string_lossy());
            }
        }
        self.1.lock().close(uri);
    }

    pub(crate) fn rename(
        &self,
        documents: &crate::document::DocumentStore,
        old: &Url,
        new: Url,
    ) -> bool {
        let renamed = documents.rename(old, new);
        if renamed {
            self.close_document(old);
        }
        renamed
    }

    pub(crate) fn invalidate_interfaces(&self) {
        self.0.lock().invalidate_interfaces();
        self.1.lock().invalidate();
    }

    pub(crate) fn sources(
        &self,
        documents: &crate::document::DocumentStore,
    ) -> Option<(u64, Arc<TypeSourceSnapshot>)> {
        let sources = self.1.lock().snapshot(documents)?;
        self.0.lock().set_source_world_revision(sources.0);
        Some(sources)
    }

    pub(crate) fn interface(
        &self,
        key: &str,
        text: &str,
        configuration: &str,
        export: impl FnOnce(&SharedDescriptor) -> Option<AlphaPages>,
    ) -> Option<ComponentSurface> {
        self.0
            .lock()
            .component_surface(key, key, text, configuration, export)
    }

    /// Lookups served and parses run since the last call.
    #[cfg(test)]
    pub(crate) fn take_stats(&self) -> vize_resident::DescriptorStats {
        self.0.lock().take_stats()
    }

    #[cfg(test)]
    pub(crate) fn take_interface_stats(&self) -> vize_resident::InterfaceStats {
        self.0.lock().take_interface_stats()
    }
}

impl ServerState {
    pub(crate) fn component_type_sources(&self) -> Option<(u64, Arc<TypeSourceSnapshot>)> {
        self.resident.sources(&self.documents)
    }
    /// Publish current Croquis alpha pages before imported-component
    /// completion/hover/diagnostics consume the declaration contracts.
    pub(crate) fn component_interface(
        &self,
        path: &Path,
        text: &str,
        configuration: &str,
        export: impl FnOnce(&SharedDescriptor) -> Option<AlphaPages>,
    ) -> Option<ComponentSurface> {
        self.resident
            .interface(&path.to_string_lossy(), text, configuration, export)
    }

    pub(crate) fn invalidate_component_interfaces(&self) {
        self.component_metadata_cache.clear();
        self.resident.invalidate_interfaces();
    }
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

#[cfg(test)]
mod request_tests;
