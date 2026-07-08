use std::fs;

use tower_lsp::lsp_types::{CompletionResponse, Documentation, Url};

use crate::ide::{CompletionService, IdeContext};
use crate::server::ServerState;

#[test]
fn offers_native_attributes_in_opening_tag() {
    let source = r#"<script setup lang="ts">
const count = ref(0)
</script>
<template>
  <button
    
  >
    {{ count }}
  </button>
</template>
"#;
    let (state, uri) = state_with_document("NativeAttributeCompletion.vue", source);
    let offset = source.find("    \n  >").unwrap() + "    ".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let items = completion_items(CompletionService::complete(&ctx).unwrap());
    let labels = items
        .iter()
        .map(|item| item.label.clone())
        .collect::<Vec<_>>();

    assert!(has_label(&labels, "class"), "{labels:?}");
    assert!(has_label(&labels, "type"), "{labels:?}");
    assert!(has_label(&labels, "@click"), "{labels:?}");
    assert!(has_label(&labels, "v-if"), "{labels:?}");
    assert!(!has_label(&labels, "Transition"), "{labels:?}");
    assert!(!has_label(&labels, "vfor"), "{labels:?}");
    assert!(!has_label(&labels, "count"), "{labels:?}");
    let class_doc = markdown_doc(
        &items
            .iter()
            .find(|item| item.label == "class")
            .unwrap()
            .documentation,
    );
    assert!(class_doc.contains("```vue"), "got {class_doc:?}");
    assert!(
        class_doc.contains("Vue template syntax"),
        "got {class_doc:?}"
    );
}

#[test]
fn offers_event_shorthand_after_prefix() {
    let source = r#"<template>
  <button @cli></button>
</template>
"#;
    let (state, uri) = state_with_document("EventShorthandCompletion.vue", source);
    let offset = source.find("@cli").unwrap() + "@cli".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

    assert!(has_label(&labels, "@click"), "{labels:?}");
    assert!(!has_label(&labels, "v-if"), "{labels:?}");
    assert!(!has_label(&labels, "class"), "{labels:?}");
}

#[test]
fn keeps_bindings_in_multiline_event_handler() {
    let source = r#"<script setup lang="ts">
const count = ref(0)
</script>
<template>
  <button
    @click="
      () => {
        co
      }
    "
  >
    {{ count }}
  </button>
</template>
"#;
    let (state, uri) = state_with_document("MultilineEventCompletion.vue", source);
    let offset = source.find("        co").unwrap() + "        co".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

    assert!(has_label(&labels, "count"), "{labels:?}");
    assert!(!has_label(&labels, "v-if"), "{labels:?}");
    assert!(!has_label(&labels, "Transition"), "{labels:?}");
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
