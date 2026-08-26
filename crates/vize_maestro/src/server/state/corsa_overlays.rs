//! Incremental cache of the unsaved-buffer overlays handed to Corsa.
//!
//! Corsa resolves a component's imports against the filesystem, so every
//! diagnostics pass must also tell it about buffers the user has edited but not
//! saved. The straightforward way to do that — materialize the text of every
//! open document on every pass — makes one keystroke cost
//! `O(open documents x document size)` in copying before any type checking
//! starts, which is what #3442 is about.
//!
//! Between two consecutive passes at most one document can have changed, so the
//! overlays for the rest are already known. This cache keeps them as `Arc<str>`
//! and rebuilds only the entries whose [`Document::revision`] moved, turning the
//! steady-state pass into pointer work.
//!
//! [`Document::revision`]: crate::document::Document::revision
#![cfg(feature = "native")]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;
use tower_lsp::lsp_types::Url;
use vize_s0::FxHashMap;

use super::ServerState;
use crate::document::DocumentStore;

/// One open document's contribution to the overlay set.
struct OverlayEntry {
    /// Stamp of the document content `text` was materialized from. The cached
    /// text is reusable exactly while this still matches the live document.
    revision: u64,
    /// Pass that last observed this document open, used to evict entries for
    /// documents closed since.
    seen: u64,
    /// Shared so a pass can hand out the text without copying it.
    text: Arc<str>,
}

#[derive(Default)]
struct OverlayCacheState {
    /// Monotonic pass counter; see [`OverlayEntry::seen`].
    pass: u64,
    entries: FxHashMap<Url, OverlayEntry>,
}

/// Cached overlay set, refreshed incrementally per diagnostics pass.
#[derive(Default)]
pub(crate) struct CorsaOverlayCache {
    state: RwLock<OverlayCacheState>,
    /// Count of documents whose text this cache has materialized. Rises by one
    /// per changed document per pass once warm, which is the property the
    /// perf tests assert (a wall-clock bound could not).
    materializations: AtomicU64,
}

impl CorsaOverlayCache {
    /// The complete overlay set for the currently open documents.
    ///
    /// Reuses the cached text of every document whose revision is unchanged and
    /// materializes only the rest, so a pass following a single edit copies one
    /// document rather than all of them. Documents opened since the last pass
    /// are added, documents closed since are dropped.
    ///
    /// The returned snapshot is owned: it borrows nothing from the document
    /// store and holds no lock, so callers may keep it across the `.await` of a
    /// bridge call without parking the executor thread the next `didOpen` /
    /// `didChange` / `didClose` needs (#3315, #3377).
    pub(crate) fn snapshot(&self, documents: &DocumentStore) -> Vec<(PathBuf, Arc<str>)> {
        let mut state = self.state.write();
        let pass = state.pass.wrapping_add(1);
        state.pass = pass;

        let mut overlays = Vec::with_capacity(documents.len());
        for document in documents.iter() {
            // A document with no filesystem path cannot shadow a file Corsa
            // would otherwise read, so it is not an overlay.
            let Ok(path) = document.key().to_file_path() else {
                continue;
            };
            let revision = document.value().revision();

            let mut cached = None;
            if let Some(entry) = state.entries.get_mut(document.key())
                && entry.revision == revision
            {
                entry.seen = pass;
                cached = Some(Arc::clone(&entry.text));
            }

            let text = match cached {
                Some(text) => text,
                None => {
                    // Read the text from the same guard that produced
                    // `revision`, so the stamp always describes the bytes
                    // actually cached. Re-reading under a fresh guard could
                    // pair a stamp with text from a later edit and pin a stale
                    // overlay until the next change.
                    let text: Arc<str> = Arc::from(document.value().text());
                    self.materializations.fetch_add(1, Ordering::Relaxed);
                    state.entries.insert(
                        document.key().clone(),
                        OverlayEntry {
                            revision,
                            seen: pass,
                            text: Arc::clone(&text),
                        },
                    );
                    text
                }
            };
            overlays.push((path, text));
        }

        state.entries.retain(|_, entry| entry.seen == pass);
        overlays
    }

    /// Drop every cached overlay.
    ///
    /// Only ever costs a rebuild: entries are keyed by revision, so a stale
    /// entry cannot survive invalidation being missed. Called where the
    /// resolution the overlays feed can change out from under them.
    pub(crate) fn invalidate(&self) {
        let mut state = self.state.write();
        state.entries.clear();
    }

    /// Stop retaining the buffer for a document removed from the store.
    pub(super) fn remove(&self, uri: &Url) {
        self.state.write().entries.remove(uri);
    }

    /// How many documents this cache has materialized text for since creation.
    #[cfg(test)]
    pub(crate) fn materializations(&self) -> u64 {
        self.materializations.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(crate) fn entry_count(&self) -> usize {
        self.state.read().entries.len()
    }
}

impl ServerState {
    /// The complete Corsa overlay set for the currently open documents.
    ///
    /// See [`CorsaOverlayCache::snapshot`].
    pub(crate) fn corsa_overlays(&self) -> Vec<(PathBuf, Arc<str>)> {
        self.corsa_overlays.snapshot(&self.documents)
    }

    /// Drop the cached Corsa overlays.
    pub(crate) fn invalidate_corsa_overlays(&self) {
        self.corsa_overlays.invalidate();
    }

    #[cfg(test)]
    pub(crate) fn corsa_overlay_materializations(&self) -> u64 {
        self.corsa_overlays.materializations()
    }

    #[cfg(test)]
    pub(crate) fn corsa_overlay_entries(&self) -> usize {
        self.corsa_overlays.entry_count()
    }
}
