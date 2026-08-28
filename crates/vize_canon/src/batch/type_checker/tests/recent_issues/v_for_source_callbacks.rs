//! Diagnostics inside v-for source expressions must map back to authored bytes (#3756).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_s0::String;

#[test]
fn v_for_source_callbacks_report_implicit_any_at_the_authored_parameter() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "v-for-source-callback-diagnostics",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const items: any = []
</script>

<template>
  <div v-for="item in items.filter(value => value)" :key="item" />
</template>
"#,
        )],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    assert_eq!(
        snapshot,
        // vue-tsc 3.3.4 with TypeScript 6.0.3, on the byte-identical fixture:
        // src/App.vue(6,36): error TS7006: Parameter 'value' implicitly has an 'any' type.
        Some(vec![(
            String::from("src/App.vue"),
            Some(7006),
            String::from("6:36:error Parameter 'value' implicitly has an 'any' type."),
        )]),
    );
}

#[test]
fn component_v_for_source_diagnostics_are_mapped_once() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "component-v-for-source-callback-diagnostics",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ value: unknown }>()
</script>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
const items: any = []
</script>

<template>
  <Child v-for="item in items.filter(value => value)" :key="item" :value="item" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    assert_eq!(
        snapshot,
        Some(vec![(
            String::from("src/App.vue"),
            Some(7006),
            String::from("7:38:error Parameter 'value' implicitly has an 'any' type."),
        )]),
    );
}

#[test]
fn generic_slot_v_for_source_does_not_expose_a_contextual_typing_gap() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-slot-v-for-source-callback",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="T">
defineProps<{ items: T[] }>()
defineSlots<{ default(props: { items: T[] }): unknown }>()
</script>
<template><slot :items="items" /></template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
const items = [{ id: 1 }]
</script>
<template>
  <Child v-slot="{ items: pageItems }" :items="items">
    <div v-for="item in pageItems.map(value => value)" :key="item.id" />
  </Child>
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    // vue-tsc 3.3.4 contextually types `value` through the generic slot. Until
    // Vize resolves those generics, do not surface a Vize-only TS7006 (#3756).
    assert_eq!(snapshot, Some(Vec::new()));
}

#[test]
fn v_for_source_resolves_template_only_props_before_mapping_diagnostics() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "v-for-source-template-prop",
        &[
            (
                "src/contracts.ts",
                r#"export interface DialogProps {
  messages: Array<{ text: string }>
}
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import type { DialogProps } from './contracts'
defineProps<DialogProps>()
</script>
<template>
  <section v-for="message in messages.slice(0, Math.max(0, messages.length))" :key="message.text">
    <span v-for="word in message.text.split(' ')" :key="word">{{ word }}</span>
  </section>
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    assert_eq!(snapshot, Some(Vec::new()));
}
