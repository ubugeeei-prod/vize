#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::disallowed_types
)]

use super::CompletionService;
use crate::{ide::IdeContext, server::ServerState};
use tower_lsp::lsp_types::{
    CompletionItemKind, CompletionResponse, CompletionTextEdit, Documentation, Url,
};

#[test]
fn event_completion_replaces_typed_shorthand_prefix() {
    let source = r#"<template>
  <button @c />
  <button @cli />
</template>
"#;
    let (state, uri) = state_with_document("EventCompletion.vue", source);

    let first_offset = source.find("@c").unwrap() + "@c".len();
    let ctx = IdeContext::new(&state, &uri, first_offset).unwrap();
    let items = completion_items(CompletionService::complete(&ctx).unwrap());
    let click = items
        .iter()
        .find(|item| item.label == "@click")
        .expect("@click completion should be present for @c");
    assert_eq!(click.kind, Some(CompletionItemKind::EVENT));
    let doc = markdown_doc(&click.documentation);
    assert!(doc.contains("```vue"), "got {doc:?}");
    assert!(doc.contains("Vue event handling"), "got {doc:?}");
    assert_eq!(click.insert_text, None);

    let edit = match click
        .text_edit
        .as_ref()
        .expect("@click completion should replace the typed token")
    {
        CompletionTextEdit::Edit(edit) => edit,
        CompletionTextEdit::InsertAndReplace(_) => panic!("expected a simple text edit"),
    };
    assert_eq!(edit.new_text, "@click=\"$1\"");
    assert_eq!(edit.range.start.line, 1);
    assert_eq!(edit.range.start.character, 10);
    assert_eq!(edit.range.end.line, 1);
    assert_eq!(edit.range.end.character, 12);

    let second_offset = source.find("@cli").unwrap() + "@cli".len();
    let ctx = IdeContext::new(&state, &uri, second_offset).unwrap();
    let labels = completion_labels(CompletionService::complete(&ctx).unwrap());
    assert!(has_label(&labels, "@click"), "{labels:?}");
}

fn state_with_document(name: &str, source: &str) -> (ServerState, Url) {
    let uri = Url::parse(&format!("file:///{name}")).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);
    (state, uri)
}

fn completion_labels(response: CompletionResponse) -> Vec<String> {
    completion_items(response)
        .into_iter()
        .map(|item| item.label)
        .collect()
}

fn completion_items(response: CompletionResponse) -> Vec<tower_lsp::lsp_types::CompletionItem> {
    match response {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    }
}

fn has_label(labels: &[String], expected: &str) -> bool {
    labels.iter().any(|label| label == expected)
}

fn markdown_doc(doc: &Option<Documentation>) -> &str {
    match doc.as_ref().expect("completion should include docs") {
        Documentation::MarkupContent(content) => &content.value,
        Documentation::String(value) => value,
    }
}
