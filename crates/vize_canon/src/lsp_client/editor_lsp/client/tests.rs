use super::{CorsaProjectClient, Value};

#[test]
fn hierarchy_items_preserve_unmapped_foreign_documents() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let mut client = CorsaProjectClient::empty_for_test(root.join("workspace"));
    client.materialized_project_session = true;
    let foreign = crate::file_uri::path_to_file_uri(&root.join("foreign/module.ts"));
    let item = serde_json::json!({"uri":foreign,"data":{"uri":foreign}});
    assert_eq!(
        client.native_call_hierarchy_item(item.clone()).unwrap(),
        (foreign, item)
    );
    assert!(client.session_document_uris.is_empty());
    assert!(client.external_document_uris.is_empty());
}

#[test]
fn hierarchy_items_keep_native_identity_and_opaque_data() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let mut client = CorsaProjectClient::empty_for_test(root.clone());
    client.materialized_project_session = true;
    let uri = crate::file_uri::path_to_file_uri(&root.join("src/App.vue.ts"));
    let native = client.editor_document_uri(&uri);
    assert_ne!(native, uri);
    let data = serde_json::json!({"uri":uri,"arguments":[uri]});
    let (request, item) = client
        .native_call_hierarchy_item(serde_json::json!({"uri":uri,"data":data}))
        .unwrap();
    assert_eq!(request, uri);
    assert_eq!(item, serde_json::json!({"uri":native,"data":data}));
    for invalid in [
        Value::Null,
        serde_json::json!([]),
        serde_json::json!({"uri":7}),
        serde_json::json!({"data":data}),
    ] {
        assert!(client.native_call_hierarchy_item(invalid).is_err());
    }
}
