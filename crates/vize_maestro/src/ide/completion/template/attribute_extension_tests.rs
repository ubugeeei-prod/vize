use serde_json::{Value, json};
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Url};

use crate::{
    ide::{CompletionService, IdeContext},
    server::ServerState,
};

const COMP: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/component-global-attributes-8015/Comp.vue.txt"
);
const CHILD: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/component-global-attributes-8015/Child.vue.txt"
);
const DATA: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/data-aria-attributes-8015/data.expected.json"
);
const ARIA: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/data-aria-attributes-8015/aria.expected.json"
);

fn item_with_edit(mut item: Value, source: &str, start: usize, end: usize, insert: &str) -> Value {
    item.as_object_mut().unwrap().remove("insertText");
    let point = |offset| {
        let (line, character) = crate::ide::offset_to_position(source, offset);
        json!({"line":line,"character":character})
    };
    item["textEdit"] = json!({"range":{"start":point(start),"end":point(end)},"newText":insert});
    item
}

#[test]
fn original_data_attribute_completion_preserves_the_whole_existing_assignment() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Child.vue"), CHILD).unwrap();
    let uri = Url::from_file_path(dir.path().join("Comp.vue")).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), COMP.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, COMP);
    let start = COMP.find("data-role").unwrap();
    let ctx = IdeContext::new(&state, &uri, start + 3).unwrap();
    let expected = item_with_edit(
        serde_json::from_str::<Value>(DATA).unwrap()["data-role"].clone(),
        COMP,
        start,
        start + 3,
        "dat",
    );
    assert_eq!(
        serde_json::to_value(CompletionService::complete(&ctx)).unwrap(),
        json!([expected])
    );
}

#[test]
fn data_templates_and_all_aria_candidates_are_identical_on_native_and_component_tags() {
    let data: Value = serde_json::from_str(DATA).unwrap();
    let aria: Value = serde_json::from_str(ARIA).unwrap();
    for tag in ["Child", "div"] {
        for prefix in ["data-", "aria-"] {
            let source = format!("<template><{tag} {prefix} /></template>");
            let uri = Url::parse("file:///Attribute.vue").unwrap();
            let state = ServerState::new();
            state
                .documents
                .open(uri.clone(), source.clone(), 1, "vue".to_string());
            let end = source.find(" />").unwrap();
            let ctx = IdeContext::new(&state, &uri, end).unwrap();
            let tag_context =
                super::tag_context::opening_tag_context_at_offset(&source, end).unwrap();
            let actual = super::native::custom_attributes::completions(&ctx, &tag_context, &[]);
            let expected = if prefix == "data-" {
                json!([data["data-*"].clone()])
            } else {
                aria.clone()
            };
            assert_eq!(serde_json::to_value(actual).unwrap(), expected);
        }
    }
}

#[test]
fn declared_data_and_aria_props_keep_their_complete_type_documentation_and_data() {
    for (prefix, label) in [("data-role", "dataRole"), ("aria-hidden", "ariaHidden")] {
        let source = format!("<template><Child :{prefix} /></template>");
        let uri = Url::parse("file:///Declared.vue").unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.clone(), 1, "vue".to_string());
        let offset = source.find(" />").unwrap();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let tag = super::tag_context::opening_tag_context_at_offset(&source, offset).unwrap();
        let prop = CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some("prop: number (required)".to_string()),
            data: Some(json!({"declared":true})),
            ..Default::default()
        };
        assert!(super::native::custom_attributes::completions(&ctx, &tag, &[prop]).is_empty());
    }
}

#[test]
fn existing_aria_assignments_and_empty_default_prefixes_keep_the_original_bank() {
    let common: Value = serde_json::from_str(include_str!(
        "../../../../../../tests/_fixtures/differential/lsp/component-global-attributes-8015/common.expected.json"
    ))
    .unwrap();
    for (source, marker) in [
        ("<template><Child  /></template>", " />"),
        ("<template><Child : /></template>", " />"),
        (
            "<template><Child aria-label=\"wide\" /></template>",
            "label",
        ),
    ] {
        let uri = Url::parse("file:///Default.vue").unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        let ctx = IdeContext::new(&state, &uri, source.find(marker).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(super::native::native_element_attribute_completions(
                &ctx,
                &[]
            ))
            .unwrap(),
            common
        );
    }
}

#[test]
fn assigned_aria_names_preserve_the_suffix_and_spaced_assignment_on_both_tag_kinds() {
    let aria: Vec<Value> = serde_json::from_str(ARIA).unwrap();
    let hidden = aria
        .iter()
        .find(|item| item["label"] == "aria-hidden")
        .unwrap();
    for tag in ["Child", "div"] {
        let source = format!("<template><{tag} aria-hidden = \"true\" /></template>");
        for (marker, prefix) in [("idden", "aria-h"), (" =", "aria-hidden")] {
            let uri = Url::parse("file:///Assigned.vue").unwrap();
            let state = ServerState::new();
            state
                .documents
                .open(uri.clone(), source.clone(), 1, "vue".to_string());
            let start = source.find("aria-hidden").unwrap();
            let end = source.find(marker).unwrap();
            let ctx = IdeContext::new(&state, &uri, end).unwrap();
            let expected = item_with_edit(hidden.clone(), &source, start, end, prefix);
            assert_eq!(
                serde_json::to_value(CompletionService::complete(&ctx)).unwrap(),
                json!([expected]),
                "assigned {tag} {prefix} completion"
            );
            let restored = format!("{}{}{}", &source[..start], prefix, &source[end..]);
            assert_eq!(
                restored, source,
                "the offered edit must preserve the assignment"
            );
        }
    }
}
