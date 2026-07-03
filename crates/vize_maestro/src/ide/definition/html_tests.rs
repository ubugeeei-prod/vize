use std::fs;

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Url};

use super::{DefinitionService, helpers};
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn static_boolean_attribute_is_detected_at_identifier_end_boundary() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("Static.vue");
    let source = r#"<template><button disabled></button></template>"#;
    fs::write(&source_path, source).unwrap();

    let uri = Url::from_file_path(&source_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    let offset = source.find("disabled").unwrap() + "disabled".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();

    assert_eq!(
        helpers::get_attribute_and_component_at_offset(&ctx),
        Some(("disabled".to_string(), "button".to_string()))
    );
}

#[test]
fn definition_resolves_native_html_tag_to_mdn_reference() {
    let (state, uri, source) = open_source(
        "NativeTag.vue",
        r#"<template>
  <button disabled>Save</button>
</template>
"#,
    );

    let offset = source.find("button").unwrap() + "button".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let location = scalar_location(DefinitionService::definition(&ctx).unwrap());

    assert_eq!(
        location.uri.as_str(),
        "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button"
    );
}

#[test]
fn definition_resolves_native_html_attribute_to_mdn_reference() {
    let (state, uri, source) = open_source(
        "NativeAttribute.vue",
        r#"<template>
  <button disabled>Save</button>
</template>
"#,
    );

    let offset = source.find("disabled").unwrap() + "disabled".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let location = scalar_location(DefinitionService::definition(&ctx).unwrap());

    assert_eq!(
        location.uri.as_str(),
        "https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Attributes/disabled"
    );
}

fn open_source(filename: &str, source: &str) -> (ServerState, Url, String) {
    let dir = tempfile::tempdir().unwrap();
    let dir_path = Box::leak(Box::new(dir)).path();
    let source_path = dir_path.join(filename);
    fs::write(&source_path, source).unwrap();

    let uri = Url::from_file_path(&source_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    (state, uri, source.to_string())
}

fn scalar_location(response: GotoDefinitionResponse) -> Location {
    match response {
        GotoDefinitionResponse::Scalar(location) => location,
        GotoDefinitionResponse::Array(mut locations) => {
            assert_eq!(locations.len(), 1);
            locations.remove(0)
        }
        GotoDefinitionResponse::Link(_) => panic!("expected location result"),
    }
}
