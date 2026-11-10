//! Static and `v-if` component listeners keep their payload checks when their
//! inference-only emit-props call reads its literal as `any`.

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

const CHILD: &str = r#"<script setup lang="ts">
defineProps<{ label: string; count?: number }>();
defineEmits<{ select: [id: number, label: string]; "update:count": [value: number] }>();
</script>

<template>
  <button>{{ label }}</button>
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

const VALID: &str = r#"<script setup lang="ts">
import Child from "./Child.vue";
import GenericChild from "./GenericChild.vue";
const name = "a";
const flag = Math.random() > 0.5;
const onSelect = (id: number, label: string) => id + label.length;
</script>

<template>
  <Child :label="name" @select="(id, label) => id.toFixed() + label.length" />
  <Child v-if="flag" :label="name" @select="onSelect" @update:count="(value) => value.toFixed()" />
  <GenericChild :items="[1, 2]" @pick="(item) => item.toFixed()" />
  <GenericChild v-if="flag" :items="['a']" @pick="(item) => item.length" />
</template>
"#;

const WRONG: &str = r#"<script setup lang="ts">
import Child from "./Child.vue";
import GenericChild from "./GenericChild.vue";
const name = "a";
const flag = Math.random() > 0.5;
</script>

<template>
  <Child :label="name" @select="(id: string) => id" />
  <Child v-if="flag" :label="name" @select="(id: string) => id" />
  <Child v-if="flag && Child" :label="name" @update:count="(value: string) => value" />
  <GenericChild :items="[1, 2]" @pick="(item) => item.length" />
  <GenericChild v-if="flag" :items="['a']" @pick="(item: number) => item" />
</template>
"#;

#[test]
fn static_and_guarded_listeners_keep_payload_diagnostics() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = super::real_vue::create_project_case(
        "static-listener-emit-gate",
        &[
            ("src/Child.vue", CHILD),
            ("src/GenericChild.vue", GENERIC_CHILD),
            ("src/Valid.vue", VALID),
            ("src/Wrong.vue", WRONG),
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
    // Valid.vue stays clean; each wrong payload keeps its code, text and anchor.
    assert_eq!(
        snapshot,
        Some(vec![
            (
                "src/Wrong.vue".into(),
                Some(2322),
                "10:37:error Type '(id: string) => string' is not assignable to type '(id: number, label: string) => any'.\nTypes of parameters 'id' and 'id' are incompatible.\nType 'number' is not assignable to type 'string'.".into(),
            ),
            (
                "src/Wrong.vue".into(),
                Some(2322),
                "11:46:error Type '(value: string) => string' is not assignable to type '(value: number) => any'.\nTypes of parameters 'value' and 'value' are incompatible.\nType 'number' is not assignable to type 'string'.".into(),
            ),
            (
                "src/Wrong.vue".into(),
                Some(2322),
                "13:45:error Type '(item: number) => number' is not assignable to type '(item: string) => any'.\nTypes of parameters 'item' and 'item' are incompatible.\nType 'string' is not assignable to type 'number'.".into(),
            ),
            (
                "src/Wrong.vue".into(),
                Some(2322),
                "9:25:error Type '(id: string) => string' is not assignable to type '(id: number, label: string) => any'.\nTypes of parameters 'id' and 'id' are incompatible.\nType 'number' is not assignable to type 'string'.".into(),
            ),
            (
                "src/Wrong.vue".into(),
                Some(2339),
                "12:55:error Property 'length' does not exist on type 'number'.".into(),
            ),
        ])
    );
}
