use super::*;

#[test]
fn infers_generic_callback_prop_through_listener_shorthand() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "issue-2807-generic-callback-listener-shorthand",
        &[
            (
                "src/Child.vue",
                r#"<script lang="ts">
export interface ChildProps<T> {
  items: T[];
  onChoose?: (item: T) => void;
}
</script>

<script setup lang="ts" generic="T">
defineProps<ChildProps<T>>();
</script>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Child from "./Child.vue";

interface Item {
  value: number;
}

const items: Item[] = [{ value: 1 }];

function onChoose(item: Item) {
  item.value.toFixed(0);
}
</script>

<template>
  <Child :items @choose="onChoose" />
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
        "listener shorthand must share generic inference with callback props: {snapshot:#?}"
    );
    let _ = std::fs::remove_dir_all(&project_root);
}
