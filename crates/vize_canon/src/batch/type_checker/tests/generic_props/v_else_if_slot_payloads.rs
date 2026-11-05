//! A slot host under a `v-else-if` chain keeps only its own discriminant test;
//! each branch's generic child still instantiates its slot from the narrowed props.
use super::*;

#[test]
fn v_else_if_slot_hosts_instantiate_each_branch_from_its_narrowed_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "v-else-if-slot-payloads",
        &[
            (
                "src/Pick.vue",
                r#"<script setup lang="ts" generic="T">
defineProps<{ items: T[] }>()
defineSlots<{ label(props: { item: T }): any }>()
</script>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Pick from './Pick.vue'
type Item =
  | { type: 'root' }
  | { type: 'text'; text: string }
  | { type: 'select'; items: string[] }
  | { type: 'numbers'; items: number[] }
const c = { type: 'select', items: [] } as Item
</script>

<template>
  <span v-if="c.type === 'root'"></span>
  <span v-else-if="c.type === 'text'">{{ c.text }}</span>
  <Pick v-else-if="c.type === 'select'" :items="c.items"><template #label="{ item }">{{ item.toUpperCase() }}</template></Pick>
  <Pick v-else-if="c.type === 'numbers'" :items="c.items"><template #label="{ item }">{{ item.toUpperCase() }}</template></Pick>
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };
    let _ = std::fs::remove_dir_all(&project_root);

    // Only the numeric branch's string method fails.
    let anchors: Vec<_> = snapshot
        .iter()
        .map(|(path, code, message)| {
            (
                path.as_str(),
                *code,
                message.split(' ').next().unwrap_or_default(),
            )
        })
        .collect();
    assert_eq!(
        anchors,
        vec![("src/App.vue", Some(2339), "15:95:error")],
        "{snapshot:#?}"
    );
}
