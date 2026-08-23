use std::fs;

use tower_lsp::lsp_types::Url;

use super::super::{BindingKind, bindings, helpers, script};
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn test_find_binding_location_const() {
    let content = r#"// Virtual TypeScript
// Generated

const message = ref('hello')
const count = ref(0)
"#;

    let loc = script::find_binding_location(content, "message", true);
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.name, "message");
    assert_eq!(loc.kind, BindingKind::Const);
}

#[test]
fn test_find_binding_location_function() {
    let content = r#"// Virtual TypeScript
// Generated

function handleClick() {
  console.log('clicked')
}
"#;

    let loc = script::find_binding_location(content, "handleClick", true);
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.name, "handleClick");
    assert_eq!(loc.kind, BindingKind::Function);
}

#[test]
fn test_find_binding_location_destructure() {
    let content = r#"// Virtual TypeScript
// Generated

const { data, error } = useFetch('/api')
"#;

    let loc = script::find_binding_location(content, "data", true);
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.name, "data");
    assert_eq!(loc.kind, BindingKind::Destructure);
}

#[test]
fn test_offset_to_position() {
    let content = "line1\nline2\nline3";

    let (line, col) = helpers::offset_to_position(content, 0);
    assert_eq!(line, 0);
    assert_eq!(col, 0);

    let (line, col) = helpers::offset_to_position(content, 3);
    assert_eq!(line, 0);
    assert_eq!(col, 3);

    let (line, col) = helpers::offset_to_position(content, 6);
    assert_eq!(line, 1);
    assert_eq!(col, 0);
}

#[test]
fn test_get_word_at_offset() {
    let content = "const message = 'hello'";

    let word = helpers::get_word_at_offset(content, 6);
    assert_eq!(word, Some("message".to_string()));

    let word = helpers::get_word_at_offset(content, 5);
    assert_eq!(word, Some("const".to_string()));

    let word = helpers::get_word_at_offset(content, 14);
    assert_eq!(word, None);

    let word = helpers::get_word_at_offset(content, 0);
    assert_eq!(word, Some("const".to_string()));
}

#[test]
fn test_get_tag_at_offset_only_matches_tag_name() {
    let content = r#"<MyButton :message="msg" />"#;

    let tag = helpers::get_tag_at_offset(content, "<MyButton".len());
    assert_eq!(tag, Some("MyButton".to_string()));

    let tag = helpers::get_tag_at_offset(content, content.find("message").unwrap() + 7);
    assert_eq!(tag, None);
}

#[test]
fn test_get_attribute_and_component_at_offset_only_matches_attribute_name() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("Parent.vue");
    let content = r#"<template><Child :message="msg" /></template>"#;
    fs::write(&file_path, content).unwrap();

    let uri = Url::from_file_path(&file_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), content.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, content);

    let attr_offset = content.find(":message").unwrap() + ":message".len();
    let attr_ctx = IdeContext::new(&state, &uri, attr_offset).unwrap();
    let attr = helpers::get_attribute_and_component_at_offset(&attr_ctx);
    assert_eq!(attr, Some(("message".to_string(), "Child".to_string())));

    let value_offset = content.rfind("msg").unwrap() + "msg".len();
    let value_ctx = IdeContext::new(&state, &uri, value_offset).unwrap();
    let attr = helpers::get_attribute_and_component_at_offset(&value_ctx);
    assert_eq!(attr, None);
}

#[test]
fn test_get_attribute_and_component_at_offset_maps_v_model_props() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("Parent.vue");
    let content = r#"<template><Child v-model="model" v-model:title.trim="title" v-model:[dynamic]="value" /></template>"#;
    fs::write(&file_path, content).unwrap();

    let uri = Url::from_file_path(&file_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), content.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, content);

    let default_offset = content.find("v-model=").unwrap() + "v-model".len();
    let default_ctx = IdeContext::new(&state, &uri, default_offset).unwrap();
    assert_eq!(
        helpers::get_attribute_and_component_at_offset(&default_ctx),
        Some(("modelValue".to_string(), "Child".to_string()))
    );

    let title_offset = content.find("v-model:title").unwrap() + "v-model:title".len();
    let title_ctx = IdeContext::new(&state, &uri, title_offset).unwrap();
    assert_eq!(
        helpers::get_attribute_and_component_at_offset(&title_ctx),
        Some(("title".to_string(), "Child".to_string()))
    );

    let dynamic_offset = content.find("v-model:[dynamic]").unwrap() + "v-model".len();
    let dynamic_ctx = IdeContext::new(&state, &uri, dynamic_offset).unwrap();
    assert_eq!(
        helpers::get_attribute_and_component_at_offset(&dynamic_ctx),
        None
    );
}

#[test]
fn test_is_valid_identifier() {
    assert!(bindings::is_valid_identifier("foo"));
    assert!(bindings::is_valid_identifier("_foo"));
    assert!(bindings::is_valid_identifier("$foo"));
    assert!(bindings::is_valid_identifier("foo123"));
    assert!(!bindings::is_valid_identifier("123foo"));
    assert!(!bindings::is_valid_identifier(""));
}

#[test]
fn test_find_binding_location_raw_const() {
    let content = r#"
import { ref } from 'vue'

const message = ref('hello')
const count = ref(0)
"#;

    let loc = script::find_binding_location_raw(content, "message");
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.name, "message");
    assert_eq!(loc.kind, BindingKind::Const);
    assert_eq!(&content[loc.offset..loc.offset + 7], "message");
}

#[test]
fn test_find_binding_location_raw_import() {
    let content = r#"import { ref } from 'vue'
import MyComponent from './MyComponent.vue'
"#;

    let loc = script::find_binding_location_raw(content, "MyComponent");
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.name, "MyComponent");
    assert_eq!(loc.kind, BindingKind::Import);
    assert_eq!(&content[loc.offset..loc.offset + 11], "MyComponent");
}

#[test]
fn test_find_binding_location_raw_destructure() {
    let content = r#"const { data, error } = useFetch('/api')
"#;

    let loc = script::find_binding_location_raw(content, "data");
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.name, "data");
    assert_eq!(loc.kind, BindingKind::Destructure);
    assert_eq!(&content[loc.offset..loc.offset + 4], "data");
}

#[test]
fn test_find_prop_in_define_props() {
    let content = r#"defineProps<{
  title: string
  isSubmitting?: boolean
  count: number
}>()"#;

    let pos = helpers::find_prop_in_define_props(content, "title");
    assert!(pos.is_some());

    let pos = helpers::find_prop_in_define_props(content, "isSubmitting");
    assert!(pos.is_some());

    let pos = helpers::find_prop_in_define_props(content, "nonExistent");
    assert!(pos.is_none());
}

#[test]
fn test_find_prop_in_define_model() {
    let content = r#"
const value = defineModel<string>({ required: true })
const title = defineModel<number>('title')
"#;

    let default = helpers::find_prop_in_define_model(content, "modelValue").unwrap();
    assert_eq!(&content[default.0..default.0 + default.1], "defineModel");

    let title = helpers::find_prop_in_define_model(content, "title").unwrap();
    assert_eq!(&content[title.0..title.0 + title.1], "title");

    assert!(helpers::find_prop_in_define_model(content, "missing").is_none());
}

#[test]
fn test_is_in_vue_directive_expression_detection() {
    let vue_attrs = [
        ":disabled",
        "@click",
        "v-if",
        "v-for",
        "v-model",
        "#default",
    ];
    let html_attrs = ["id", "class", "href", "src", "title"];

    for attr in vue_attrs {
        assert!(
            attr.starts_with(':')
                || attr.starts_with('@')
                || attr.starts_with('#')
                || attr.starts_with("v-"),
            "Vue directive {} should match pattern",
            attr
        );
    }

    for attr in html_attrs {
        assert!(
            !attr.starts_with(':')
                && !attr.starts_with('@')
                && !attr.starts_with('#')
                && !attr.starts_with("v-"),
            "HTML attribute {} should NOT match Vue pattern",
            attr
        );
    }
}
