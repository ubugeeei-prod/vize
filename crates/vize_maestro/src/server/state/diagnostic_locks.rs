//! Owned per-document diagnostic locks, released before transport writes.

use super::{CorsaRequestStamp, ServerState};
use futures::lock::{Mutex as AsyncMutex, MutexGuard};
use parking_lot::Mutex;
use std::sync::Arc;
use tower_lsp::lsp_types::{Diagnostic, Url};

#[derive(Default)]
pub(crate) struct DiagnosticDocument {
    collection: AsyncMutex<()>,
    publication: Mutex<Option<Arc<DiagnosticPublication>>>,
}

struct DiagnosticPublication {
    version: i32,
    stamp: CorsaRequestStamp,
    diagnostics: Vec<Diagnostic>,
}

pub(crate) struct DiagnosticPublicationGuard {
    document: Arc<DiagnosticDocument>,
    publication: Arc<DiagnosticPublication>,
    finished: bool,
}

pub(crate) struct DiagnosticSourceChanged;

impl DiagnosticDocument {
    pub(crate) async fn lock(&self) -> MutexGuard<'_, ()> {
        self.collection.lock().await
    }

    /// Retain one complete payload in the existing per-document lock entry.
    /// No synchronous guard survives transport backpressure. A cancelled
    /// sender removes only its own claim; a newer source world's claim wins.
    pub(crate) fn claim(
        self: &Arc<Self>,
        state: &ServerState,
        uri: &Url,
        version: i32,
        stamp: CorsaRequestStamp,
        diagnostics: &[Diagnostic],
        explicit_save: bool,
    ) -> Result<Option<DiagnosticPublicationGuard>, DiagnosticSourceChanged> {
        let mut slot = self.publication.lock();
        // A newer world may have claimed this slot since the caller's early
        // check. Refuse the stale owner before touching that newer claim.
        if !stamp.is_current(state) || state.documents.version(uri) != Some(version) {
            return Err(DiagnosticSourceChanged);
        }
        if !explicit_save
            && slot.as_ref().is_some_and(|publication| {
                publication.version == version
                    && publication.stamp == stamp
                    && publication.diagnostics == diagnostics
            })
        {
            return Ok(None);
        }
        let publication = Arc::new(DiagnosticPublication {
            version,
            stamp,
            diagnostics: diagnostics.to_vec(),
        });
        *slot = Some(Arc::clone(&publication));
        Ok(Some(DiagnosticPublicationGuard {
            document: Arc::clone(self),
            publication,
            finished: false,
        }))
    }
}

impl DiagnosticPublicationGuard {
    pub(crate) fn finish(mut self) {
        self.finished = true;
    }
}

impl Drop for DiagnosticPublicationGuard {
    fn drop(&mut self) {
        if !self.finished {
            let mut slot = self.document.publication.lock();
            if slot
                .as_ref()
                .is_some_and(|current| Arc::ptr_eq(current, &self.publication))
            {
                *slot = None;
            }
        }
    }
}

impl ServerState {
    /// Owned per-document lock for a diagnostic pass. Clone the `Arc` before
    /// awaiting so no DashMap guard survives across a suspension point.
    pub(crate) fn diagnostic_lock(&self, uri: &Url) -> Arc<DiagnosticDocument> {
        self.diagnostic_locks
            .entry(uri.clone())
            .or_insert_with(|| Arc::new(DiagnosticDocument::default()))
            .clone()
    }

    pub(super) fn remove_idle_diagnostic_lock(&self, uri: &Url) {
        if let dashmap::mapref::entry::Entry::Occupied(entry) =
            self.diagnostic_locks.entry(uri.clone())
        {
            entry.get().publication.lock().take();
            if Arc::strong_count(entry.get()) == 1 {
                entry.remove();
            }
        }
    }
}
