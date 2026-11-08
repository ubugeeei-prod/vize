//! Incremental cache of the unsaved-buffer overlays handed to Corsa.
//!
//! Corsa resolves a component's imports against the filesystem, so every
//! diagnostics pass must also tell it about buffers the user has edited but not
//! saved. The straightforward way to do that — materialize the text of every
//! open document on every pass — makes one keystroke cost
//! `O(open documents x document size)` in copying before any type checking
//! starts, which is what #3442 is about.
//!
//! Between consecutive passes only the mutation delta needs to be revisited;
//! overlays for the rest are already known. This cache keeps resident
//! `Arc<String>` identities and rebuilds only entries whose
//! [`Document::revision`] moved, turning an unchanged pass into one pointer
//! clone and a small edit into persistent path-copying.
//!
//! [`Document::revision`]: crate::document::Document::revision
#![cfg(feature = "native")]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;
use tower_lsp::lsp_types::Url;
use vize_canon::CorsaOverlaySnapshot;
use vize_l0::FxHashMap;

use super::ServerState;
use crate::document::DocumentStore;

/// One open document's last projectable contribution to the overlay set.
struct OverlayEntry {
    /// Last observed document revision. A structurally incomplete revision can
    /// retain older `text`, but matching this stamp still avoids reparsing it.
    revision: u64,
    /// Pass that last observed this document open, used to evict entries for
    /// documents closed since.
    seen: u64,
    /// Shared so a pass can hand out the text without copying it.
    text: Arc<String>,
}

#[derive(Default)]
struct OverlayCacheState {
    /// Monotonic pass counter; see [`OverlayEntry::seen`].
    pass: u64,
    entries: FxHashMap<Url, OverlayEntry>,
    /// Complete result for one stable open-document set. Repeated language
    /// features can clone this pointer without walking every document again.
    snapshot_revision: Option<u64>,
    journal_cursor: Option<u64>,
    snapshot: CorsaOverlaySnapshot,
}

/// Cached overlay set, refreshed incrementally per diagnostics pass.
#[derive(Default)]
pub(crate) struct CorsaOverlayCache {
    state: RwLock<OverlayCacheState>,
    /// Count of documents whose text this cache has materialized. Rises by one
    /// per changed document per pass once warm, which is the property the
    /// perf tests assert (a wall-clock bound could not).
    materializations: AtomicU64,
    #[cfg(test)]
    document_visits: AtomicU64,
    #[cfg(test)]
    snapshot_rebuilds: AtomicU64,
}

impl CorsaOverlayCache {
    /// The last projectable overlay set for the currently open documents.
    ///
    /// Reuses the cached text of every document whose revision is unchanged and
    /// materializes only the rest, so a pass following a single edit copies one
    /// document rather than all of them. An unprojectable revision retains the
    /// previous bytes; a newly opened unprojectable document is omitted.
    /// Documents opened since the last pass are added, documents closed since
    /// are dropped.
    ///
    /// The returned snapshot is owned: it borrows nothing from the document
    /// store and holds no lock, so callers may keep it across the `.await` of a
    /// bridge call without parking the executor thread the next `didOpen` /
    /// `didChange` / `didClose` needs (#3315, #3377).
    pub(crate) fn snapshot(
        &self,
        documents: &DocumentStore,
        projectable: impl Fn(&Url, &str) -> bool,
    ) -> CorsaOverlaySnapshot {
        let revision = documents.stable_revision();
        let mut state = self.state.write();
        if revision.is_some()
            && state.snapshot_revision == revision
            && documents.stable_revision() == revision
        {
            return state.snapshot.clone();
        }
        let pass = state.pass.wrapping_add(1);
        state.pass = pass;

        let delta = revision
            .zip(state.journal_cursor)
            .and_then(|(_, cursor)| documents.overlay_changes_since(cursor));
        let (snapshot, journal_cursor) = if let Some(delta) = delta {
            let mut upserts = Vec::with_capacity(delta.uris.len());
            let mut removals = Vec::new();
            let changed = delta.uris.into_iter().collect::<vize_l0::FxHashSet<_>>();
            for uri in changed {
                let Some(document) = documents.get(&uri) else {
                    state.entries.remove(&uri);
                    if let Ok(path) = uri.to_file_path() {
                        removals.push(path);
                    }
                    continue;
                };
                let Ok(path) = uri.to_file_path() else {
                    state.entries.remove(&uri);
                    continue;
                };
                match self.overlay_for_document(
                    &mut state,
                    &uri,
                    document.value(),
                    pass,
                    &projectable,
                ) {
                    Some(text) => upserts.push((path, text)),
                    None => removals.push(path),
                }
            }
            (state.snapshot.revise(pass, upserts, removals), delta.cursor)
        } else {
            let mut overlays = Vec::with_capacity(documents.len());
            for document in documents.iter() {
                let Ok(path) = document.key().to_file_path() else {
                    continue;
                };
                if let Some(text) = self.overlay_for_document(
                    &mut state,
                    document.key(),
                    document.value(),
                    pass,
                    &projectable,
                ) {
                    overlays.push((path, text));
                }
            }
            state.entries.retain(|_, entry| entry.seen == pass);
            (
                CorsaOverlaySnapshot::new(pass, overlays),
                documents.overlay_change_cursor(),
            )
        };
        if revision.is_some()
            && documents.stable_revision() == revision
            && documents.overlay_change_cursor() == journal_cursor
        {
            state.snapshot_revision = revision;
            state.journal_cursor = Some(journal_cursor);
            state.snapshot = snapshot.clone();
            documents.acknowledge_overlay_changes(journal_cursor);
        } else {
            state.snapshot_revision = None;
        }
        #[cfg(test)]
        self.snapshot_rebuilds.fetch_add(1, Ordering::Relaxed);
        snapshot
    }

    fn overlay_for_document(
        &self,
        state: &mut OverlayCacheState,
        uri: &Url,
        document: &crate::document::Document,
        pass: u64,
        projectable: &impl Fn(&Url, &str) -> bool,
    ) -> Option<Arc<String>> {
        #[cfg(test)]
        self.document_visits.fetch_add(1, Ordering::Relaxed);
        let revision = document.revision();
        if let Some(entry) = state.entries.get_mut(uri)
            && entry.revision == revision
        {
            entry.seen = pass;
            return Some(Arc::clone(&entry.text));
        }
        let text = document.text_snapshot();
        self.materializations.fetch_add(1, Ordering::Relaxed);
        if projectable(uri, &text) {
            state.entries.insert(
                uri.clone(),
                OverlayEntry {
                    revision,
                    seen: pass,
                    text: Arc::clone(&text),
                },
            );
            Some(text)
        } else if let Some(entry) = state.entries.get_mut(uri) {
            // Keep the last projectable bytes while this revision is
            // structurally incomplete.
            entry.revision = revision;
            entry.seen = pass;
            Some(Arc::clone(&entry.text))
        } else {
            None
        }
    }

    /// Drop every cached overlay.
    ///
    /// Only ever costs a rebuild: entries are keyed by revision, so a stale
    /// entry cannot survive invalidation being missed. Called where the
    /// resolution the overlays feed can change out from under them.
    pub(crate) fn invalidate(&self) {
        let mut state = self.state.write();
        state.entries.clear();
        state.snapshot_revision = None;
        state.journal_cursor = None;
        state.snapshot = CorsaOverlaySnapshot::default();
    }

    /// Stop retaining the buffer for a document removed from the store.
    pub(super) fn remove(&self, uri: &Url) {
        let mut state = self.state.write();
        state.entries.remove(uri);
        state.snapshot_revision = None;
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

    #[cfg(test)]
    pub(crate) fn snapshot_rebuilds(&self) -> u64 {
        self.snapshot_rebuilds.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(crate) fn document_visits(&self) -> u64 {
        self.document_visits.load(Ordering::Relaxed)
    }
}

impl ServerState {
    /// The last projectable Corsa overlay set for the currently open documents.
    ///
    /// See [`CorsaOverlayCache::snapshot`].
    pub(crate) fn corsa_overlays(&self) -> CorsaOverlaySnapshot {
        self.corsa_overlays.snapshot(&self.documents, |uri, text| {
            !uri.path().ends_with(".vue")
                || matches!(
                    self.sfc_parsed(uri, text),
                    vize_incremental::ParsedSfc::Descriptor(_)
                )
        })
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

    #[cfg(test)]
    pub(crate) fn corsa_overlay_snapshot_rebuilds(&self) -> u64 {
        self.corsa_overlays.snapshot_rebuilds()
    }

    #[cfg(test)]
    pub(crate) fn corsa_overlay_document_visits(&self) -> u64 {
        self.corsa_overlays.document_visits()
    }
}
