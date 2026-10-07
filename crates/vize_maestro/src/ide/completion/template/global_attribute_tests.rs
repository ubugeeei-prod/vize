#![expect(
    clippy::disallowed_methods,
    reason = "whole LSP test payloads use std strings"
)]

use serde_json::{Value, json};
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Url};

use crate::ide::{CompletionService, HoverService, IdeContext};
use crate::server::ServerState;

const COMP: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/component-global-attributes-8015/Comp.vue.txt"
);
const CHILD: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/component-global-attributes-8015/Child.vue.txt"
);
const COMMON: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/component-global-attributes-8015/common.expected.json"
);

#[test]
fn original_component_class_keeps_the_whole_existing_common_item() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Child.vue"), CHILD).unwrap();
    let uri = Url::from_file_path(dir.path().join("Comp.vue")).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), COMP.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, COMP);
    let offset = COMP.find("class=").unwrap() + 2;
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let mut item = common().as_array().unwrap().get(1).unwrap().clone();
    item.as_object_mut().unwrap().remove("insertText");
    item.as_object_mut().unwrap().insert(
        "textEdit".to_string(),
        json!({"range":{"start":{"line":7,"character":9},"end":{"line":7,"character":11}},"newText":"class=\"$1\""}),
    );
    assert_eq!(
        serde_json::to_value(CompletionService::complete(&ctx)).unwrap(),
        json!([item])
    );
}

#[test]
fn component_globals_are_common_only_and_declared_props_keep_complete_authority() {
    let uri = Url::parse("file:///Global.vue").unwrap();
    let state = ServerState::new();
    let source = "<template><Child  /></template>";
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);
    let ctx = IdeContext::new(&state, &uri, source.find(" />").unwrap()).unwrap();
    let actual = super::native::native_element_attribute_completions(&ctx, &[]);
    assert_eq!(serde_json::to_value(&actual).unwrap(), common());
    let declared = CompletionItem {
        label: "class".to_string(),
        kind: Some(CompletionItemKind::PROPERTY),
        detail: Some("prop: number (required)".to_string()),
        documentation: Some(tower_lsp::lsp_types::Documentation::String(
            "whole declared documentation".to_string(),
        )),
        data: Some(json!({"declared":true})),
        ..Default::default()
    };
    let mut expected = actual;
    expected.remove(1);
    let mut globals =
        super::native::native_element_attribute_completions(&ctx, std::slice::from_ref(&declared));
    assert_eq!(globals, expected);
    super::event_union::append_component_surface(&mut globals, vec![declared.clone()]);
    expected.push(declared);
    assert_eq!(globals, expected);
    let event = CompletionItem {
        label: "class".to_string(),
        kind: Some(CompletionItemKind::EVENT),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(super::native::native_element_attribute_completions(
            &ctx,
            &[event]
        ))
        .unwrap(),
        common()
    );
}

#[test]
fn native_button_retains_its_complete_old_attributes_and_component_values_remain_empty() {
    let old = include_str!(
        "../../../../../../tests/_fixtures/differential/lsp/component-native-events/native/button-attributes.expected.json"
    );
    for (source, marker, expected) in [
        (
            "<template><button  /></template>",
            " />",
            serde_json::from_str::<Value>(old).unwrap(),
        ),
        (
            "<template><Child class=\"wide\" /></template>",
            "de\"",
            json!([]),
        ),
        ("<template><Child @cli /></template>", " />", json!([])),
        ("<template><Child #foo /></template>", " />", json!([])),
    ] {
        let uri = Url::parse("file:///Control.vue").unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);
        let ctx = IdeContext::new(&state, &uri, source.find(marker).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(super::native::native_element_attribute_completions(
                &ctx,
                &[]
            ))
            .unwrap(),
            expected
        );
    }
}

#[test]
fn original_generic_prop_metadata_retains_complete_declared_hover() {
    let list = include_str!(
        "../../../../../../tests/_fixtures/differential/lsp/component-global-attributes-8015/List.vue.txt"
    );
    let app = include_str!(
        "../../../../../../tests/_fixtures/differential/lsp/component-global-attributes-8015/App.vue.txt"
    );
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("List.vue"), list).unwrap();
    let uri = Url::from_file_path(dir.path().join("App.vue")).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), app.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, app);
    for (name, ty, optional) in [("items", "T[]", false), ("title", "string", true)] {
        let ctx = IdeContext::new(&state, &uri, app.rfind(name).unwrap()).unwrap();
        let requirement = if optional { "Optional" } else { "Required" };
        let signature = vize_l0::cstr!("{name}{}: {ty}", if optional { "?" } else { "" });
        let expected = vize_l0::cstr!(
            "**{name}**\n\n_Component prop_\n\n```typescript\n{signature}\n```\n\n**Requirement**\n\n{requirement}\n\n**Example**\n\n```vue\n<List {name}=\"...\" />\n<List :{name}=\"value\" />\n```\n\n**Docs**\n\n[Vue Component Props](https://vuejs.org/guide/components/props.html)"
        );
        assert_eq!(
            serde_json::to_value(HoverService::hover(&ctx)).unwrap(),
            json!({"contents":{"kind":"markdown","value":expected}})
        );
    }
}

fn common() -> Value {
    serde_json::from_str(COMMON).unwrap()
}
