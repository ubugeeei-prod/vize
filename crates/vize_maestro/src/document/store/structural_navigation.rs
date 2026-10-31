//! A cache belongs to one exact resident source owner, never an editor version.

use std::sync::Arc;
use tower_lsp::lsp_types::Url;

use super::DocumentStore;
use crate::ide::references::StructuralSnapshotOwner;

impl DocumentStore {
    #[cfg(test)]
    pub(crate) fn isolate_structural_snapshot(
        &self,
        uri: &Url,
    ) -> Option<Arc<crate::ide::references::StructuralSnapshotCell>> {
        let mut document = self.documents.get_mut(uri)?;
        let cell = Arc::new(crate::ide::references::StructuralSnapshotCell::isolated());
        document.structural_snapshot = std::sync::OnceLock::from(Arc::clone(&cell));
        Some(cell)
    }
    pub(crate) fn structural_snapshot_owner(
        &self,
        uri: &Url,
        source: &Arc<String>,
    ) -> Option<StructuralSnapshotOwner> {
        let document = self.documents.get(uri)?;
        let current = document.text_snapshot();
        if !Arc::ptr_eq(&current, source) {
            return None;
        }
        let cell = Arc::clone(document.structural_snapshot.get_or_init(Arc::default));
        Some(StructuralSnapshotOwner::new(
            cell,
            document.revision(),
            uri.clone(),
            current,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::TextDocumentContentChangeEvent;

    const SOURCE: &str =
        "<script setup>const alpha = 1;</script>\r\n<template>雪😀 {{ alpha }}</template>";

    fn open(store: &DocumentStore, uri: &Url) -> (Arc<String>, StructuralSnapshotOwner) {
        store.open(uri.clone(), SOURCE.into(), 1, "vue".into());
        let source = store.text_snapshot(uri).unwrap();
        let owner = store.structural_snapshot_owner(uri, &source).unwrap();
        (source, owner)
    }

    #[test]
    fn only_the_exact_resident_source_owner_is_admitted() {
        let store = DocumentStore::new();
        let uri = Url::parse("file:///workspace/Owner.vue").unwrap();
        let (source, first) = open(&store, &uri);
        let second = store.structural_snapshot_owner(&uri, &source).unwrap();
        assert!(first.same_generation(&second));
        assert!(
            store
                .structural_snapshot_owner(&uri, &Arc::new(SOURCE.into()))
                .is_none()
        );
        let other = Url::parse("file:///workspace/Other.vue").unwrap();
        store.open(other.clone(), SOURCE.into(), 1, "vue".into());
        assert!(store.structural_snapshot_owner(&other, &source).is_none());
    }

    #[test]
    fn edit_get_mut_rename_close_and_reopen_detach_old_generations() {
        let store = DocumentStore::new();
        let uri = Url::parse("file:///workspace/Owner.vue").unwrap();
        let (source, original) = open(&store, &uri);
        let changed = SOURCE.replace("1;", "2;");
        assert!(store.apply_changes(
            &uri,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: changed
            }],
            2
        ));
        assert!(store.structural_snapshot_owner(&uri, &source).is_none());
        let current = store.text_snapshot(&uri).unwrap();
        let after_edit = store.structural_snapshot_owner(&uri, &current).unwrap();
        assert!(!original.same_generation(&after_edit));
        {
            let mut document = store.get_mut(&uri).unwrap();
            document.language_id = "vue".into();
        }
        assert!(store.structural_snapshot_owner(&uri, &current).is_none());
        let current = store.text_snapshot(&uri).unwrap();
        let after_mut = store.structural_snapshot_owner(&uri, &current).unwrap();
        assert!(!after_edit.same_generation(&after_mut));
        let renamed = Url::parse("file:///workspace/Renamed.vue").unwrap();
        assert!(store.rename(&uri, renamed.clone()));
        assert!(store.structural_snapshot_owner(&uri, &current).is_none());
        let after_rename = store.structural_snapshot_owner(&renamed, &current).unwrap();
        assert!(!after_mut.same_generation(&after_rename));
        store.close(&renamed);
        assert!(
            store
                .structural_snapshot_owner(&renamed, &current)
                .is_none()
        );
        let (reopened, after_reopen) = open(&store, &renamed);
        assert!(!after_rename.same_generation(&after_reopen));
        assert!(
            store
                .structural_snapshot_owner(&renamed, &current)
                .is_none()
        );
        assert!(
            store
                .structural_snapshot_owner(&renamed, &reopened)
                .is_some()
        );
    }

    #[test]
    fn a_retired_owner_cannot_rejoin_a_concurrently_reopened_document() {
        let store = DocumentStore::new();
        let uri = Url::parse("file:///workspace/Concurrent.vue").unwrap();
        let (source, original) = open(&store, &uri);
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let worker_store = store.clone();
        let worker_uri = uri.clone();
        let worker_barrier = Arc::clone(&barrier);
        let worker = std::thread::spawn(move || {
            let old = worker_store
                .structural_snapshot_owner(&worker_uri, &source)
                .unwrap();
            worker_barrier.wait();
            worker_barrier.wait();
            old
        });
        barrier.wait();
        store.close(&uri);
        let (_, reopened) = open(&store, &uri);
        barrier.wait();
        let retired = worker.join().unwrap();
        assert!(retired.same_generation(&original));
        assert!(!retired.same_generation(&reopened));
    }
}
