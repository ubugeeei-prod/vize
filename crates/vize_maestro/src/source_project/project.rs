//! Actual host lifecycle updates cancel their retained source-query work.
#![expect(
    clippy::disallowed_types,
    reason = "the actual LSP host receives std String and suspended query leases share Arc state"
)]

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
pub use tracked::{ProjectQuery, ProjectQueryResult};

struct ActiveQuery {
    id: u64,
    uri: Url,
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
    documents: &'host DocumentStore,
    cache: SourceSnapshotCache,
    active: Arc<ActiveQueries>,
    next_query: AtomicU64,
    lifecycle: Mutex<()>,
}

impl<'host> SourceQueryProject<'host> {
    pub fn new(documents: &'host DocumentStore) -> Self {
        Self {
            documents,
            cache: SourceSnapshotCache::default(),
            active: Arc::new(ActiveQueries::default()),
            next_query: AtomicU64::new(0),
            lifecycle: Mutex::new(()),
        }
    }

    /// Capture and register under the same synchronous lifecycle boundary.
    pub fn begin_query(
        &self,
        uri: &Url,
    ) -> Result<(ProjectQuery<'host>, AbortHandle), SnapshotRefusal> {
        let _lifecycle = self.lifecycle.lock();
        let (query, cancel) = self.cache.begin_query(self.documents, uri)?;
        let id = self.next_query.fetch_add(1, Ordering::Relaxed);
        self.active.0.lock().push(ActiveQuery {
            id,
            uri: uri.clone(),
            cancel: cancel.clone(),
        });
        Ok((
            ProjectQuery::new(query, self.documents, Arc::clone(&self.active), id),
            cancel,
        ))
    }

    pub fn open(&self, uri: Url, source: String, version: i32, language_id: String) {
        let aborted = {
            let _lifecycle = self.lifecycle.lock();
            self.documents
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
                .documents
                .get(uri)
                .map(|document| (document.revision(), document.version));
            if !self.documents.apply_changes(uri, changes, version) {
                return false;
            }
            let after = self
                .documents
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
            self.documents.close(uri);
            self.invalidate(uri)
        };
        abort(aborted);
    }

    pub fn rename(&self, old_uri: &Url, new_uri: Url) -> bool {
        let aborted = {
            let _lifecycle = self.lifecycle.lock();
            if !self.documents.rename(old_uri, new_uri.clone()) {
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
