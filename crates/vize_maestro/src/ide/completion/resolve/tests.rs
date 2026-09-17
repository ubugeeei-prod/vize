use super::*;

const SOURCE: &str = "<script setup>const value = 1</script>";

#[test]
fn list_defaults_supply_resolve_data_and_snippet_format() {
    let items = completion_items(json!({
        "items": [{"label": "greet", "insertText": "greet(${1:name})"}],
        "itemDefaults": {"data": {"id": 3}, "insertTextFormat": 2, "editRange": {}}
    }));
    assert_eq!(
        items,
        vec![json!({"label": "greet", "insertText": "greet(${1:name})",
        "data": {"id": 3}, "insertTextFormat": 2})]
    );
    let item = CompletionService::convert_lsp_completion(
        serde_json::from_value(items[0].clone()).unwrap(),
    );
    assert_eq!(
        item.insert_text_format,
        Some(tower_lsp::lsp_types::InsertTextFormat::SNIPPET)
    );
}

#[test]
fn item_fields_override_defaults_including_explicit_null_data() {
    let items = json!([
        {"label": "one", "data": {"id": 1}, "insertTextFormat": 1},
        {"label": "two", "data": null, "insertTextFormat": 2}
    ]);
    assert_eq!(
        completion_items(json!({"items": items,
            "itemDefaults": {"data": {"id": 3}, "insertTextFormat": 1}
        })),
        items.as_array().unwrap().clone()
    );
    assert_eq!(
        completion_items(items.clone()),
        items.as_array().unwrap().clone()
    );
}

#[test]
fn malformed_lists_cannot_create_candidates() {
    for value in [Value::Null, json!({}), json!({"items": false})] {
        assert_eq!(completion_items(value), Vec::<Value>::new());
    }
}

fn fixture() -> (ServerState, Url, u64, CompletionItem) {
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&json!({ "editor": true, "typecheck": true })));
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    state
        .documents
        .open(uri.clone(), SOURCE.into(), 1, "vue".into());
    let revision = state.documents.get(&uri).unwrap().revision();
    let item = CompletionItem {
        label: "value".into(),
        data: Some(json!({ RESOLVE_DATA: {
            "uri": uri,
            "revision": revision,
            "requestUri": "file:///workspace/App.vue.ts",
            "item": { "label": "value", "data": { "name": "value" } },
        }})),
        ..Default::default()
    };
    (state, uri, revision, item)
}

#[test]
fn disabled_completion_or_typecheck_preserves_the_item_without_starting_a_backend() {
    for options in [
        json!({ "completion": false }),
        json!({ "typecheck": false }),
    ] {
        let (state, uri, revision, item) = fixture();
        assert!(can_resolve(&state, &uri, revision));
        state.apply_lsp_initialization_options(Some(&options));
        assert!(!can_resolve(&state, &uri, revision));
        assert_eq!(
            futures::executor::block_on(CompletionService::resolve(&state, item.clone())),
            item
        );
    }
}

#[test]
fn identical_reopened_content_and_client_version_cannot_reuse_a_candidate() {
    let (state, uri, revision, item) = fixture();
    state.documents.close(&uri);
    state
        .documents
        .open(uri.clone(), SOURCE.into(), 1, "vue".into());
    assert!(!can_resolve(&state, &uri, revision));
    assert_eq!(
        futures::executor::block_on(CompletionService::resolve(&state, item.clone())),
        item
    );
}

#[test]
fn mismatched_labels_and_incomplete_payloads_are_not_forwarded() {
    let (state, _, _, mut item) = fixture();
    item.label = "different".into();
    assert_eq!(
        futures::executor::block_on(CompletionService::resolve(&state, item.clone())),
        item
    );
    for data in [
        Value::Null,
        json!({}),
        json!({ RESOLVE_DATA: { "uri": false } }),
    ] {
        item.data = Some(data);
        assert_eq!(
            futures::executor::block_on(CompletionService::resolve(&state, item.clone())),
            item
        );
    }
}
