use super::*;

fn source() -> TrackedDocumentTexts {
    let mut source = TrackedDocumentTexts::default();
    for uri in ["file:///a.ts", "file:///b.ts", "file:///c.ts"] {
        source.insert(uri.into(), uri.into());
    }
    source
}

#[test]
fn one_edit_selects_only_its_exact_identity() {
    let mut source = source();
    assert!(source.changes_since(None).is_none());
    let checkpoint = source.acknowledge();
    source.insert("file:///b.ts".into(), "changed".into());
    assert_eq!(
        source.changes_since(Some(checkpoint)).unwrap(),
        &FxHashSet::from_iter(["file:///b.ts".into()])
    );
    assert_eq!(source.get("file:///a.ts").unwrap(), "file:///a.ts");
}

#[test]
fn equal_text_does_not_dirty_the_owner() {
    let mut source = source();
    let checkpoint = source.acknowledge();
    source.insert("file:///b.ts".into(), "file:///b.ts".into());
    source.remove("file:///absent.ts");
    assert!(source.changes_since(Some(checkpoint)).unwrap().is_empty());
    assert!(!source.is_dirty());
    assert_eq!(source.acknowledge(), checkpoint);
}

#[test]
fn renamed_and_encoded_uris_remain_distinct() {
    let mut source = source();
    let checkpoint = source.acknowledge();
    let text = source.remove("file:///a.ts").unwrap();
    source.insert("file:///%61.ts".into(), text);
    assert_eq!(
        source.changes_since(Some(checkpoint)).unwrap(),
        &FxHashSet::from_iter(["file:///a.ts".into(), "file:///%61.ts".into()])
    );
}

#[test]
fn close_reopen_and_revert_keep_one_identity_until_delivery() {
    let mut source = source();
    let checkpoint = source.acknowledge();
    source.remove("file:///b.ts");
    source.insert("file:///b.ts".into(), "temporary".into());
    source.insert("file:///b.ts".into(), "file:///b.ts".into());
    assert_eq!(source.changes_since(Some(checkpoint)).unwrap().len(), 1);
    assert_eq!(source.get("file:///b.ts").unwrap(), "file:///b.ts");
}

#[test]
fn clear_retains_every_removal() {
    let mut source = source();
    let checkpoint = source.acknowledge();
    source.clear();
    assert_eq!(source.changes_since(Some(checkpoint)).unwrap().len(), 3);
    assert!(source.is_empty());
}

#[test]
fn a_foreign_owner_or_replaced_source_cannot_borrow_a_checkpoint() {
    let mut original = source();
    let checkpoint = original.acknowledge();
    assert!(source().changes_since(Some(checkpoint)).is_none());
    assert!(original.clone().changes_since(Some(checkpoint)).is_none());
    let replacement = TrackedDocumentTexts::from(FxHashMap::<String, String>::from_iter([(
        "file:///a.ts".into(),
        "replacement".into(),
    )]));
    assert!(replacement.changes_since(Some(checkpoint)).is_none());
}

#[test]
fn a_retired_receiver_checkpoint_requires_a_complete_reconciliation() {
    let mut source = source();
    let retired = source.acknowledge();
    source.insert("file:///b.ts".into(), "changed".into());
    let current = source.acknowledge();
    assert!(source.changes_since(Some(retired)).is_none());
    assert!(source.changes_since(Some(current)).unwrap().is_empty());
    assert!(source.changes_since(None).is_none());
}

#[test]
fn failed_delivery_does_not_consume_the_selected_changes() {
    let mut source = source();
    let checkpoint = source.acknowledge();
    source.insert("file:///b.ts".into(), "changed".into());
    source.remove("file:///a.ts");
    let before_failure = source.changes_since(Some(checkpoint)).unwrap().clone();
    assert_eq!(
        source.changes_since(Some(checkpoint)).unwrap(),
        &before_failure
    );
    source.insert("file:///c.ts".into(), "later".into());
    assert_eq!(source.changes_since(Some(checkpoint)).unwrap().len(), 3);
}

#[test]
fn unbounded_uri_churn_discards_the_journal_for_a_full_sync() {
    let mut source = source();
    let checkpoint = source.acknowledge();
    for index in 0..8 {
        let uri = vize_l0::cstr!("file:///temporary-{index}.ts");
        source.insert(uri.clone(), "temporary".into());
        source.remove(uri.as_str());
    }
    assert!(source.changes_since(Some(checkpoint)).is_none());
    assert_eq!(source.len(), 3);
    let checkpoint = source.acknowledge();
    assert!(source.changes_since(Some(checkpoint)).unwrap().is_empty());
}

#[test]
fn replay_restores_every_buffer_without_overwriting_a_newer_write() {
    let mut source = source();
    let checkpoint = source.acknowledge();
    let retained = std::mem::take(&mut source);
    source.insert("file:///b.ts".into(), "newer".into());
    for (uri, text) in retained {
        source.insert_if_absent(uri, text);
    }
    assert_eq!(source.len(), 3);
    assert_eq!(source.get("file:///b.ts").unwrap(), "newer");
    assert!(source.changes_since(Some(checkpoint)).is_none());
}

#[test]
fn serialization_and_equality_keep_the_existing_plain_document_map() {
    let source = source();
    let expected = FxHashMap::from_iter([
        (String::from("file:///a.ts"), String::from("file:///a.ts")),
        (String::from("file:///b.ts"), String::from("file:///b.ts")),
        (String::from("file:///c.ts"), String::from("file:///c.ts")),
    ]);
    assert_eq!(source, expected);
    assert_eq!(
        serde_json::to_value(&source).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
}
