//! Generic construct signatures intersected with Vue's runtime constructor (#10416).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_l0::String;

const GENERIC_LIST: &str = r#"import { defineComponent, h, type VNode } from 'vue'

const impl = defineComponent({
  props: { items: { type: Array, required: true } },
  setup: (props, { slots }) => () => h('div', props.items.map((item) => slots.default?.({ item }))),
})

export const GenericList = impl as typeof impl & (new <T>(
  props: { items: T[] },
  slots: { default: (arg: { item: T }) => VNode[] },
) => {
  $props: { items: T[] }
  $slots: { default?: (arg: { item: T }) => VNode[] }
})

// Vuetify's second constructor argument contains covariant slot payloads,
// not slot callbacks. Its erased T must not become another inference input.
export const GenericIterator = impl as typeof impl & (new <T>(
  props: { items?: readonly T[] },
  slots: { default: { items: readonly { raw: T }[] } },
) => {
  $props: { items?: readonly T[] }
  $slots: { default?: (arg: { items: readonly { raw: T }[] }) => VNode[] }
})
"#;

#[test]
fn generic_constructor_intersection_slots_infer_from_bound_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "generic-constructor-slots-10416",
        &[
            ("src/GenericList.ts", GENERIC_LIST),
            (
                "src/Plain.vue",
                r#"<script setup lang="ts">
import { GenericList, GenericIterator } from './GenericList'
const users = [{ id: 1, name: 'Ada' }]
</script>
<template>
  <GenericList :items="users">
    <template #default="{ item }">{{ item.name }}{{ item.id.toFixed() }}</template>
  </GenericList>
  <GenericList :items="users" v-slot="{ item }">{{ item.name.toUpperCase() }}</GenericList>
  <GenericIterator :items="users">
    <template #default="{ items }">
      <div v-for="item in items" :key="item.raw.id">{{ item.raw.name }}</div>
    </template>
  </GenericIterator>
</template>
"#,
            ),
            (
                "src/Invalid.vue",
                r#"<script setup lang="ts">
import { GenericList, GenericIterator } from './GenericList'
const users = [{ id: 1, name: 'Ada' }]
</script>
<template>
  <GenericList :items="users" v-slot="{ item }">{{ item.nope }}</GenericList>
  <GenericIterator :items="users" v-slot="{ items }">{{ items[0].raw.nope }}</GenericIterator>
</template>
"#,
            ),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(
        actual,
        Some(vec![
            (
                String::from("src/Invalid.vue"),
                Some(2339),
                String::from(
                    "6:57:error Property 'nope' does not exist on type '{ id: number; name: string; }'."
                ),
            ),
            (
                String::from("src/Invalid.vue"),
                Some(2339),
                String::from(
                    "7:70:error Property 'nope' does not exist on type '{ id: number; name: string; }'."
                ),
            ),
        ]),
        "the reporter's valid usages must pass and the invalid member must remain typed",
    );
}
