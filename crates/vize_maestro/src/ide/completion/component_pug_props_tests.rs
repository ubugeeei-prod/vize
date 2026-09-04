use std::fs;

use tower_lsp::lsp_types::{CompletionResponse, Url};

use super::CompletionService;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn template_component_prop_completion_resolves_pug_imported_props() {
    let dir = tempfile::tempdir().unwrap();
    let child_path = dir.path().join("HighlightMessage.vue");
    fs::write(
        &child_path,
        r#"<script setup lang="ts">
defineProps<{
  type?: string
  noIcon?: boolean
}>()
</script>
"#,
    )
    .unwrap();

    let source = r#"<script setup lang="ts">
import HighlightMessage from './HighlightMessage.vue'
</script>

<template lang="pug">
  highlight-message(type="success")
</template>
"#;
    let parent_path = dir.path().join("Parent.vue");
    fs::write(&parent_path, source).unwrap();

    let uri = Url::from_file_path(&parent_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    let offset = source.find("highlight-message").unwrap() + "highlight-message".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

    assert!(labels.iter().any(|label| label == "type"), "{labels:?}");
    assert!(labels.iter().any(|label| label == "no-icon"), "{labels:?}");
}

fn completion_labels(response: CompletionResponse) -> Vec<String> {
    match response {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    }
    .into_iter()
    .map(|item| item.label)
    .collect()
}
