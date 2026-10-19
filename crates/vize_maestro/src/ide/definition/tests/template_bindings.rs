//! Go-to-definition from template expressions onto authored declarations.

use std::fs;

use tower_lsp::lsp_types::{GotoDefinitionResponse, Url};

use super::super::DefinitionService;
use crate::{ide::IdeContext, server::ServerState};

const CHILD: &str = r#"<script setup lang="ts">
defineProps<{ title: string; count?: number }>()
defineEmits<{ (e: 'save-item', id: number): void }>()
defineSlots<{ row(props: { item: { id: number; label: string }; index: number }): any }>()
</script>
<template><slot name="row" :item="{ id: 1, label: 'a' }" :index="0" /></template>
"#;

const PARENT: &str = r#"<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'
import RowList from './RowList.vue'
const items = ref([{ id: 1, label: 'One', tags: ['a'] }])
const selected = ref<number | null>(null)
const input = useTemplateRef<HTMLInputElement>('field')
function select(id: number) { selected.value = id }
</script>

<template>
  <input ref="field" :value="selected" />
  <ul>
    <li v-for="({ id, label, tags }, index) in items" :key="id" @click="select(id)">
      {{ index }}: {{ label }}
      <span v-for="tag in tags" :key="tag">{{ tag }}</span>
    </li>
  </ul>
  <RowList title="t" :count="items.length" @save-item="select">
    <template #row="{ item, index }">{{ item.label }} {{ index }}</template>
  </RowList>
</template>
"#;

/// `(file, line, character)` of the definition at the `nth` byte of the last
/// occurrence of `needle` in the parent.
fn definition(needle: &str, nth: usize) -> Option<(String, u32, u32)> {
    let dir = crate::test_support::tempdir().unwrap();
    fs::write(dir.path().join("RowList.vue"), CHILD).unwrap();
    let path = dir.path().join("Parent.vue");
    fs::write(&path, PARENT).unwrap();
    let uri = Url::from_file_path(&path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), PARENT.to_owned(), 1, "vue".to_owned());
    state.update_virtual_docs(&uri, PARENT);
    let offset = PARENT.rfind(needle).unwrap() + nth;
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let location = match DefinitionService::definition(&ctx)? {
        GotoDefinitionResponse::Scalar(location) => location,
        GotoDefinitionResponse::Array(mut locations) => locations.remove(0),
        GotoDefinitionResponse::Link(mut links) => {
            let link = links.remove(0);
            tower_lsp::lsp_types::Location::new(link.target_uri, link.target_selection_range)
        }
    };
    let file = location
        .uri
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .unwrap_or_default()
        .to_owned();
    Some((
        file,
        location.range.start.line,
        location.range.start.character,
    ))
}

#[test]
fn template_bindings_resolve_to_their_authored_declarations() {
    let parent = |line, character| Some(("Parent.vue".to_owned(), line, character));
    let child = |line, character| Some(("RowList.vue".to_owned(), line, character));
    for (needle, nth, expected) in [
        // Setup bindings, refs and functions.
        (":value=\"selected\"", 9, parent(4, 6)),
        (":count=\"items.length\"", 9, parent(3, 6)),
        ("select(id)", 2, parent(6, 9)),
        ("ref=\"field\"", 6, parent(5, 6)),
        // Destructured v-for aliases, the index, and a nested v-for.
        ("select(id)", 7, parent(12, 18)),
        ("{{ index }}: ", 4, parent(12, 37)),
        ("{{ label }}", 4, parent(12, 22)),
        ("tag in tags", 8, parent(12, 29)),
        ("{{ tag }}", 4, parent(14, 19)),
        // Slot props shadow the outer v-for index of the same name.
        ("{{ item.label }}", 4, parent(18, 22)),
        ("{{ index }}</template>", 4, parent(18, 28)),
        // Component contract: tag, props, emits and slots of the child.
        ("<RowList title", 3, child(0, 0)),
        ("title=\"t\"", 2, child(1, 14)),
        (":count=", 2, child(1, 29)),
        ("@save-item", 3, child(2, 19)),
        ("#row=", 2, child(3, 14)),
    ] {
        assert_eq!(definition(needle, nth), expected, "{needle} @{nth}");
    }
}
