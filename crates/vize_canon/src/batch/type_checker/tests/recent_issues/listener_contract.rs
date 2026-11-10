//! Every listener form keeps its payload diagnostics when the per-event alias
//! chain is one shared `$ListenerContract` reference.

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

const CHILD: &str = r#"<script setup lang="ts">
defineProps<{
  label: string;
  onPress?: (id: number) => number;
  onTap?: ((value: string) => void) | ((value: string) => void)[];
}>();
defineEmits<{
  select: [id: number, label: string];
  "row-click": [row: { id: number }];
  "update:count": [value: number];
  ping: [];
}>();
</script>

<template>
  <button>{{ label }}</button>
</template>
"#;

const OVERLOAD_CHILD: &str = r#"<script setup lang="ts">
defineEmits<{ (event: "save", value: string): void; (event: "close"): void }>();
</script>

<template>
  <div />
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

const ROOT_CHILD: &str = r#"<script setup lang="ts">
defineProps<{ name?: string }>();
</script>

<template>
  <input />
</template>
"#;

const LOOSE_CHILD: &str = r#"<script setup lang="ts">
defineEmits(["loose"]);
</script>

<template>
  <span />
</template>
"#;

const VALID: &str = r#"<script setup lang="ts">
import Child from "./Child.vue";
import OverloadChild from "./OverloadChild.vue";
import GenericChild from "./GenericChild.vue";
import RootChild from "./RootChild.vue";
import LooseChild from "./LooseChild.vue";
const flag = Math.random() > 0.5;
const onSelect = (id: number, label: string) => id + label.length;
const taps = [(value: string) => value.length, (value: string) => value.trim()];
</script>

<template>
  <Child label="a" @select="onSelect" @row-click="(row) => row.id.toFixed()" />
  <Child label="a" @rowClick="(row) => row.id" @update:count="(value) => value.toFixed()" />
  <Child label="a" @ping="flag = !flag" @press="(id) => id * 2" @tap="taps" />
  <Child v-if="flag" label="a" @select="(id, label) => label.repeat(id)" @tap="(value) => value.length" />
  <OverloadChild @save="(value) => value.toUpperCase()" @close="() => undefined" />
  <GenericChild :items="[1, 2]" @pick="(item) => item.toFixed()" />
  <RootChild name="n" @input="(event) => event.target" @focus="(event) => event.relatedTarget" />
  <LooseChild @loose="(...args) => args.length" />
  <component :is="flag ? Child : OverloadChild" label="a" @select="onSelect" />
</template>
"#;

const WRONG: &str = r#"<script setup lang="ts">
import Child from "./Child.vue";
import OverloadChild from "./OverloadChild.vue";
import GenericChild from "./GenericChild.vue";
import RootChild from "./RootChild.vue";
const flag = Math.random() > 0.5;
</script>

<template>
  <Child label="a" @select="(id: string) => id" />
  <Child label="a" @row-click="(row) => row.missing" />
  <Child label="a" @update:count="(value: string) => value" />
  <Child label="a" @press="(id) => String(id)" />
  <Child label="a" @tap="[(value: number) => value]" />
  <Child v-if="flag" label="a" @select="(id) => id.length" />
  <OverloadChild @save="(value: number) => value" />
  <GenericChild :items="['a']" @pick="(item: number) => item" />
  <RootChild @input="(event: string) => event" />
  <Child label="a" @ping="(value: string) => value" />
</template>
"#;

#[test]
fn shared_listener_contracts_keep_every_payload_diagnostic() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = super::real_vue::create_project_case(
        "listener-contract",
        &[
            ("src/Child.vue", CHILD),
            ("src/OverloadChild.vue", OVERLOAD_CHILD),
            ("src/GenericChild.vue", GENERIC_CHILD),
            ("src/RootChild.vue", ROOT_CHILD),
            ("src/LooseChild.vue", LOOSE_CHILD),
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
    // Valid.vue stays clean; each wrong payload keeps its code, text and anchor,
    // exactly as the per-event alias chain reported them.
    assert_eq!(
        snapshot,
        Some(vec![
            (
                "src/Wrong.vue".into(),
                Some(2322),
                "10:21:error Type '(id: string) => string' is not assignable to type '(id: number, label: string) => any'.\nTypes of parameters 'id' and 'id' are incompatible.\nType 'number' is not assignable to type 'string'.".into(),
            ),
            (
                "src/Wrong.vue".into(),
                Some(2322),
                "12:21:error Type '(value: string) => string' is not assignable to type '(value: number) => any'.\nTypes of parameters 'value' and 'value' are incompatible.\nType 'number' is not assignable to type 'string'.".into(),
            ),
            (
                "src/Wrong.vue".into(),
                Some(2322),
                "13:21:error Type '(id: number) => string' is not assignable to type '(id: number) => number'.\nType 'string' is not assignable to type 'number'.".into(),
            ),
            (
                "src/Wrong.vue".into(),
                Some(2322),
                "16:19:error Type '(value: number) => number' is not assignable to type '(value: string) => any'.\nTypes of parameters 'value' and 'value' are incompatible.\nType 'string' is not assignable to type 'number'.".into(),
            ),
            (
                "src/Wrong.vue".into(),
                Some(2322),
                "17:33:error Type '(item: number) => number' is not assignable to type '(item: string) => any'.\nTypes of parameters 'item' and 'item' are incompatible.\nType 'string' is not assignable to type 'number'.".into(),
            ),
            (
                "src/Wrong.vue".into(),
                Some(2322),
                "19:21:error Type '(value: string) => string' is not assignable to type '() => any'.\nTarget signature provides too few arguments. Expected 1 or more, but got 0.".into(),
            ),
            (
                "src/Wrong.vue".into(),
                Some(2339),
                "11:45:error Property 'missing' does not exist on type '{ id: number; }'.".into(),
            ),
            (
                "src/Wrong.vue".into(),
                Some(2339),
                "15:52:error Property 'length' does not exist on type 'number'.".into(),
            ),
        ])
    );
}
