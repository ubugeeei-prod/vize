use super::{Document, DocumentStore};
use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};

fn test_uri() -> Url {
    Url::parse("file:///test.vue").unwrap()
}

fn pos(line: u32, character: u32) -> Position {
    Position { line, character }
}

fn ranged_change(
    start: Position,
    end: Position,
    text: impl Into<String>,
) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeEvent {
        range: Some(Range { start, end }),
        range_length: None,
        text: text.into(),
    }
}

fn full_change(text: impl Into<String>) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: text.into(),
    }
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
    let change = ranged_change(pos(0, 6), pos(0, 11), "universe");

    doc.apply_change(&change, 2);

    assert_eq!(doc.text(), "hello universe");
    assert_eq!(doc.version, 2);
}

#[test]
fn test_incremental_change_uses_utf16_positions() {
    let mut doc = Document::new(test_uri(), "a😀b".to_string(), 1, "vue".to_string());
    let change = ranged_change(pos(0, 3), pos(0, 4), "c");

    doc.apply_change(&change, 2);

    assert_eq!(doc.text(), "a😀c");
    assert_eq!(doc.version, 2);
}

#[test]
fn test_incremental_change_rejects_utf16_surrogate_pair_interior() {
    let mut doc = Document::new(test_uri(), "a😀b".to_string(), 1, "vue".to_string());
    let change = ranged_change(pos(0, 2), pos(0, 3), "x");

    doc.apply_change(&change, 2);

    assert_eq!(doc.text(), "a😀b");
    assert_eq!(doc.version, 2);
}

#[test]
fn test_petite_vue_detection_is_memoized_and_reset_on_change() {
    let mut doc = Document::new(
        test_uri(),
        "<div id=\"app\">{{ count }}</div>".to_string(),
        1,
        "html".to_string(),
    );
    assert!(!doc.petite_vue_detected());
    assert!(!doc.petite_vue_detected());

    let change = full_change(
        "<script src=\"https://unpkg.com/petite-vue\" defer init></script>\n\
         <div v-scope>{{ count }}</div>",
    );
    doc.apply_change(&change, 2);

    assert!(doc.petite_vue_detected());
}

#[test]
fn test_full_content_change() {
    let mut doc = Document::new(test_uri(), "hello world".to_string(), 1, "vue".to_string());
    let change = full_change("completely new content");

    doc.apply_change(&change, 2);

    assert_eq!(doc.text(), "completely new content");
}

#[test]
fn test_reversed_incremental_change_is_ignored() {
    let mut doc = Document::new(test_uri(), "hello world".to_string(), 1, "vue".to_string());
    let change = ranged_change(pos(0, 11), pos(0, 6), "universe");

    doc.apply_change(&change, 2);

    assert_eq!(doc.text(), "hello world");
    assert_eq!(doc.version, 2);
}

#[test]
fn test_out_of_bounds_incremental_change_is_ignored() {
    let mut doc = Document::new(test_uri(), "hello world".to_string(), 1, "vue".to_string());
    let change = ranged_change(pos(42, 0), pos(42, 5), "ignored");

    doc.apply_change(&change, 2);

    assert_eq!(doc.text(), "hello world");
    assert_eq!(doc.version, 2);
}

#[test]
fn test_multiline_incremental_change_replaces_cross_line_range() {
    let mut doc = Document::new(
        test_uri(),
        "first\nsecond\nthird".to_string(),
        1,
        "vue".to_string(),
    );

    let change = ranged_change(pos(0, 2), pos(2, 2), "X\nY");
    doc.apply_change(&change, 2);

    assert_eq!(doc.text(), "fiX\nYird");
    assert_eq!(doc.version, 2);
}

#[test]
fn test_multiple_document_store_changes_apply_in_order_with_final_version() {
    let store = DocumentStore::new();
    let uri = test_uri();
    store.open(
        uri.clone(),
        "alpha beta gamma".to_string(),
        1,
        "vue".to_string(),
    );

    store.apply_changes(
        &uri,
        vec![
            ranged_change(pos(0, 6), pos(0, 10), "BETA"),
            ranged_change(pos(0, 16), pos(0, 16), "!"),
        ],
        2,
    );

    let doc = store.get(&uri).unwrap();
    assert_eq!(doc.text(), "alpha BETA gamma!");
    assert_eq!(doc.version, 2);
}

#[test]
fn test_document_store_ignores_changes_for_closed_document() {
    let store = DocumentStore::new();
    let uri = test_uri();
    store.open(uri.clone(), "content".to_string(), 1, "vue".to_string());
    store.close(&uri);

    store.apply_changes(&uri, vec![full_change("resurrected")], 2);

    assert!(!store.contains(&uri));
    assert!(store.is_empty());
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

#[test]
fn test_document_store_rename_same_uri_is_noop_for_open_document() {
    let store = DocumentStore::new();
    let uri = test_uri();

    store.open(uri.clone(), "content".to_string(), 3, "vue".to_string());

    assert!(store.rename(&uri, uri.clone()));
    assert!(store.contains(&uri));

    let doc = store.get(&uri).unwrap();
    assert_eq!(doc.text(), "content");
    assert_eq!(doc.version, 3);
}

#[test]
fn test_document_store_rename_missing_source_returns_false() {
    let store = DocumentStore::new();
    let old_uri = test_uri();
    let new_uri = Url::parse("file:///renamed.vue").unwrap();

    assert!(!store.rename(&old_uri, new_uri.clone()));
    assert!(!store.contains(&old_uri));
    assert!(!store.contains(&new_uri));
}

#[test]
fn test_document_store_rename_does_not_overwrite_open_target() {
    let store = DocumentStore::new();
    let old_uri = test_uri();
    let new_uri = Url::parse("file:///renamed.vue").unwrap();

    store.open(old_uri.clone(), "source".to_string(), 3, "vue".to_string());
    store.open(new_uri.clone(), "target".to_string(), 7, "vue".to_string());

    assert!(!store.rename(&old_uri, new_uri.clone()));

    let source = store.get(&old_uri).unwrap();
    assert_eq!(source.text(), "source");
    assert_eq!(source.version, 3);
    drop(source);

    let target = store.get(&new_uri).unwrap();
    assert_eq!(target.text(), "target");
    assert_eq!(target.version, 7);
}
