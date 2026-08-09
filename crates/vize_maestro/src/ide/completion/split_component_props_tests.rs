use std::fs;

use tower_lsp::lsp_types::{CompletionItem, CompletionResponse, Url};

use super::CompletionService;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn split_script_component_props_include_local_inherited_and_unsaved_members() {
    let workspace = tempfile::tempdir().expect("temporary workspace");
    let src = workspace.path().join("src");
    let primitive = src.join("Primitive");
    let toolbar = src.join("Toolbar");
    fs::create_dir_all(&primitive).expect("primitive directory");
    fs::create_dir_all(&toolbar).expect("toolbar directory");
    fs::write(
        primitive.join("index.ts"),
        "export interface PrimitiveProps { as?: string; asChild?: boolean; primitiveKind: 'button' | 'div' }\n",
    )
    .expect("primitive props");
    let child_source = r#"<script lang="ts">
import type { PrimitiveProps } from '@/Primitive'

export interface ToolbarButtonProps extends PrimitiveProps {
  disabled?: boolean
}
</script>

<script setup lang="ts">
defineProps<ToolbarButtonProps>()
</script>
"#;
    let child_path = toolbar.join("ToolbarButton.vue");
    fs::write(&child_path, child_source).expect("child component");
    let parent_source = r#"<script setup lang="ts">
import ToolbarButton from './ToolbarButton.vue'
</script>

<template><ToolbarButton  /></template>
"#;
    let parent_path = toolbar.join("ToolbarToggleItem.vue");
    fs::write(&parent_path, parent_source).expect("parent component");
    let parent_uri = Url::from_file_path(&parent_path).expect("parent URI");
    let child_uri = Url::from_file_path(&child_path).expect("child URI");
    let state = ServerState::new();
    state.documents.open(
        parent_uri.clone(),
        parent_source.to_owned(),
        1,
        "vue".to_owned(),
    );
    let offset = parent_source.find("<ToolbarButton  />").unwrap() + "<ToolbarButton ".len();
    let ctx = IdeContext::new(&state, &parent_uri, offset).expect("IDE context");

    let items = completion_items(CompletionService::complete(&ctx).expect("completion"));
    assert_prop(&items, "disabled", "prop: boolean (optional)");
    assert_prop(&items, "as", "prop: string (optional)");
    assert_prop(&items, "as-child", "prop: boolean (optional)");
    assert_prop(
        &items,
        "primitive-kind",
        "prop: 'button' | 'div' (required)",
    );

    let changed = child_source.replace(
        "  disabled?: boolean\n",
        "  disabled?: boolean\n  vizeOracleProbe?: boolean\n",
    );
    state
        .documents
        .open(child_uri, changed, 2, "vue".to_owned());
    let items = completion_items(CompletionService::complete(&ctx).expect("live completion"));
    assert_prop(&items, "vize-oracle-probe", "prop: boolean (optional)");
}

fn completion_items(response: CompletionResponse) -> Vec<CompletionItem> {
    match response {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    }
}

fn assert_prop(items: &[CompletionItem], label: &str, detail: &str) {
    let item = items
        .iter()
        .find(|item| item.label == label)
        .unwrap_or_else(|| panic!("missing {label:?} in {items:#?}"));
    assert_eq!(item.detail.as_deref(), Some(detail));
}
