//! Owned per-document diagnostic locks, released before transport writes.

use super::ServerState;
use futures::lock::Mutex as AsyncMutex;
use std::sync::Arc;
use tower_lsp::lsp_types::Url;

impl ServerState {
    /// Owned per-document lock for a diagnostic pass. Clone the `Arc` before
    /// awaiting so no DashMap guard survives across a suspension point.
    pub(crate) fn diagnostic_lock(&self, uri: &Url) -> Arc<AsyncMutex<()>> {
        self.diagnostic_locks
            .entry(uri.clone())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    }

    pub(super) fn remove_idle_diagnostic_lock(&self, uri: &Url) {
        if let dashmap::mapref::entry::Entry::Occupied(entry) =
            self.diagnostic_locks.entry(uri.clone())
            && Arc::strong_count(entry.get()) == 1
        {
            entry.remove();
        }
    }
}
