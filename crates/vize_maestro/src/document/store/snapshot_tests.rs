use super::DocumentStore;
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
        snapshots,
        vec![(vue_uri.clone(), "<template>before</template>".to_string())]
    );
    assert_eq!(
        store.text(&vue_uri).as_deref(),
        Some("<template>after</template>")
    );
}
