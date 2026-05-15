//! Document store implementation using Rope for efficient text operations.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use crate::utils::position_to_offset;

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
}

impl Document {
    /// Create a new document.
    pub fn new(uri: Url, content: String, version: i32, language_id: String) -> Self {
        Self {
            uri,
            version,
            content: Rope::from_str(&content),
            language_id,
        }
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
        let Some((_, mut document)) = self.documents.remove(old_uri) else {
            return false;
        };

        document.uri = new_uri.clone();
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

    /// Apply changes to a document.
    pub fn apply_changes(
        &self,
        uri: &Url,
        changes: Vec<TextDocumentContentChangeEvent>,
        version: i32,
    ) {
        if let Some(mut doc) = self.documents.get_mut(uri) {
            for change in changes {
                doc.apply_change(&change, version);
            }
        }
    }

    /// Check if a document exists.
    pub fn contains(&self, uri: &Url) -> bool {
        self.documents.contains_key(uri)
    }

    /// Get all document URIs.
    pub fn uris(&self) -> Vec<Url> {
        self.documents.iter().map(|r| r.key().clone()).collect()
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
mod tests {
    use super::{Document, DocumentStore};
    use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};

    fn test_uri() -> Url {
        Url::parse("file:///test.vue").unwrap()
    }

    #[test]
    fn test_document_creation() {
        let doc = Document::new(test_uri(), "hello world".to_string(), 1, "vue".to_string());

        assert_eq!(doc.text(), "hello world");
        assert_eq!(doc.version, 1);
        assert_eq!(doc.language_id, "vue");
    }

    #[test]
    fn test_document_line_count() {
        let doc = Document::new(
            test_uri(),
            "line1\nline2\nline3".to_string(),
            1,
            "vue".to_string(),
        );

        assert_eq!(doc.line_count(), 3);
    }

    #[test]
    fn test_document_get_line() {
        let doc = Document::new(
            test_uri(),
            "line1\nline2\nline3".to_string(),
            1,
            "vue".to_string(),
        );

        assert_eq!(doc.line(0), Some("line1\n".to_string()));
        assert_eq!(doc.line(1), Some("line2\n".to_string()));
        assert_eq!(doc.line(2), Some("line3".to_string()));
        assert_eq!(doc.line(3), None);
    }

    #[test]
    fn test_incremental_change() {
        let mut doc = Document::new(test_uri(), "hello world".to_string(), 1, "vue".to_string());

        // Replace "world" with "universe"
        let change = TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position {
                    line: 0,
                    character: 6,
                },
                end: Position {
                    line: 0,
                    character: 11,
                },
            }),
            range_length: None,
            text: "universe".to_string(),
        };

        doc.apply_change(&change, 2);

        assert_eq!(doc.text(), "hello universe");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_full_content_change() {
        let mut doc = Document::new(test_uri(), "hello world".to_string(), 1, "vue".to_string());

        let change = TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "completely new content".to_string(),
        };

        doc.apply_change(&change, 2);

        assert_eq!(doc.text(), "completely new content");
    }

    #[test]
    fn test_document_store() {
        let store = DocumentStore::new();

        store.open(test_uri(), "content".to_string(), 1, "vue".to_string());

        assert!(store.contains(&test_uri()));
        assert_eq!(store.len(), 1);

        {
            let doc = store.get(&test_uri()).unwrap();
            assert_eq!(doc.text(), "content");
        }

        store.close(&test_uri());
        assert!(!store.contains(&test_uri()));
        assert!(store.is_empty());
    }

    #[test]
    fn test_document_store_rename() {
        let store = DocumentStore::new();
        let old_uri = test_uri();
        let new_uri = Url::parse("file:///renamed.vue").unwrap();

        store.open(old_uri.clone(), "content".to_string(), 3, "vue".to_string());

        assert!(store.rename(&old_uri, new_uri.clone()));
        assert!(!store.contains(&old_uri));
        assert!(store.contains(&new_uri));

        let doc = store.get(&new_uri).unwrap();
        assert_eq!(doc.text(), "content");
        assert_eq!(doc.version, 3);
    }
}
