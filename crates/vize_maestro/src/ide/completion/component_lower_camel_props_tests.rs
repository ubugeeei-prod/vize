use std::fs;

use tower_lsp::lsp_types::CompletionResponse;

use super::CompletionService;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn template_component_prop_completion_resolves_lower_camel_import_as_kebab_tag() {
    let dir = tempfile::tempdir().unwrap();
    let child_dir = dir.path().join("descriptionItem");
    fs::create_dir_all(&child_dir).unwrap();
    let child_path = child_dir.join("index.vue");
    fs::write(
        &child_path,
        r#"<script setup lang="ts">
defineProps<{
  someMessage: string
  disabled?: boolean
}>()
</script>
"#,
    )
    .unwrap();

    let source = r#"<script setup lang="ts">
import descriptionItem from './descriptionItem/index.vue'
</script>

<template>
  <description-item  />
</template>
"#;
    let parent_path = dir.path().join("Parent.vue");
    fs::write(&parent_path, source).unwrap();

    let parent_uri = tower_lsp::lsp_types::Url::from_file_path(&parent_path).unwrap();
    let child_uri = tower_lsp::lsp_types::Url::from_file_path(&child_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(parent_uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&parent_uri, source);

    let offset = source.find("<description-item  />").unwrap() + "<description-item ".len();
    let ctx = IdeContext::new(&state, &parent_uri, offset).unwrap();
    let labels = completion_labels(CompletionService::complete(&ctx).unwrap());
    assert!(has_label(&labels, "some-message"), "{labels:?}");
    assert!(has_label(&labels, "disabled"), "{labels:?}");

    let changed_child_source =
        "<script setup lang=\"ts\">defineProps<{ freshProbe: string }>()</script>";
    state.documents.open(
        child_uri.clone(),
        changed_child_source.to_string(),
        2,
        "vue".to_string(),
    );
    state.update_virtual_docs(&child_uri, changed_child_source);
    let labels = completion_labels(CompletionService::complete(&ctx).unwrap());
    assert!(has_label(&labels, "fresh-probe"), "{labels:?}");
    assert!(!has_label(&labels, "some-message"), "{labels:?}");
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

fn has_label(labels: &[String], expected: &str) -> bool {
    labels.iter().any(|label| label == expected)
}
