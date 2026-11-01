//! Owned-buffer controls only; native synchronization is qualified separately.

use super::CorsaProjectClient;
use crate::file_uri::path_to_file_uri;
use vize_l0::String;

const URI: &str =
    "file:///workspace/independent-project/long-owned-component-path/component.vue.ts";

fn source(revision: &str) -> String {
    String::from(
        vize_l0::cstr!("// 雪😀 {revision}\r\nexport const value = '{revision}';\r\n").repeat(128),
    )
}

fn pointers(client: &CorsaProjectClient, uri: &str) -> (*const u8, *const u8) {
    let (key, text) = client.document_texts.get_key_value(uri).unwrap();
    (key.as_str().as_ptr(), text.as_str().as_ptr())
}

#[test]
fn equal_unicode_crlf_text_keeps_the_original_owned_key_and_value() {
    let root = tempfile::tempdir().unwrap();
    let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
    let text = source("first");
    let inserted = client.retain_document_text(URI, &text);
    assert!(inserted.changed());
    assert!(!inserted.was_present());
    assert_eq!(inserted.previous_text(), None);
    let original = pointers(&client, URI);

    for _ in 0..32 {
        let unchanged = client.retain_document_text(URI, &text);
        assert!(!unchanged.changed());
        assert!(unchanged.was_present());
        assert_eq!(unchanged.previous_text(), None);
        let current = pointers(&client, URI);
        assert_eq!(current.0, original.0);
        assert_eq!(current.1, original.1);
        assert_eq!(client.document_texts.get(URI), Some(&text));
    }
    assert!(client.session.is_none());
    assert!(client.editor_lsp.is_none());
}

#[test]
fn changed_values_keep_the_original_allocation_alive_through_the_callers_scope() {
    let root = tempfile::tempdir().unwrap();
    let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
    let first = source("first");
    let second = source("second");
    let third = source("third");
    let _ = client.retain_document_text(URI, &first);
    let original = pointers(&client, URI);
    let changed = client.retain_document_text(URI, &second);
    assert!(changed.changed());
    assert!(changed.was_present());
    assert_eq!(changed.previous_text(), Some(first.as_str()));
    assert_eq!(changed.previous_text().unwrap().as_ptr(), original.1);
    assert_eq!(pointers(&client, URI).0, original.0);
    assert_ne!(pointers(&client, URI).1, original.1);
    let second_update = client.retain_document_text(URI, &third);
    assert_eq!(second_update.previous_text(), Some(second.as_str()));
    assert_eq!(client.document_texts.get(URI), Some(&third));
    drop(client);
    assert_eq!(changed.previous_text(), Some(first.as_str()));
    assert_eq!(changed.previous_text().unwrap().as_ptr(), original.1);
    assert_eq!(second_update.previous_text(), Some(second.as_str()));
}

#[test]
fn duplicate_batch_entries_keep_sequential_last_writer_and_dirty_state() {
    let root = tempfile::tempdir().unwrap();
    let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
    let first = source("first");
    let second = source("second");
    client
        .did_open_batch_fast(&[(URI, &first), (URI, &first), (URI, &second), (URI, &second)])
        .unwrap();
    assert_eq!(client.document_texts.len(), 1);
    assert_eq!(client.document_texts.get(URI), Some(&second));
    assert!(client.editor_lsp_documents_dirty);
    let original = pointers(&client, URI);

    client.editor_lsp_documents_dirty = false;
    client.diagnostics.insert(URI.into(), Vec::new());
    client.overlay_versions.insert(URI.into(), 7);
    client
        .did_open_batch_fast(&[(URI, &second), (URI, &second)])
        .unwrap();
    assert_eq!(pointers(&client, URI), original);
    assert!(!client.diagnostics.contains_key(URI));
    assert_eq!(client.overlay_versions.get(URI), Some(&7));
    assert!(!client.editor_lsp_documents_dirty);

    client.editor_lsp_documents_dirty = true;
    client.diagnostics.insert(URI.into(), Vec::new());
    client.did_open_batch_fast(&[(URI, &second)]).unwrap();
    assert!(client.editor_lsp_documents_dirty);
    assert!(!client.diagnostics.contains_key(URI));
    assert_eq!(pointers(&client, URI), original);
    assert!(client.session.is_none());
    assert!(client.editor_lsp.is_none());
}

#[test]
fn the_single_writer_keeps_equal_storage_without_skipping_diagnostic_clearing() {
    let root = tempfile::tempdir().unwrap();
    let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
    let text = source("single");
    client.did_open(URI, &text).unwrap();
    let original = pointers(&client, URI);
    client.editor_lsp_documents_dirty = false;
    client.diagnostics.insert(URI.into(), Vec::new());
    client.did_change(URI, &text).unwrap();
    assert_eq!(pointers(&client, URI), original);
    assert!(!client.editor_lsp_documents_dirty);
    assert!(!client.diagnostics.contains_key(URI));

    client.editor_lsp_documents_dirty = true;
    client.did_change(URI, &text).unwrap();
    assert_eq!(pointers(&client, URI), original);
    assert!(client.editor_lsp_documents_dirty);
    assert!(client.session.is_none());
    assert!(client.editor_lsp.is_none());
}

#[test]
fn equal_logical_batches_still_retain_the_complete_revision_and_uri_mappings() {
    let root = tempfile::tempdir().unwrap();
    let uri = path_to_file_uri(&root.path().join("long-owned-logical-component.vue.ts"));
    let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
    client.logical_workspace_overlays = true;
    let text = source("logical");
    let _ = client.retain_document_text(&uri, &text);
    let original = pointers(&client, &uri);
    client.editor_lsp_documents_dirty = false;
    client.diagnostics.insert(uri.clone(), Vec::new());
    client
        .did_open_batch_fast(&[(&uri, &text), (&uri, &text)])
        .unwrap();
    assert_eq!(pointers(&client, &uri), original);
    assert_eq!(client.document_texts.get(&uri), Some(&text));
    assert!(client.editor_lsp_documents_dirty);
    assert!(client.diagnostics.is_empty());
    assert_eq!(client.session_document_uris.get(&uri), Some(&uri));
    assert_eq!(client.external_document_uris.get(&uri), Some(&uri));
    assert!(client.session.is_none());
    assert!(client.editor_lsp.is_none());
}

#[test]
fn equal_retained_materialized_bytes_still_repair_a_stale_disk_member() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    std::fs::write(root.join("__vize_vue_modules.d.ts"), "export {};\n").unwrap();
    let file = root.join("Component.vue.ts");
    std::fs::write(&file, "stale disk bytes\r\n").unwrap();
    let uri = path_to_file_uri(&file);
    let text = source("materialized");
    let mut client = CorsaProjectClient::empty_for_test(root);
    let _ = client.retain_document_text(&uri, &text);
    let original = pointers(&client, &uri);
    client.editor_lsp_documents_dirty = false;
    client.diagnostics.insert(uri.clone(), Vec::new());
    client
        .synchronize_materialized_project_documents(&[(&uri, &text)])
        .unwrap();
    assert_eq!(std::fs::read(&file).unwrap(), text.as_bytes());
    assert_eq!(pointers(&client, &uri), original);
    assert_eq!(client.session_document_uris.get(&uri), Some(&uri));
    assert_eq!(client.external_document_uris.get(&uri), Some(&uri));
    assert!(!client.diagnostics.contains_key(&uri));
    assert!(!client.editor_lsp_documents_dirty);
    assert!(client.session.is_none());
    assert!(client.editor_lsp.is_none());
}

#[test]
fn independent_clients_and_removed_membership_do_not_share_retained_storage() {
    let first_root = tempfile::tempdir().unwrap();
    let second_root = tempfile::tempdir().unwrap();
    let mut first = CorsaProjectClient::empty_for_test(first_root.path().to_path_buf());
    let mut second = CorsaProjectClient::empty_for_test(second_root.path().to_path_buf());
    let text = source("independent");
    let _ = first.retain_document_text(URI, &text);
    let _ = second.retain_document_text(URI, &text);
    assert_ne!(pointers(&first, URI).0, pointers(&second, URI).0);
    assert_ne!(pointers(&first, URI).1, pointers(&second, URI).1);
    let original_second = pointers(&second, URI);
    let changed = source("changed");
    let previous = first.retain_document_text(URI, &changed);
    assert_eq!(previous.previous_text(), Some(text.as_str()));
    assert_eq!(second.document_texts.get(URI), Some(&text));
    assert_eq!(pointers(&second, URI), original_second);

    let removed = first.document_texts.remove(URI).unwrap();
    assert_eq!(removed, changed);
    let reopened = first.retain_document_text(URI, &text);
    assert!(reopened.changed());
    assert!(!reopened.was_present());
    assert_eq!(reopened.previous_text(), None);
    assert_eq!(first.document_texts.get(URI), Some(&text));
    assert_eq!(pointers(&second, URI), original_second);
    assert!(first.session.is_none());
    assert!(second.session.is_none());
}
