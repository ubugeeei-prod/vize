//! One owned source buffer per actual host document revision.
#![expect(
    clippy::disallowed_types,
    reason = "async host snapshots retain std Arc buffers after the document shard guard is released"
)]

use std::convert::Infallible;
use std::sync::Arc;

use dashmap::DashMap;
use tower_lsp::lsp_types::Url;

use crate::document::{Document, DocumentStore};

/// A host refusal is not native syntax or semantic admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotRefusal {
    MissingDocument,
    Superseded,
    Cancelled,
    NativeFileUnavailable,
}

/// Actual host URI and process-global content revision, never editor version alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotKey<'snapshot> {
    pub(super) uri: &'snapshot Url,
    pub(super) revision: u64,
}

/// Source captured once under the host's read guard, then retained without a guard.
#[derive(Debug)]
pub struct SourceSnapshot {
    uri: Url,
    revision: u64,
    version: i32,
    language_id: Arc<str>,
    source: Arc<str>,
}

impl SourceSnapshot {
    #[must_use]
    pub fn uri(&self) -> &Url {
        &self.uri
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub const fn version(&self) -> i32 {
        self.version
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The real host language identifier captured with this source revision.
    #[must_use]
    pub fn language_id(&self) -> &str {
        &self.language_id
    }

    #[must_use]
    pub const fn key(&self) -> SnapshotKey<'_> {
        SnapshotKey {
            uri: &self.uri,
            revision: self.revision,
        }
    }

    /// A full native File producer is still required; host identity cannot mint it.
    /// There is no success value until the genuine provider bridge is integrated.
    pub fn native_file(&self, documents: &DocumentStore) -> Result<Infallible, SnapshotRefusal> {
        self.check_current(documents)?;
        Err(SnapshotRefusal::NativeFileUnavailable)
    }

    /// Compare the actual current document under a new read guard.
    pub fn check_current(&self, documents: &DocumentStore) -> Result<(), SnapshotRefusal> {
        self.with_current(documents, |_| ())
    }

    pub(super) fn with_current<T>(
        &self,
        documents: &DocumentStore,
        action: impl FnOnce(&Document) -> T,
    ) -> Result<T, SnapshotRefusal> {
        let document = documents
            .get(&self.uri)
            .ok_or(SnapshotRefusal::MissingDocument)?;
        if document.uri != self.uri
            || document.revision() != self.revision
            || document.version != self.version
            || document.language_id.as_str() != self.language_id()
        {
            return Err(SnapshotRefusal::Superseded);
        }
        Ok(action(&document))
    }
}

/// Cached snapshots reuse the same physical buffer for the same actual revision.
#[derive(Default)]
pub struct SourceSnapshotCache {
    snapshots: DashMap<Url, Arc<SourceSnapshot>>,
}

impl SourceSnapshotCache {
    pub fn capture(
        &self,
        documents: &DocumentStore,
        uri: &Url,
    ) -> Result<Arc<SourceSnapshot>, SnapshotRefusal> {
        let document = documents.get(uri).ok_or(SnapshotRefusal::MissingDocument)?;
        let mut entry = self.snapshots.entry(uri.clone()).or_insert_with(|| {
            Arc::new(SourceSnapshot {
                uri: document.uri.clone(),
                revision: document.revision(),
                version: document.version,
                language_id: Arc::from(document.language_id.as_str()),
                source: Arc::from(document.text()),
            })
        });
        if entry.revision != document.revision()
            || entry.version != document.version
            || entry.uri != document.uri
            || entry.language_id() != document.language_id.as_str()
        {
            *entry = Arc::new(SourceSnapshot {
                uri: document.uri.clone(),
                revision: document.revision(),
                version: document.version,
                language_id: Arc::from(document.language_id.as_str()),
                source: Arc::from(document.text()),
            });
        }
        Ok(Arc::clone(&entry))
    }

    /// Drop a cache entry after a host lifecycle event, without changing the host.
    pub fn forget(&self, uri: &Url) {
        self.snapshots.remove(uri);
    }

    pub(super) fn forget_superseded(&self, documents: &DocumentStore, uri: &Url) {
        // Match capture's document-before-cache lock order.
        let document = documents.get(uri);
        self.snapshots.remove_if(uri, |_, snapshot| {
            !document.as_ref().is_some_and(|doc| {
                doc.uri == snapshot.uri
                    && doc.revision() == snapshot.revision
                    && doc.version == snapshot.version
                    && doc.language_id.as_str() == snapshot.language_id()
            })
        });
    }
}
