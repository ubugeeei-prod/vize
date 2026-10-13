use super::{LspClient, VirtualChange, VirtualDocument};
use crate::{CorsaError, Result};
use corsa_core::fast::{CompactString, FastMap, SmallVec, compact_format};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, Uri,
    notification::{DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument},
};
use parking_lot::RwLock;
use std::sync::Arc;

/// In-memory virtual document overlay synchronized with an [`LspClient`].
///
/// The overlay is intentionally stateful: it remembers which documents are
/// currently open so it can emit valid `didOpen` / `didChange` / `didClose`
/// sequences and keep version numbers monotonic.
#[derive(Clone)]
pub struct LspOverlay {
    client: LspClient,
    documents: Arc<RwLock<FastMap<CompactString, VirtualDocument>>>,
}

impl LspOverlay {
    /// Creates an empty overlay bound to a client.
    pub fn new(client: LspClient) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(FastMap::default())),
        }
    }

    /// Returns the client used to send LSP notifications.
    pub fn client(&self) -> &LspClient {
        &self.client
    }

    /// Returns a virtual document by URI if it is currently open.
    pub fn document(&self, uri: &Uri) -> Option<VirtualDocument> {
        self.documents.read().get(uri.as_str()).cloned()
    }

    /// Returns a snapshot of all open virtual documents.
    ///
    /// The returned collection is detached from the internal lock-protected map.
    pub fn documents(&self) -> SmallVec<[VirtualDocument; 8]> {
        self.documents.read().values().cloned().collect()
    }

    /// Opens a virtual document and emits `textDocument/didOpen`.
    ///
    /// Fails if the URI is already open in the overlay.
    pub fn open(&self, document: VirtualDocument) -> Result<VirtualDocument> {
        let key = document.key();
        if self.documents.read().contains_key(key.as_str()) {
            return Err(CorsaError::Protocol(compact_format(format_args!(
                "virtual document is already open: {key}"
            ))));
        }
        self.client
            .notify::<DidOpenTextDocument>(DidOpenTextDocumentParams {
                text_document: document.text_document_item(),
            })?;
        self.documents.write().insert(key, document.clone());
        Ok(document)
    }

    /// Replaces the full contents of a virtual document.
    ///
    /// This is a convenience wrapper around [`change`](Self::change) using a
    /// single full-document replacement.
    pub fn replace(&self, uri: &Uri, text: impl Into<CompactString>) -> Result<VirtualDocument> {
        self.change(uri, [VirtualChange::replace(text)])
    }

    /// Applies one or more incremental changes to a virtual document.
    ///
    /// The overlay mutates its stored [`VirtualDocument`] first, then emits a
    /// single `textDocument/didChange` notification that preserves the exact
    /// change list passed in.
    pub fn change<I>(&self, uri: &Uri, changes: I) -> Result<VirtualDocument>
    where
        I: IntoIterator<Item = VirtualChange>,
    {
        let changes = changes
            .into_iter()
            .collect::<SmallVec<[VirtualChange; 4]>>();
        let mut documents = self.documents.write();
        let document = documents.get_mut(uri.as_str()).ok_or_else(|| {
            CorsaError::Protocol(compact_format(format_args!(
                "unknown virtual document: {}",
                uri.as_str()
            )))
        })?;
        let events = document.apply_changes(&changes)?;
        if events.is_empty() {
            return Ok(document.clone());
        }
        self.client
            .notify::<DidChangeTextDocument>(DidChangeTextDocumentParams {
                text_document: document.versioned_identifier(),
                content_changes: events.into_iter().collect(),
            })?;
        Ok(document.clone())
    }

    /// Closes a virtual document and emits `textDocument/didClose`.
    ///
    /// Returns the removed document so callers can persist or inspect its last
    /// known state.
    pub fn close(&self, uri: &Uri) -> Result<Option<VirtualDocument>> {
        let removed = self.documents.write().remove(uri.as_str());
        if let Some(document) = &removed {
            self.client
                .notify::<DidCloseTextDocument>(DidCloseTextDocumentParams {
                    text_document: document.identifier(),
                })?;
        }
        Ok(removed)
    }
}
