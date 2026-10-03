//! Actual host lifecycle updates cancel their retained source-query work.
#![expect(
    clippy::disallowed_types,
    reason = "the actual LSP host receives std String and suspended query leases share Arc state"
)]

mod host;
mod notify;
#[cfg(test)]
mod tests;
mod tracked;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use futures::future::AbortHandle;
use parking_lot::Mutex;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use crate::document::DocumentStore;

use super::{SnapshotRefusal, SourceSnapshotCache};
use host::DocumentHost;
pub use tracked::{ProjectQuery, ProjectQueryResult};

struct ActiveQuery {
    id: u64,
    uri: Url,
    revision: u64,
    cancel: AbortHandle,
}

#[derive(Default)]
struct ActiveQueries(Mutex<Vec<ActiveQuery>>);

impl ActiveQueries {
    fn remove_for(&self, uri: &Url) -> Vec<AbortHandle> {
        let mut removed = Vec::new();
        self.0.lock().retain(|query| {
            if &query.uri == uri {
                removed.push(query.cancel.clone());
                false
            } else {
                true
            }
        });
        removed
    }
}

/// Scoped source API over the caller's genuine open-document store.
///
/// Use these lifecycle methods for source queries owned by this project. Direct
/// external store mutations are still refused by final snapshot checks, but do
/// not notify this project's suspended work. No document guard or lifecycle lock
/// survives an await. Production request dispatch is not selected by this type.
pub struct SourceQueryProject<'host> {
    host: DocumentHost<'host>,
    cache: SourceSnapshotCache,
    active: Arc<ActiveQueries>,
    next_query: AtomicU64,
    lifecycle: Mutex<()>,
}

impl<'host> SourceQueryProject<'host> {
    pub fn new(documents: &'host DocumentStore) -> Self {
        Self::with_host(DocumentHost::Borrowed(documents))
    }

    /// Capture and register under the same synchronous lifecycle boundary.
    pub fn begin_query(
        &self,
        uri: &Url,
    ) -> Result<(ProjectQuery<'host>, AbortHandle), SnapshotRefusal> {
        let _lifecycle = self.lifecycle.lock();
        let (query, cancel) = self.cache.begin_query(self.host.documents(), uri)?;
        let id = self.next_query.fetch_add(1, Ordering::Relaxed);
        self.active.0.lock().push(ActiveQuery {
            id,
            uri: uri.clone(),
            revision: query.snapshot().revision(),
            cancel: cancel.clone(),
        });
        Ok((
            ProjectQuery::new(query, self.host.clone(), Arc::clone(&self.active), id),
            cancel,
        ))
    }

    pub fn open(&self, uri: Url, source: String, version: i32, language_id: String) {
        let aborted = {
            let _lifecycle = self.lifecycle.lock();
            self.host
                .documents()
                .open(uri.clone(), source, version, language_id);
            self.invalidate(&uri)
        };
        abort(aborted);
    }

    /// Preserve the host's version ordering and exact incremental UTF-16 edits.
    pub fn apply_changes(
        &self,
        uri: &Url,
        changes: Vec<TextDocumentContentChangeEvent>,
        version: i32,
    ) -> bool {
        let aborted = {
            let _lifecycle = self.lifecycle.lock();
            let before = self
                .host
                .documents()
                .get(uri)
                .map(|document| (document.revision(), document.version));
            if !self.host.documents().apply_changes(uri, changes, version) {
                return false;
            }
            let after = self
                .host
                .documents()
                .get(uri)
                .map(|document| (document.revision(), document.version));
            // The actual host accepts a newer empty list without changing this
            // document's key/version. Preserve its physical buffer and work.
            if before == after {
                return true;
            }
            self.invalidate(uri)
        };
        abort(aborted);
        true
    }

    pub fn close(&self, uri: &Url) {
        let aborted = {
            let _lifecycle = self.lifecycle.lock();
            self.host.documents().close(uri);
            self.invalidate(uri)
        };
        abort(aborted);
    }

    pub fn rename(&self, old_uri: &Url, new_uri: Url) -> bool {
        let aborted = {
            let _lifecycle = self.lifecycle.lock();
            if !self.host.documents().rename(old_uri, new_uri.clone()) {
                return false;
            }
            if old_uri == &new_uri {
                return true;
            }
            let mut aborted = self.invalidate(old_uri);
            aborted.extend(self.invalidate(&new_uri));
            aborted
        };
        abort(aborted);
        true
    }

    fn invalidate(&self, uri: &Url) -> Vec<AbortHandle> {
        self.cache.forget(uri);
        self.active.remove_for(uri)
    }
}

impl SourceQueryProject<'static> {
    /// Retain the caller's exact shared open-document store through queries.
    pub fn new_shared(documents: Arc<DocumentStore>) -> Self {
        Self::with_host(DocumentHost::SharedStore(documents))
    }

    /// Retain the actual server owner and borrow its original inline store.
    /// Store this controller beside the server state, never inside that owner.
    /// This prevents a reference cycle; no new store or self-reference is made.
    pub fn new_server(state: Arc<crate::server::ServerState>) -> Self {
        Self::with_host(DocumentHost::Server(state))
    }
}

impl Drop for SourceQueryProject<'_> {
    fn drop(&mut self) {
        let aborted = self
            .active
            .0
            .lock()
            .drain(..)
            .map(|query| query.cancel)
            .collect();
        abort(aborted);
    }
}

fn abort(aborted: Vec<AbortHandle>) {
    // A waker may invoke arbitrary code: release every lifecycle/registry lock
    // before waking work. Newly registered queries are absent from this list.
    for cancel in aborted {
        cancel.abort();
    }
}
