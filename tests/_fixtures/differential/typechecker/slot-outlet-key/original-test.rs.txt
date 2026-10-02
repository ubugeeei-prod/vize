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
