//! A `v-for` alias is an immutable loop binding, so TypeScript preserves its
//! branch narrowing inside an authored listener callback (#11973).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_l0::String;

fn diagnostics(with_guard: bool) -> Option<Vec<(String, Option<u32>, String)>> {
    let condition = if with_guard {
        r#"<div v-if="isItem(entry)">
      <button type="button" @click.stop="() => $emit('pick', entry)">pick</button>
    </div>"#
    } else {
        r#"<button type="button" @click.stop="() => $emit('pick', entry)">pick</button>"#
    };
    let app = format!(
        r#"<script setup lang="ts">
import {{ isItem, type Entry, type Item }} from "./types";

defineProps<{{ entries: Entry[] }}>();
defineEmits<{{ pick: [item: Item] }}>();
</script>

<template>
  <div v-for="entry in entries" :key="entry.id">
    {condition}
  </div>
</template>
"#
    );
    let project = super::real_vue::create_project_case(
        "vfor-const-listener-narrowing-11973",
        &[
            (
                "src/types.ts",
                r#"export type Item = { kind: "item"; id: string };
export type Folder = { kind: "folder"; id: string; children: Item[] };
export type Entry = Item | Folder;
export const isItem = (value: Entry): value is Item => value.kind === "item";
"#,
            ),
            ("src/App.vue", app.as_str()),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    actual
}

#[test]
fn guarded_v_for_const_alias_stays_narrowed_in_listener_callback() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    assert_eq!(diagnostics(true), Some(Vec::new()));
}

#[test]
fn unguarded_v_for_alias_keeps_the_payload_error() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let actual = diagnostics(false).expect("native diagnostics");
    assert_eq!(actual.len(), 1, "{actual:#?}");
    assert_eq!(actual[0].0, "src/App.vue");
    assert_eq!(actual[0].1, Some(2345));
    assert_eq!(
        actual[0].2,
        "10:60:error Argument of type 'Entry' is not assignable to parameter of type 'Item'.\nType 'Folder' is not assignable to type 'Item'.\nTypes of property 'kind' are incompatible.\nType '\"folder\"' is not assignable to type '\"item\"'."
    );
}
