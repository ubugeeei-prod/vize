use std::fs;

use tower_lsp::lsp_types::{CompletionItem, CompletionResponse, Url};

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

#[test]
fn template_completion_prefers_typed_macro_props_in_interpolation_and_handler() {
    let source = r#"<script setup lang="ts">
const props = defineProps<{ title: string; count?: number }>()
const emit = defineEmits<{ select: [id: number] }>()
</script>
<template><button @click="emit('select', props.count ?? 0)">{{ props.title }}</button></template>"#;
    let (state, uri) = state_with_document("MacroProps.vue", source);
    for (cursor, name, detail) in [
        ("emit('select'", "title", "prop: string"),
        ("props.title", "count", "prop: number"),
    ] {
        let offset = source.find(cursor).unwrap();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let items = completion_items(CompletionService::complete(&ctx).unwrap());
        let props: Vec<_> = items.iter().filter(|item| item.label == name).collect();
        assert_eq!(props.len(), 1, "{cursor}: {name}");
        assert_eq!(props[0].detail.as_deref(), Some(detail));
    }
}

fn completion_labels(response: CompletionResponse) -> Vec<String> {
    completion_items(response)
        .into_iter()
        .map(|item| item.label)
        .collect()
}

fn completion_items(response: CompletionResponse) -> Vec<CompletionItem> {
    match response {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    }
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
