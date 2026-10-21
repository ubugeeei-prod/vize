//! Adding a component listener must not change how its other attributes are
//! checked (#9852).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn listener_keeps_class_style_and_spread_attribute_parity() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "listener-attribute-parity",
        &[
            (
                "src/MyButton.vue",
                r#"<script setup lang="ts">
defineEmits<{ click: [event: MouseEvent] }>();
</script>
<template><button @click="$emit('click', $event)"><slot /></button></template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import { ref } from "vue";
import MyButton from "./MyButton.vue";
const active = ref(false);
const count = ref(0);
</script>
<template>
  <MyButton class="base" :class="{ active }" @click="count++">class</MyButton>
  <MyButton style="color: red" :style="{ fontWeight: 'bold' }" @click="count++">style</MyButton>
</template>
"#,
            ),
            (
                "src/Card.vue",
                r#"<script setup lang="ts">
defineProps<{ title?: string; body?: string }>();
defineEmits<{ "update:title": [value: string] }>();
</script>
<template><div>{{ title }} {{ body }}</div></template>
"#,
            ),
            (
                "src/Spread.vue",
                r#"<script setup lang="ts">
import { ref } from "vue";
import Card from "./Card.vue";
const item = ref({ id: 1, title: "t", body: "b" });
</script>
<template>
  <Card v-bind="item" />
  <Card v-bind="item" @update:title="item.title = $event" />
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
        snapshot.is_empty(),
        "listeners must not add diagnostics to accepted attributes: {snapshot:#?}"
    );
}
