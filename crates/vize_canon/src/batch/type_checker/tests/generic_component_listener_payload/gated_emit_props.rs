//! Listener payloads around the props-independent emit-props gate: a computed
//! `:is` union takes the gated call, a static child the direct one, and a
//! generic child still instantiates its payload from the bound props.
use super::*;

#[test]
fn gated_and_generic_children_keep_their_listener_payloads() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "gated-emit-props-listener-payloads",
        &[
            (
                "src/Save.vue",
                r#"<script setup lang="ts">
defineProps<{ label?: string; column?: { id: string } }>()
defineEmits<{ save: [value: number]; headerWheel: [delta: number] }>()
</script>
"#,
            ),
            (
                "src/Plain.vue",
                r#"<script setup lang="ts">
defineProps<{ column?: { id: string } }>()
</script>
"#,
            ),
            (
                "src/Pick.vue",
                r#"<script setup lang="ts" generic="T">
defineProps<{ value: T }>()
defineEmits<{ pick: [value: T] }>()
</script>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Save from './Save.vue'
import Plain from './Plain.vue'
import Pick from './Pick.vue'

const label = 'label'
const columns = [{ id: 'a', type: 'save' as 'save' | 'plain' }]
const components = { save: Save, plain: Plain }
function onNumber(value: number) { return value.toFixed() }
function onString(value: string) { return value.toUpperCase() }
</script>

<template>
  <Save :label="label" @save="onNumber" />
  <Save :label="label" @save="onString" />
  <Save :label="label" @save="(value) => value.toFixed()" />
  <component
    v-for="column in columns"
    :key="column.id"
    :is="components[columns.find(c => c.id === column.id)!.type] ?? Save"
    :column="column"
    @header-wheel="onNumber"
  />
  <Pick :value="'text'" @pick="onString" />
  <Pick :value="1" @pick="onString" />
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

    // The wrong closed handler fails at `save`; the generic payload follows
    // `:value`, so only the numeric `Pick` rejects the string handler.
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
        vec![
            ("src/App.vue", Some(2322), "15:25:error"),
            ("src/App.vue", Some(2322), "25:21:error"),
        ],
        "{snapshot:#?}"
    );
}
