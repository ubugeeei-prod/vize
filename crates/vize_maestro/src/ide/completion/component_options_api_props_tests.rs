use std::fs;

use tower_lsp::lsp_types::{CompletionResponse, Url};

use super::CompletionService;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn template_component_prop_completion_resolves_options_api_runtime_props() {
    let dir = tempfile::tempdir().unwrap();
    let child_path = dir.path().join("Child.vue");
    fs::write(
        &child_path,
        r#"<script lang="ts">
export default {
  props: {
    count: {
      default: 2
    },
    html: null,
    vizeOracleProbe: {
      type: Boolean,
      default: false,
    },
  },
}
</script>
"#,
    )
    .unwrap();

    let source = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>
<template><Child  /></template>
"#;
    let parent_path = dir.path().join("Parent.vue");
    fs::write(&parent_path, source).unwrap();

    let uri = Url::from_file_path(&parent_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    let offset = source.find("<Child  />").unwrap() + "<Child ".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

    assert!(has_label(&labels, "count"), "{labels:?}");
    assert!(has_label(&labels, "html"), "{labels:?}");
    assert!(has_label(&labels, "vize-oracle-probe"), "{labels:?}");
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
