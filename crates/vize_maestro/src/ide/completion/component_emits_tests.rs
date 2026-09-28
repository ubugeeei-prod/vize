use std::fs;

use tower_lsp::lsp_types::{CompletionResponse, CompletionTextEdit, Url};

use super::CompletionService;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn imported_component_emits_complete_in_empty_and_at_prefixed_attributes() {
    let dir = tempfile::tempdir().unwrap();
    let child_path = dir.path().join("Child.vue");
    fs::write(
        &child_path,
        "<script setup lang='ts'>defineEmits<{ select: [id: number] }>()</script>",
    )
    .unwrap();
    let source = "<script setup lang='ts'>\nimport Child from './Child.vue'\n</script>\n<template>\n  <Child  />\n  <Child @s />\n</template>";
    let parent_path = dir.path().join("Parent.vue");
    fs::write(&parent_path, source).unwrap();
    let uri = Url::from_file_path(&parent_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    for (needle, offset) in [
        ("<Child  />", "<Child ".len()),
        ("<Child @s />", "<Child @s".len()),
    ] {
        let offset = source.find(needle).unwrap() + offset;
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let items = match CompletionService::complete(&ctx).unwrap() {
            CompletionResponse::Array(items) => items,
            CompletionResponse::List(list) => list.items,
        };
        let matches: Vec<_> = items
            .iter()
            .filter(|item| item.label == "@select")
            .collect();
        assert_eq!(matches.len(), 1, "{needle}");
        let event = matches[0];
        if needle.contains("@s") {
            let Some(CompletionTextEdit::Edit(edit)) = &event.text_edit else {
                panic!("prefixed event completion must replace the partial token");
            };
            assert_eq!(edit.new_text, "@select=\"$1\"");
        } else {
            assert_eq!(event.insert_text.as_deref(), Some("@select=\"$1\""));
        }
    }
}
