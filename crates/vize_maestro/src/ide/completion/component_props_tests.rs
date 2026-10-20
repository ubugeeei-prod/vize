use std::fs;

use tower_lsp::lsp_types::{
    CompletionResponse, CompletionTextEdit, Documentation, TextDocumentContentChangeEvent, Url,
};

use super::CompletionService;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn template_component_prop_completion_resolves_imported_script_setup_props() {
    let dir = tempfile::tempdir().unwrap();
    let child_path = dir.path().join("Child.vue");
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
import Child from './Child.vue'
</script>

<template>
  <Child  />
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

    let offset = source.find("<Child  />").unwrap() + "<Child ".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let items = completion_items(CompletionService::complete(&ctx).unwrap());
    let labels = items
        .iter()
        .map(|item| item.label.clone())
        .collect::<Vec<_>>();

    assert!(has_label(&labels, "some-message"), "{labels:?}");
    assert!(has_label(&labels, "disabled"), "{labels:?}");
    let prop = items
        .iter()
        .find(|item| item.label == "some-message")
        .expect("some-message prop completion should be present");
    let doc = markdown_doc(&prop.documentation);
    assert_eq!(
        doc,
        r#"**Prop `someMessage`**

_Required component prop_

```typescript
interface Props {
  someMessage: string
}
```

**Requirement**

Required

**Example**

```vue
<Component some-message="..." />
<Component :someMessage="value" />
```

**Docs**

[Vue Component Props](https://vuejs.org/guide/components/props.html)"#
    );
}

#[test]
fn nested_template_slot_completion_only_lists_component_slots() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("MySelect.vue"),
        r#"<script setup lang="ts">
defineSlots<{
  selection?(props: { item: string }): unknown
  item?(props: { item: string }): unknown
}>()
</script>"#,
    )
    .unwrap();

    for (suffix, prefix) in [("#></template>", "#"), ("v-slot:></template>", "v-slot:")] {
        let source = vize_l0::cstr!(
            r#"<script setup lang="ts">
import MySelect from "./MySelect.vue"
</script>
<template>
  <MySelect><template {suffix}</MySelect>
</template>"#
        );
        let parent_path = dir
            .path()
            .join(vize_l0::cstr!("Parent-{}.vue", prefix.len()));
        fs::write(&parent_path, &source).unwrap();
        let uri = Url::from_file_path(&parent_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, &source);

        let offset = source
            .find(vize_l0::cstr!("<template {prefix}").as_str())
            .unwrap()
            + "<template ".len()
            + prefix.len();
        let ctx = IdeContext::at_completion(&state, &uri, offset).unwrap();
        let items = completion_items(CompletionService::complete(&ctx).unwrap());
        let labels = items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        let edits = items
            .iter()
            .map(|item| match item.text_edit.as_ref().unwrap() {
                CompletionTextEdit::Edit(edit) => edit.new_text.clone(),
                CompletionTextEdit::InsertAndReplace(_) => panic!("expected a simple text edit"),
            })
            .collect::<Vec<_>>();

        assert_eq!(labels, vec!["selection", "item"], "{prefix}: {items:#?}");
        assert_eq!(
            edits,
            vec![
                vize_l0::cstr!(r#"{prefix}selection="{{ ${{1:item}} }}""#),
                vize_l0::cstr!(r#"{prefix}item="{{ ${{1:item}} }}""#),
            ]
        );
    }
}

#[test]
fn incomplete_nested_template_slot_completion_recovers_template_context() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("MySelect.vue"),
        "<script setup lang='ts'>defineSlots<{ selection(): unknown; item(): unknown }>()</script>",
    )
    .unwrap();
    let source = r#"<script setup lang="ts">
import MySelect from "./MySelect.vue"
</script>
<template>
  <MySelect><template #
  <span>markup typed after the incomplete tag</span>
  </MySelect>
</template>"#;
    let parent_path = dir.path().join("Parent.vue");
    fs::write(&parent_path, source).unwrap();
    let uri = Url::from_file_path(&parent_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    let offset = source.find("<template #").unwrap() + "<template #".len();
    let ctx = IdeContext::at_completion(&state, &uri, offset).unwrap();
    let labels = completion_items(CompletionService::complete(&ctx).unwrap())
        .into_iter()
        .map(|item| item.label)
        .collect::<Vec<_>>();

    assert_eq!(labels, vec!["selection", "item"]);
    assert!(!labels.iter().any(|label| matches!(
        label.as_str(),
        "template" | "script setup" | "style" | "v-if" | "@click" | ":" | "#"
    )));
}

#[test]
fn template_component_prop_completion_replaces_dynamic_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let child_path = dir.path().join("Child.vue");
    fs::write(
        &child_path,
        r#"<script setup lang="ts">
defineProps<{
  someMessage: string
}>()
</script>
"#,
    )
    .unwrap();

    let source = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child :so />
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

    let offset = source.find(":so").unwrap() + ":so".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let items = completion_items(CompletionService::complete(&ctx).unwrap());
    let prop = items
        .iter()
        .find(|item| item.label == "someMessage")
        .expect("dynamic prop completion should use the camelCase label");

    let edit = match prop
        .text_edit
        .as_ref()
        .expect("dynamic prop completion should replace the typed prefix")
    {
        CompletionTextEdit::Edit(edit) => edit,
        CompletionTextEdit::InsertAndReplace(_) => panic!("expected a simple text edit"),
    };
    assert_eq!(edit.new_text, ":someMessage=\"$1\"");
}

#[test]
fn template_component_prop_completion_tracks_open_child_buffer() {
    let dir = tempfile::tempdir().unwrap();
    let child_path = dir.path().join("Child.vue");
    fs::write(
        &child_path,
        "<script setup lang=\"ts\">defineProps<{ staleProp: string }>()</script>",
    )
    .unwrap();
    let source = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>
<template><Child  /></template>
"#;
    let parent_path = dir.path().join("Parent.vue");
    fs::write(&parent_path, source).unwrap();

    let parent_uri = Url::from_file_path(&parent_path).unwrap();
    let child_uri = Url::from_file_path(&child_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(parent_uri.clone(), source.to_string(), 1, "vue".to_string());
    state.documents.open(
        child_uri.clone(),
        "<script setup lang=\"ts\">defineProps<{ freshProp: string }>()</script>".to_string(),
        3,
        "vue".to_string(),
    );

    let offset = source.find("<Child  />").unwrap() + "<Child ".len();
    let ctx = IdeContext::new(&state, &parent_uri, offset).unwrap();
    let labels = completion_labels(CompletionService::complete(&ctx).unwrap());
    assert!(has_label(&labels, "fresh-prop"), "{labels:?}");
    assert!(!has_label(&labels, "stale-prop"), "{labels:?}");

    state.documents.apply_changes(
        &child_uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "<script setup lang=\"ts\">defineProps<{ newerProp: string }>()</script>"
                .to_string(),
        }],
        4,
    );
    let labels = completion_labels(CompletionService::complete(&ctx).unwrap());
    assert!(has_label(&labels, "newer-prop"), "{labels:?}");
    assert!(!has_label(&labels, "fresh-prop"), "{labels:?}");

    state.documents.close(&child_uri);
    let labels = completion_labels(CompletionService::complete(&ctx).unwrap());
    assert!(has_label(&labels, "stale-prop"), "{labels:?}");
    assert!(!has_label(&labels, "newer-prop"), "{labels:?}");

    state.documents.open(
        child_uri,
        "<script setup lang=\"ts\">defineProps<{ otherProp: string }>()</script>".to_string(),
        4,
        "vue".to_string(),
    );
    let labels = completion_labels(CompletionService::complete(&ctx).unwrap());
    assert!(has_label(&labels, "other-prop"), "{labels:?}");
    assert!(!has_label(&labels, "newer-prop"), "{labels:?}");
}

#[cfg(feature = "native")]
#[test]
fn template_component_prop_completion_is_preserved_with_corsa() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            dir.path().join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
        )
        .unwrap();
        fs::write(
            src.join("vue.d.ts"),
            r#"declare module "vue" {
  export type DefineComponent<P = any, _B = any, _D = any> = { new(): { $props: P } };
  export interface Ref<T = unknown> { value: T }
  export interface ShallowRef<T = unknown> extends Ref<T> {}
}
"#,
        )
        .unwrap();
        fs::write(
            src.join("Child.vue"),
            r#"<script setup lang="ts">
defineProps<{
  someMessage: string
  count?: number
}>()
</script>
"#,
        )
        .unwrap();

        let source = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child :so />
</template>
"#;
        let parent_path = src.join("Parent.vue");
        fs::write(&parent_path, source).unwrap();

        let uri = Url::from_file_path(&parent_path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(dir.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let bridge = std::sync::Arc::new(vize_canon::CorsaBridge::with_config(
            vize_canon::CorsaBridgeConfig {
                corsa_path: Some(corsa_path),
                working_dir: Some(dir.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            },
        ));
        bridge.spawn().await.unwrap();

        let offset = source.find(":so").unwrap() + ":so".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(
            CompletionService::complete_with_corsa(&ctx, Some(bridge.clone()))
                .await
                .unwrap(),
        );
        let _ = bridge.shutdown().await;

        assert!(has_label(&labels, "someMessage"), "{labels:?}");
        assert!(has_label(&labels, "count"), "{labels:?}");
    });
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

fn markdown_doc(doc: &Option<Documentation>) -> &str {
    match doc.as_ref().expect("completion should include docs") {
        Documentation::MarkupContent(content) => &content.value,
        Documentation::String(value) => value,
    }
}

fn has_label(labels: &[String], expected: &str) -> bool {
    labels.iter().any(|label| label == expected)
}

#[cfg(feature = "native")]
fn resolve_tsgo_binary() -> Option<std::path::PathBuf> {
    crate::test_support::tsgo()
}
