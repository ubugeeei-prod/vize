use std::sync::Arc;

use super::DocumentStore;

impl DocumentStore {
    pub(crate) fn source_mutation_for_test(&self) -> impl Drop + '_ {
        self.source_set_mutation()
    }
}
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

fn full_change(text: impl Into<String>) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: text.into(),
    }
}

#[test]
fn vue_texts_snapshot_holds_no_shard_lock() {
    let store = DocumentStore::new();
    let vue_uri = Url::parse("file:///test.vue").unwrap();
    let ts_uri = Url::parse("file:///test.ts").unwrap();
    store.open(
        vue_uri.clone(),
        "<template>before</template>".to_string(),
        1,
        "vue".to_string(),
    );
    store.open(ts_uri, "export {}".to_string(), 1, "typescript".to_string());

    let snapshots = store.vue_texts();
    store.apply_changes(&vue_uri, vec![full_change("<template>after</template>")], 2);

    assert_eq!(
        snapshots
            .iter()
            .map(|(uri, source)| (uri.clone(), source.as_str()))
            .collect::<Vec<_>>(),
        vec![(vue_uri.clone(), "<template>before</template>")]
    );
    assert_eq!(
        store.text(&vue_uri).as_deref(),
        Some("<template>after</template>")
    );
}

#[test]
fn vue_texts_reuses_one_batch_until_vue_sources_change() {
    let store = DocumentStore::new();
    let vue_uri = Url::parse("file:///workspace/App.vue").unwrap();
    let ts_uri = Url::parse("file:///workspace/index.ts").unwrap();
    store.open(vue_uri.clone(), "before".into(), 1, "vue".into());
    store.open(
        ts_uri.clone(),
        "export const before = 1".into(),
        1,
        "typescript".into(),
    );

    let seeded = store.vue_texts();
    assert!(Arc::ptr_eq(&seeded, &store.vue_texts()));

    assert!(store.apply_changes(&ts_uri, vec![full_change("export const after = 2")], 2));
    assert!(Arc::ptr_eq(&seeded, &store.vue_texts()));

    assert!(store.apply_changes(&vue_uri, vec![full_change("after")], 2));
    let edited = store.vue_texts();
    assert!(!Arc::ptr_eq(&seeded, &edited));
    assert_eq!(seeded[0].1.as_str(), "before");
    assert_eq!(edited[0].1.as_str(), "after");

    let renamed_uri = Url::parse("file:///workspace/Renamed.vue").unwrap();
    assert!(store.rename(&vue_uri, renamed_uri.clone()));
    let renamed = store.vue_texts();
    assert!(!Arc::ptr_eq(&edited, &renamed));
    assert_eq!(renamed[0].0, renamed_uri);

    store.close(&renamed_uri);
    let closed = store.vue_texts();
    assert!(!Arc::ptr_eq(&renamed, &closed));
    assert!(closed.is_empty());
    assert!(Arc::ptr_eq(&closed, &store.vue_texts()));
}

#[test]
fn texts_reuses_one_batch_until_any_open_source_changes() {
    let store = DocumentStore::new();
    let vue_uri = Url::parse("file:///workspace/App.vue").unwrap();
    let ts_uri = Url::parse("file:///workspace/index.ts").unwrap();
    store.open(vue_uri.clone(), "vue-before".into(), 1, "vue".into());
    store.open(
        ts_uri.clone(),
        "export const before = 1".into(),
        1,
        "typescript".into(),
    );

    let seeded = store.texts();
    assert!(Arc::ptr_eq(&seeded, &store.texts()));
    assert_eq!(seeded.len(), 2);

    assert!(store.apply_changes(&ts_uri, vec![full_change("export const after = 2")], 2,));
    let edited = store.texts();
    assert!(!Arc::ptr_eq(&seeded, &edited));
    assert!(Arc::ptr_eq(&edited, &store.texts()));

    let old_ts = seeded.iter().find(|(uri, _)| uri == &ts_uri).unwrap();
    let new_ts = edited.iter().find(|(uri, _)| uri == &ts_uri).unwrap();
    assert_eq!(old_ts.1.as_str(), "export const before = 1");
    assert_eq!(new_ts.1.as_str(), "export const after = 2");

    store.close(&vue_uri);
    let closed = store.texts();
    assert!(!Arc::ptr_eq(&edited, &closed));
    assert_eq!(closed.len(), 1);
    assert_eq!(closed[0].0, ts_uri);
}
