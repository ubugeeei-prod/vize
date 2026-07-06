use std::fs;

use tower_lsp::lsp_types::{CompletionResponse, Url};

use crate::ide::{CompletionService, IdeContext};
use crate::server::ServerState;

#[test]
fn script_completion_lists_reactive_binding_once() {
    let source = r#"<script setup lang="ts">
import { ref, computed } from 'vue'
const st = ref(0)
const ts = computed(() => st.value * 2)
st
</script>
"#;
    let (state, uri) = state_with_document("ScriptDedup.vue", source);
    let offset = source.rfind("st\n").unwrap() + 2;
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

    assert_eq!(labels.iter().filter(|l| l.as_str() == "st").count(), 1);
    assert_eq!(labels.iter().filter(|l| l.as_str() == "ts").count(), 1);
}

#[test]
fn template_completion_lists_reactive_binding_once() {
    let source = r#"<script setup lang="ts">
import { ref, computed } from 'vue'
const st = ref(0)
const ts = computed(() => st.value * 2)
</script>
<template>
  <div>{{ st }}</div>
</template>
"#;
    let (state, uri) = state_with_document("TemplateDedup.vue", source);
    let offset = source.rfind("st }}").unwrap() + 2;
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

    assert_eq!(labels.iter().filter(|l| l.as_str() == "st").count(), 1);
    assert_eq!(labels.iter().filter(|l| l.as_str() == "ts").count(), 1);
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

fn state_with_document(name: &str, source: &str) -> (ServerState, Url) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join(name);
    fs::write(&source_path, source).unwrap();

    let uri = Url::from_file_path(&source_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    (state, uri)
}
