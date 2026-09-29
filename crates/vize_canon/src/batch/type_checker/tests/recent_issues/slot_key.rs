use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn slot_outlet_key_matches_declared_and_inferred_payloads() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "slot-outlet-key-payload",
        &[
            (
                "src/Declared.vue",
                r#"<script setup lang="ts">
defineProps<{ messages: string[] }>()
defineSlots<{ message(scope: { message: string; key: number }): unknown }>()
</script>
<template>
  <div v-for="(message, key) in messages" :key="key">
    <slot name="message" :message="message" :key="key" />
  </div>
</template>
"#,
            ),
            (
                "src/Inferred.vue",
                r#"<script setup lang="ts">
defineProps<{ messages: string[] }>()
</script>
<template>
  <div v-for="(message, key) in messages" :key="key">
    <slot name="message" :message="message" :key="key" />
  </div>
</template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Declared from './Declared.vue'
import Inferred from './Inferred.vue'
</script>
<template>
  <Declared :messages="['a']">
    <template #message="{ message, key }">{{ key.toFixed() }} {{ message }}</template>
  </Declared>
  <Inferred :messages="['b']">
    <template #message="{ message, key }">{{ key.toFixed() }} {{ message }}</template>
  </Inferred>
</template>
"#,
            ),
        ],
    );
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    assert_eq!(snapshot, Some(Vec::new()));
}

#[test]
fn slot_outlet_required_key_is_a_checked_prop() {
    // #7048: a required defineSlots `key` passed as `:key` must not be TS2741.
    // A slot type that omits `key` must not report that vnode key as TS2353.
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "slot-outlet-required-key",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineSlots<{ default(props: { key: number; label: string }): unknown }>()
const label = "item"
const key = 1
</script>
<template>
  <slot :key="key" :label="label" />
</template>
"#,
            ),
            (
                "src/Omitted.vue",
                r#"<script setup lang="ts">
defineSlots<{ default(props: { label: string }): unknown }>()
const label = "item"
const key = 1
</script>
<template>
  <slot :key="key" :label="label" />
</template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
import Omitted from './Omitted.vue'
</script>
<template>
  <Child v-slot="{ key, label }">{{ key.toFixed() }} {{ label }}</Child>
  <Omitted v-slot="{ label }">{{ label }}</Omitted>
</template>
"#,
            ),
        ],
    );
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };
    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/Child.vue"
                && *code == Some(2741)
                && (message.contains("'key'") || message.contains("\"key\"")))
        }),
        "required slot key passed on the outlet must not be TS2741: {snapshot:#?}"
    );
    assert!(
        snapshot
            .iter()
            .all(|(file, code, _)| { !(file == "src/Omitted.vue" && *code == Some(2353)) }),
        "vnode key on a slot that does not declare key must not be an excess property: {snapshot:#?}"
    );
}
