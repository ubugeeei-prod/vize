//! Document store implementation using Rope for efficient text operations.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

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
    pub content: Rope,
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
}

impl Document {
    /// Create a new document.
    pub fn new(uri: Url, content: String, version: i32, language_id: String) -> Self {
        Self {
            uri,
            version,
            content: Rope::from_str(&content),
            language_id,
            revision: next_revision(),
            petite_vue_detected: OnceLock::new(),
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
    /// [`vize_s0::dialect::detect_petite_vue_document`]).
    ///
    /// Computed at most once per document version; edits invalidate the cache.
    pub fn petite_vue_detected(&self) -> bool {
        *self
            .petite_vue_detected
            .get_or_init(|| vize_s0::dialect::detect_petite_vue_document(&self.text()))
    }

    /// Get the document content as a string.
    pub fn text(&self) -> String {
        self.content.to_string()
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

                    self.content.remove(start_char..end_char);
                    self.content.insert(start_char, &change.text);
                }
            }
        } else {
            // Full content replacement
            self.content = Rope::from_str(&change.text);
        }
    }
}

/// Thread-safe document store.
pub struct DocumentStore {
    documents: DashMap<Url, Document>,
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
            documents: DashMap::new(),
        }
    }

    /// Open a new document.
    pub fn open(&self, uri: Url, content: String, version: i32, language_id: String) {
        let doc = Document::new(uri.clone(), content, version, language_id);
        self.documents.insert(uri, doc);
    }

    /// Close a document.
    pub fn close(&self, uri: &Url) {
        self.documents.remove(uri);
    }

    /// Rename an open document while preserving its content and version.
    pub fn rename(&self, old_uri: &Url, new_uri: Url) -> bool {
        if old_uri == &new_uri {
            return self.documents.contains_key(old_uri);
        }

        if self.documents.contains_key(&new_uri) {
            return false;
        }

        let Some((_, mut document)) = self.documents.remove(old_uri) else {
            return false;
        };

        document.uri = new_uri.clone();
        // A rename moves the same text to a new path. Overlay consumers key on
        // the path, so the entry they cached under the old URI must not be
        // mistaken for this one; a fresh stamp keeps the two unrelatable.
        document.revision = next_revision();
        self.documents.insert(new_uri, document);
        true
    }

    /// Get a document by URI.
    pub fn get(&self, uri: &Url) -> Option<dashmap::mapref::one::Ref<'_, Url, Document>> {
        self.documents.get(uri)
    }

    /// Get a mutable reference to a document.
    pub fn get_mut(&self, uri: &Url) -> Option<dashmap::mapref::one::RefMut<'_, Url, Document>> {
        self.documents.get_mut(uri)
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
        for change in changes {
            doc.apply_change(&change, version);
        }
        true
    }

    /// Check if a document exists.
    pub fn contains(&self, uri: &Url) -> bool {
        self.documents.contains_key(uri)
    }

    /// Get all document URIs.
    pub fn uris(&self) -> Vec<Url> {
        self.documents.iter().map(|r| r.key().clone()).collect()
    }

    /// Owned snapshots of open Vue documents, holding no shard guard on return.
    ///
    /// Workspace-wide editor features parse these snapshots after the map read
    /// has finished. Keeping a `DashMap` guard alive while parsing is not an
    /// `.await`, but it can still stall other LSP handlers that need the same
    /// shard's write lock and then make later readers wait behind that writer.
    pub fn vue_texts(&self) -> Vec<(Url, String)> {
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

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod version_tests;
