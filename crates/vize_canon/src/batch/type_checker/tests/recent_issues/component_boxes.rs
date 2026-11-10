//! Listener payloads keep their diagnostics when each use's component boxes
//! and emit gate name the shared SDK helpers: union and `any` bindings, a
//! native root, a generic child, and gated static and `v-if` uses.

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

const CHILD: &str = r#"<script setup lang="ts">
defineProps<{ label: string; onPress?: (id: number) => number }>();
defineEmits<{ select: [id: number, label: string]; "row-click": [row: { id: number }] }>();
</script>

<template>
  <button>{{ label }}</button>
</template>
"#;

const ROOT_CHILD: &str = r#"<script setup lang="ts">
defineProps<{ name?: string }>();
</script>

<template>
  <input />
</template>
"#;

const GENERIC_CHILD: &str = r#"<script setup lang="ts" generic="T">
defineProps<{ items: T[] }>();
defineEmits<{ pick: [item: T] }>();
</script>

<template>
  <ul />
</template>
"#;

const PARENT: &str = r#"<script setup lang="ts">
import Child from "./Child.vue";
import RootChild from "./RootChild.vue";
import GenericChild from "./GenericChild.vue";
const flag = Math.random() > 0.5;
const Either = flag ? Child : RootChild;
const Untyped: any = Child;
</script>

<template>
  <Child label="a" @select="(id: string) => id" />
  <Child v-if="flag" label="a" @row-click="(row) => row.missing" />
  <Child label="a" @press="(id) => String(id)" />
  <Either label="a" @select="(id: string) => id" />
  <Either name="n" @focus="(event: string) => event" />
  <Untyped label="a" @select="(id: string) => id" />
  <RootChild name="n" @focus="(event: string) => event" />
  <GenericChild :items="['a']" @pick="(item: number) => item" />
</template>
"#;

#[test]
fn shared_component_boxes_keep_every_listener_diagnostic() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = super::real_vue::create_project_case(
        "component-boxes",
        &[
            ("src/Child.vue", CHILD),
            ("src/RootChild.vue", ROOT_CHILD),
            ("src/GenericChild.vue", GENERIC_CHILD),
            ("src/Parent.vue", PARENT),
        ],
    );
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ESNext"
  },
  "include": ["src/**/*.vue"]
}"#,
    )
    .unwrap();
    let snapshot = snapshot_project_diagnostics(&root);
    let _ = std::fs::remove_dir_all(&root);
    // Recorded with the inline boxes and per-module gate aliases. The union
    // binding's `@focus` reads its members' shared native root through their
    // string index; a distributive `keyof` would drop that diagnostic.
    assert_eq!(
        snapshot,
        Some(vec![
            (
                "src/Parent.vue".into(),
                Some(2322),
                "11:21:error Type '(id: string) => string' is not assignable to type '(id: number, label: string) => any'.\nTypes of parameters 'id' and 'id' are incompatible.\nType 'number' is not assignable to type 'string'.".into(),
            ),
            (
                "src/Parent.vue".into(),
                Some(2322),
                "13:21:error Type '(id: number) => string' is not assignable to type '(id: number) => number'.\nType 'string' is not assignable to type 'number'.".into(),
            ),
            (
                "src/Parent.vue".into(),
                Some(2322),
                "15:21:error Type '(event: string) => string' is not assignable to type '(payload: FocusEvent) => void'.\nTypes of parameters 'event' and 'payload' are incompatible.\nType 'FocusEvent' is not assignable to type 'string'.".into(),
            ),
            (
                "src/Parent.vue".into(),
                Some(2322),
                "18:33:error Type '(item: number) => number' is not assignable to type '(item: string) => any'.\nTypes of parameters 'item' and 'item' are incompatible.\nType 'string' is not assignable to type 'number'.".into(),
            ),
            (
                "src/Parent.vue".into(),
                Some(2339),
                "12:57:error Property 'missing' does not exist on type '{ id: number; }'.".into(),
            ),
        ])
    );
}
