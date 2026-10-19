//! Hover over template expressions, components and their attributes.

use std::fs;

use tower_lsp::lsp_types::{HoverContents, Url};

use super::super::HoverService;
use crate::{ide::IdeContext, server::ServerState};

const CHILD: &str = r#"<script setup lang="ts">
/** Row heading. */
defineProps<{ title: string; count?: number }>()
defineEmits<{ (e: 'save-item', id: number): void }>()
</script>
<template><slot /></template>
"#;

const PARENT: &str = r#"<script setup lang="ts">
import { ref, computed, reactive } from 'vue'
import RowList from './RowList.vue'
const items = ref([{ id: 1, label: 'One' }])
const total = computed(() => items.value.length)
const form = reactive({ name: '' })
const LIMIT = 10
function select(id: number) { return id }
</script>

<template>
  <ul>
    <li v-for="(item, index) in items" :key="item.id" @click="select(item.id)">
      {{ index }} {{ item.label }} {{ total }} {{ form.name }} {{ LIMIT }}
    </li>
  </ul>
  <RowList title="t" :count="total" @save-item="select" />
</template>
"#;

fn hover(needle: &str, nth: usize) -> Option<String> {
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
    let HoverContents::Markup(markup) = HoverService::hover(&ctx)?.contents else {
        panic!("expected markdown hover");
    };
    Some(markup.value)
}

/// The first `count` markdown sections of the hover.
fn head(needle: &str, nth: usize, count: usize) -> Option<Vec<String>> {
    hover(needle, nth).map(|value| value.split("\n\n").take(count).map(str::to_owned).collect())
}

fn sections(parts: &[&str]) -> Option<Vec<String>> {
    Some(parts.iter().map(|part| (*part).to_owned()).collect())
}

#[test]
fn template_scope_bindings_hover_with_their_origin() {
    let v_for = "```vue\n<li v-for=\"(item, index) in items\" :key=\"item.id\" @click=\"select(item.id)\">\n```";
    for (needle, nth, expected) in [
        (
            "{{ index }}",
            4,
            sections(&["**index**", "_v-for scope binding_", v_for]),
        ),
        (
            "{{ item.label }}",
            4,
            sections(&["**item**", "_v-for scope binding_", v_for]),
        ),
        (
            "{{ total }}",
            4,
            sections(&["**total**", "_Template binding_"]),
        ),
        (
            "{{ form.name }}",
            4,
            sections(&["**form**", "_Template binding_"]),
        ),
        (
            "{{ LIMIT }}",
            4,
            sections(&["**LIMIT**", "_Template binding_"]),
        ),
        (
            "select(item.id)",
            2,
            sections(&["**select**", "_Template binding_"]),
        ),
        (
            "in items",
            4,
            sections(&["**items**", "_Template binding_"]),
        ),
    ] {
        let count = expected.as_ref().map_or(0, Vec::len);
        assert_eq!(head(needle, nth, count), expected, "{needle} @{nth}");
    }
}

#[test]
fn component_tags_props_and_events_hover_with_the_child_contract() {
    assert_eq!(
        head("<RowList", 3, 2),
        sections(&[
            "```typescript\nconst RowList: VueComponent &\n{\n  props: { title: string; count?: number };\n  emits: { (e: 'save-item', id: number): void };\n}\n```",
            "Vue component: RowList.vue",
        ])
    );
    for (needle, nth, expected) in [
        (
            "title=\"t\"",
            2,
            sections(&[
                "**title**",
                "_Component prop_",
                "```typescript\ntitle: string\n```",
            ]),
        ),
        (
            ":count=",
            2,
            sections(&[
                "**count**",
                "_Component prop_",
                "```typescript\ncount?: number\n```",
            ]),
        ),
        (
            "@save-item",
            4,
            sections(&[
                "**@save-item**",
                "_Component event_",
                "```typescript\nsave-item: [id: number]\n```",
            ]),
        ),
    ] {
        assert_eq!(head(needle, nth, 3), expected, "{needle} @{nth}");
    }
}
