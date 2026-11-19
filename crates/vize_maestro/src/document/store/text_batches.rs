//! Generation-bound batches for project-wide editor requests.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use tower_lsp::lsp_types::Url;

use super::{DocumentStore, is_vue_uri};

impl DocumentStore {
    /// Shared sorted snapshot of every open document and exact text revision.
    /// Warm project-wide requests clone one `Arc` instead of rebuilding a URI
    /// map and cloning each source owner independently.
    pub fn texts(&self) -> Arc<[(Url, Arc<String>)]> {
        let revision = self.text_revision.load(Ordering::Acquire);
        if let Some(snapshot) = self.text_snapshots.get(revision) {
            return snapshot;
        }

        let mut snapshots = self
            .documents
            .iter()
            .map(|document| (document.key().clone(), document.value().text_snapshot()))
            .collect::<Vec<_>>();
        snapshots.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));
        let snapshots = Arc::from(snapshots);
        if revision == self.text_revision.load(Ordering::Acquire) {
            self.text_snapshots.publish(revision, snapshots)
        } else {
            snapshots
        }
    }

    /// Shared snapshot of all open Vue documents, holding no shard guard on return.
    ///
    /// Workspace-wide editor features parse this batch after the map read has
    /// finished. The complete sorted batch and each text are resident `Arc`s,
    /// so a warm search allocates neither result metadata nor open Vue buffers.
    /// Keeping a `DashMap` guard alive while parsing is not an `.await`, but it
    /// can still stall other LSP handlers that need the same shard's write lock
    /// and then make later readers wait behind that writer.
    pub fn vue_texts(&self) -> Arc<[(Url, Arc<String>)]> {
        vize_l0::micro_count!("lsp.document_store.vue_texts.calls", 1);
        let snapshots = vize_l0::micro_span!("lsp.document_store.vue_texts.clone_snapshot", {
            let revision = self.vue_revision.load(Ordering::Acquire);
            if let Some(snapshot) = self.vue_text_snapshots.get(revision) {
                snapshot
            } else {
                let mut snapshots = self
                    .documents
                    .iter()
                    .filter_map(|document| {
                        let uri = document.key();
                        is_vue_uri(uri).then(|| (uri.clone(), document.value().text_snapshot()))
                    })
                    .collect::<Vec<_>>();
                snapshots.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));
                let snapshots = Arc::from(snapshots);

                // A concurrent edit may make this batch a legitimate older view,
                // but it must not become the cache entry for the newer revision.
                // The caller can still consume the immutable batch it captured;
                // the next stable request rebuilds and publishes the current set.
                if revision == self.vue_revision.load(Ordering::Acquire) {
                    self.vue_text_snapshots.publish(revision, snapshots)
                } else {
                    snapshots
                }
            }
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
}
