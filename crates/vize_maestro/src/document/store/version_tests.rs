use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use super::DocumentStore;

fn full_change(text: &str) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: text.into(),
    }
}

#[test]
fn document_store_rejects_out_of_order_versions() {
    let store = DocumentStore::new();
    let uri = Url::parse("file:///version-order.vue").unwrap();
    store.open(uri.clone(), "initial".into(), 1, "vue".into());

    assert!(store.apply_changes(&uri, vec![full_change("newest")], 3));
    assert!(!store.apply_changes(&uri, vec![full_change("stale")], 2));
    assert!(!store.apply_changes(&uri, vec![full_change("duplicate")], 3));

    let doc = store.get(&uri).unwrap();
    assert_eq!(doc.text(), "newest");
    assert_eq!(doc.version, 3);
}
