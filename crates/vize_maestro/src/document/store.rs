//! Document store implementation using Rope for efficient text operations.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "project owners share buffers and protocol consumers require std String"
)]

mod structural_navigation;
mod uri_snapshot;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use dashmap::DashMap;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use self::uri_snapshot::UriSnapshotCache;
use super::DocumentText;
use super::edit_cancel::EditCancellation;
use crate::ide::references::StructuralSnapshotCell;
use crate::utils::position_to_offset;

/// Source of [`Document::revision`] stamps.
///
/// Process-global and monotonic, so a stamp identifies one exact content
/// snapshot across the whole server for the lifetime of the process. The
/// client-supplied `version` cannot do this job: it is chosen by the editor,
/// resets on reopen, and is reused across documents, so equal versions do not
/// imply equal text.
static NEXT_DOCUMENT_REVISION: AtomicU64 = AtomicU64::new(1);

fn next_revision() -> u64 {
    NEXT_DOCUMENT_REVISION.fetch_add(1, Ordering::Relaxed)
}

/// A document managed by the LSP server.
#[derive(Debug)]
pub struct Document {
    /// Document URI
    pub uri: Url,
    /// Document version
    pub version: i32,
    /// Document content stored as a rope for efficient editing
    content: DocumentText,
    /// Language ID (e.g., "vue", "typescript")
    pub language_id: String,
    /// Monotonic stamp for the exact content this document currently holds.
    ///
    /// Bumped by every mutation. Caches that key derived data on a document
    /// compare this instead of `version`: an unchanged stamp is proof the
    /// content is byte-identical, which is what makes reusing a cached
    /// snapshot safe rather than merely likely.
    revision: u64,
    /// Lazily computed structural petite-vue detection result, memoized per
    /// document version so per-request consumers (completion, definition, ...)
    /// never re-scan the content. Reset on every edit.
    petite_vue_detected: OnceLock<bool>,
    /// Flat immutable text shared by every request for this exact revision.
    ///
    /// Rope remains the edit/coordinate owner. This snapshot exists because
    /// request futures must release the `DashMap` guard before `.await`; an
    /// `Arc` preserves that safety without flattening the same Rope once per
    /// hover, completion, navigation, or rename request.
    text_snapshot: OnceLock<Arc<String>>,
    structural_snapshot: OnceLock<Arc<StructuralSnapshotCell>>,
}

impl Document {
    /// Create a new document.
    pub fn new(uri: Url, content: String, version: i32, language_id: String) -> Self {
        let content = Arc::new(content);
        let text_snapshot = OnceLock::new();
        let _ = text_snapshot.set(Arc::clone(&content));
        Self {
            uri,
            version,
            content: DocumentText::new(content.as_str()),
            language_id,
            revision: next_revision(),
            petite_vue_detected: OnceLock::new(),
            text_snapshot,
            structural_snapshot: OnceLock::new(),
        }
    }

    /// Monotonic stamp for the content this document currently holds.
    ///
    /// Two reads that return the same value observed the same text; a bumped
    /// value means the text may differ. Never reused across documents.
    #[inline]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Whether the document structurally uses petite-vue (see
    /// [`vize_l0::dialect::detect_petite_vue_document`]).
    ///
    /// Computed at most once per document version; edits invalidate the cache.
    pub fn petite_vue_detected(&self) -> bool {
        *self
            .petite_vue_detected
            .get_or_init(|| vize_l0::dialect::detect_petite_vue_document(self.flat_text()))
    }

    /// Get the document content as a string.
    pub fn text(&self) -> String {
        self.flat_text().to_owned()
    }

    /// Whether `source` is byte-identical to this document revision.
    #[inline]
    pub fn has_text(&self, source: &str) -> bool {
        self.flat_text() == source
    }

    /// UTF-8 byte length without flattening the resident rope.
    #[inline]
    pub fn byte_len(&self) -> usize {
        self.content.len_bytes()
    }

    /// Shared owned text for this exact document revision.
    pub fn text_snapshot(&self) -> Arc<String> {
        Arc::clone(
            self.text_snapshot
                .get_or_init(|| Arc::new(self.content.to_string())),
        )
    }

    fn flat_text(&self) -> &str {
        self.text_snapshot
            .get_or_init(|| Arc::new(self.content.to_string()))
            .as_str()
    }

    /// Get the number of lines in the document.
    pub fn line_count(&self) -> usize {
        self.content.len_lines()
    }

    /// Get a specific line (0-indexed).
    pub fn line(&self, line_idx: usize) -> Option<String> {
        if line_idx >= self.content.len_lines() {
            return None;
        }
        Some(self.content.line(line_idx).to_string())
    }

    /// Apply an incremental change to the document.
    pub fn apply_change(&mut self, change: &TextDocumentContentChangeEvent, new_version: i32) {
        self.version = new_version;
        self.revision = next_revision();
        self.petite_vue_detected = OnceLock::new();
        self.text_snapshot = OnceLock::new();
        self.structural_snapshot = OnceLock::new();

        if let Some(range) = change.range {
            // Incremental change
            let start_offset = position_to_offset(&self.content, range.start);
            let end_offset = position_to_offset(&self.content, range.end);

            if let (Some(start), Some(end)) = (start_offset, end_offset) {
                // Convert byte offsets to char indices
                if let (Ok(start_char), Ok(end_char)) = (
                    self.content.try_byte_to_char(start),
                    self.content.try_byte_to_char(end),
                ) {
                    if start_char > end_char {
                        return;
                    }

                    self.content.replace(start_char..end_char, &change.text);
                }
            }
        } else {
            // Full content replacement
            self.content = DocumentText::new(&change.text);
            let _ = self.text_snapshot.set(Arc::new(change.text.clone()));
        }
    }
}

/// Guard-free request input captured from one exact document revision.
pub(crate) struct DocumentPositionSnapshot {
    pub(crate) content: Arc<String>,
    pub(crate) offset: usize,
}

/// Thread-safe document store.
#[derive(Clone)]
pub struct DocumentStore {
    documents: Arc<DashMap<Url, Document>>,
    uri_snapshots: Arc<UriSnapshotCache>,
    revision: Arc<AtomicU64>,
    mutations: Arc<AtomicUsize>,
    edits: Arc<EditCancellation>,
}

impl Default for DocumentStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentStore {
    /// Create a new document store.
    pub fn new() -> Self {
        Self {
            documents: Arc::new(DashMap::new()),
            uri_snapshots: Arc::default(),
            revision: Arc::new(AtomicU64::new(0)),
            mutations: Arc::new(AtomicUsize::new(0)),
            edits: Arc::default(),
        }
    }

    /// Open a new document.
    pub fn open(&self, uri: Url, content: String, version: i32, language_id: String) {
        let _mutation = self.source_set_mutation();
        let uri_set = self.uri_snapshots.mutation(&uri, None);
        let doc = Document::new(uri.clone(), content, version, language_id);
        self.documents.insert(uri.clone(), doc);
        self.revision.fetch_add(1, Ordering::Release);
        uri_set.opened(uri);
    }

    /// Close a document.
    pub fn close(&self, uri: &Url) {
        if !self.documents.contains_key(uri) {
            return;
        }
        let _mutation = self.source_set_mutation();
        let uri_set = self.uri_snapshots.mutation(uri, None);
        if self.documents.remove(uri).is_some() {
            self.revision.fetch_add(1, Ordering::Release);
            uri_set.closed(uri);
        }
    }

    /// Rename an open document while preserving its content and version.
    pub fn rename(&self, old_uri: &Url, new_uri: Url) -> bool {
        if old_uri == &new_uri {
            return self.documents.contains_key(old_uri);
        }

        if self.documents.contains_key(&new_uri) {
            return false;
        }

        if !self.documents.contains_key(old_uri) {
            return false;
        }
        let _mutation = self.source_set_mutation();
        let uri_set = self.uri_snapshots.mutation(old_uri, Some(&new_uri));

        // The optimistic checks above avoid stamping obvious no-ops. Repeat
        // them after acquiring the URI stripes so conflicting key mutations
        // and their cache commits have one order.
        if self.documents.contains_key(&new_uri) || !self.documents.contains_key(old_uri) {
            return false;
        }

        let Some((_, mut document)) = self.documents.remove(old_uri) else {
            return false;
        };

        document.uri = new_uri.clone();
        document.structural_snapshot = OnceLock::new();
        // A rename moves the same text to a new path. Overlay consumers key on
        // the path, so the entry they cached under the old URI must not be
        // mistaken for this one; a fresh stamp keeps the two unrelatable.
        document.revision = next_revision();
        self.documents.insert(new_uri.clone(), document);
        self.revision.fetch_add(1, Ordering::Release);
        uri_set.renamed(old_uri, new_uri);
        true
    }

    /// Get a document by URI.
    pub fn get(&self, uri: &Url) -> Option<dashmap::mapref::one::Ref<'_, Url, Document>> {
        self.documents.get(uri)
    }

    /// Get a mutable reference to a document.
    pub fn get_mut(&self, uri: &Url) -> Option<dashmap::mapref::one::RefMut<'_, Url, Document>> {
        let mut document = self.documents.get_mut(uri)?;
        document.revision = next_revision();
        document.petite_vue_detected = OnceLock::new();
        document.text_snapshot = OnceLock::new();
        document.structural_snapshot = OnceLock::new();
        self.revision.fetch_add(1, Ordering::Release);
        Some(document)
    }

    /// Owned snapshot of a document's text, holding no lock on return.
    ///
    /// Prefer this over [`Self::get`] whenever the text outlives a statement:
    /// [`Self::get`] hands back a `DashMap` shard read guard, and the LSP runs
    /// every handler on one executor thread. A guard that is still alive at an
    /// `.await` therefore blocks the whole server the moment a concurrently
    /// polled handler writes the same shard — `apply_changes`/`open`/`close`
    /// all park forever on the shard's write lock, which the suspended reader
    /// can only release by being polled on that same parked thread (#3315).
    pub fn text(&self, uri: &Url) -> Option<String> {
        self.documents.get(uri).map(|document| document.text())
    }

    /// Shared immutable text for one exact revision, with no shard guard on
    /// return. Cloning the `Arc` is allocation-free on a warm revision.
    pub fn text_snapshot(&self, uri: &Url) -> Option<Arc<String>> {
        self.documents
            .get(uri)
            .map(|document| document.text_snapshot())
    }

    /// Capture text and an exact UTF-16 LSP position under one document guard.
    /// Ropey owns the coordinate conversion; requests avoid a second full-text
    /// line scan and release the map guard before any async backend work.
    pub(crate) fn snapshot_at(
        &self,
        uri: &Url,
        position: tower_lsp::lsp_types::Position,
    ) -> Option<DocumentPositionSnapshot> {
        let document = self.documents.get(uri)?;
        let offset = position_to_offset(&document.content, position)?;
        Some(DocumentPositionSnapshot {
            content: document.text_snapshot(),
            offset,
        })
    }

    /// Owned snapshot of a document's version, holding no lock on return.
    ///
    /// Same lock discipline as [`Self::text`]: this is the version comparison a
    /// diagnostics pass makes between its `.await` points to notice that a newer
    /// `didChange` has superseded it, so it must not leave a shard guard behind.
    pub fn version(&self, uri: &Url) -> Option<i32> {
        self.documents.get(uri).map(|document| document.version)
    }

    /// Apply changes to a document.
    pub fn apply_changes(
        &self,
        uri: &Url,
        changes: Vec<TextDocumentContentChangeEvent>,
        version: i32,
    ) -> bool {
        let Some(mut doc) = self.documents.get_mut(uri) else {
            return false;
        };
        // tower-lsp polls notification handlers concurrently. Preserve the
        // LSP version order even if a newer didChange reaches the store first.
        if version <= doc.version {
            return false;
        }
        let _mutation = self.source_set_mutation();
        for change in changes {
            doc.apply_change(&change, version);
        }
        self.revision.fetch_add(1, Ordering::Release);
        true
    }

    /// Stamp of the open source set, including close and rename operations.
    pub(crate) fn revision(&self) -> u64 {
        self.revision.load(Ordering::Acquire)
    }

    /// Capture only between complete foreground source-set mutations. A stamp
    /// read during an insert/change/close cannot bind the partially updated set.
    #[cfg(feature = "native")]
    pub(crate) fn stable_revision(&self) -> Option<u64> {
        if self.mutations.load(Ordering::Acquire) != 0 {
            return None;
        }
        let revision = self.revision();
        (self.mutations.load(Ordering::Acquire) == 0 && revision == self.revision())
            .then_some(revision)
    }

    /// Run a request until it completes, or `None` once any open source is
    /// opened, edited, renamed or closed.
    pub(crate) async fn until_edited<F: std::future::Future>(
        &self,
        future: F,
    ) -> Option<F::Output> {
        self.edits.until_edited(future).await
    }

    fn source_set_mutation(&self) -> SourceSetMutation<'_> {
        self.mutations.fetch_add(1, Ordering::AcqRel);
        self.revision.fetch_add(1, Ordering::Release);
        SourceSetMutation(self)
    }

    /// Check if a document exists.
    pub fn contains(&self, uri: &Url) -> bool {
        self.documents.contains_key(uri)
    }

    /// Get a sorted immutable snapshot of all open document URIs.
    ///
    /// Completed key mutations advance the snapshot generation. In-flight
    /// mutations remain invisible until their complete delta is published, so
    /// stale `Arc`s stay valid without exposing rename remove/insert windows.
    pub fn uris(&self) -> Arc<[Url]> {
        vize_l0::micro_count!("lsp.document_store.uris.calls", 1);
        let snapshot = vize_l0::micro_span!(
            "lsp.document_store.uris.clone_snapshot",
            self.uri_snapshots.snapshot()
        );
        vize_l0::micro_count!("lsp.document_store.uris.returned_items", snapshot.len());
        vize_l0::micro_count!(
            "lsp.document_store.uris.returned_uri_utf8_bytes",
            snapshot
                .iter()
                .map(|uri| uri.as_str().len() as u64)
                .sum::<u64>()
        );
        snapshot
    }

    /// Owned snapshots of open Vue documents, holding no shard guard on return.
    ///
    /// Workspace-wide editor features parse these snapshots after the map read
    /// has finished. Keeping a `DashMap` guard alive while parsing is not an
    /// `.await`, but it can still stall other LSP handlers that need the same
    /// shard's write lock and then make later readers wait behind that writer.
    pub fn vue_texts(&self) -> Vec<(Url, String)> {
        vize_l0::micro_count!("lsp.document_store.vue_texts.calls", 1);
        let snapshots = vize_l0::micro_span!("lsp.document_store.vue_texts.clone_snapshot", {
            let mut snapshots = self
                .documents
                .iter()
                .filter_map(|document| {
                    let uri = document.key();
                    uri.path()
                        .ends_with(".vue")
                        .then(|| (uri.clone(), document.value().text()))
                })
                .collect::<Vec<_>>();
            snapshots.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));
            snapshots
        });
        vize_l0::micro_count!(
            "lsp.document_store.vue_texts.returned_items",
            snapshots.len()
        );
        vize_l0::micro_count!(
            "lsp.document_store.vue_texts.returned_uri_utf8_bytes",
            snapshots
                .iter()
                .map(|(uri, _)| uri.as_str().len() as u64)
                .sum::<u64>()
        );
        vize_l0::micro_count!(
            "lsp.document_store.vue_texts.returned_text_utf8_bytes",
            snapshots
                .iter()
                .map(|(_, text)| text.len() as u64)
                .sum::<u64>()
        );
        snapshots
    }

    /// Get the number of open documents.
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Iterate over all documents.
    pub fn iter(&self) -> dashmap::iter::Iter<'_, Url, Document> {
        self.documents.iter()
    }
}

struct SourceSetMutation<'a>(&'a DocumentStore);
impl Drop for SourceSetMutation<'_> {
    fn drop(&mut self) {
        self.0.revision.fetch_add(1, Ordering::Release);
        self.0.mutations.fetch_sub(1, Ordering::Release);
        self.0.edits.cancel();
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod uri_snapshot_tests;
#[cfg(test)]
mod version_tests;
