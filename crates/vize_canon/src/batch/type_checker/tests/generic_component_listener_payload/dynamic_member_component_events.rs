use super::*;

#[test]
fn infers_dynamic_member_component_listener_payload() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "dynamic-member-component-listener-payload",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
interface Payload {
  value: string;
}

defineEmits<{
  change: [payload: Payload];
}>();
</script>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Child from "./Child.vue";

const Components = { Child };

interface Payload {
  value: string;
}

function handleChange(payload: Payload) {
  payload.value.toUpperCase();
}
</script>

<template>
  <component :is="Components.Child" @change="handleChange($event)" />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "dynamic member component listener should infer child emit payload: {snapshot:#?}"
    );
    let _ = std::fs::remove_dir_all(&project_root);
}
