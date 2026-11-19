//! Immutable open-document text snapshots.
#![expect(
    clippy::disallowed_types,
    reason = "request snapshots share immutable protocol URLs and source buffers"
)]

use std::sync::Arc;

use parking_lot::RwLock;
use tower_lsp::lsp_types::Url;

pub(super) type TextSnapshot = Arc<[(Url, Arc<String>)]>;

#[derive(Default)]
pub(super) struct TextSnapshotCache {
    state: RwLock<Option<VersionedSnapshot>>,
}

struct VersionedSnapshot {
    revision: u64,
    snapshot: TextSnapshot,
}

impl TextSnapshotCache {
    pub(super) fn get(&self, revision: u64) -> Option<TextSnapshot> {
        let state = self.state.read();
        let snapshot = state.as_ref().filter(|state| state.revision == revision)?;
        Some(Arc::clone(&snapshot.snapshot))
    }

    /// Publish one stable document-store revision. Concurrent readers that
    /// built the same revision converge on the first immutable allocation.
    pub(super) fn publish(&self, revision: u64, candidate: TextSnapshot) -> TextSnapshot {
        let mut state = self.state.write();
        if let Some(snapshot) = state.as_ref().filter(|state| state.revision == revision) {
            return Arc::clone(&snapshot.snapshot);
        }
        *state = Some(VersionedSnapshot {
            revision,
            snapshot: Arc::clone(&candidate),
        });
        candidate
    }
}
