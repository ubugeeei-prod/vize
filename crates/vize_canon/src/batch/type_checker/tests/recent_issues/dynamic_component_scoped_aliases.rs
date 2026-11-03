//! `<component :is>` keeps its component target when its key or guard reads a
//! name bound by an enclosing slot or loop (#11491).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_carton::{String, cstr};

const PICKER: &str = r#"<script setup lang="ts">
defineProps<{ x?: number }>();
const emit = defineEmits<{ pick: [value: string]; select: [value: string] }>();
</script>

<template>
  <button @click="emit('pick', 'a')" @focus="emit('select', 'b')" />
</template>
"#;

const PANEL: &str = r#"<script setup lang="ts">
import Picker from "./Picker.vue";
defineSlots<{
  qux: (props: { kind: "a" | "b"; open: boolean; comp: typeof Picker }) => unknown;
}>();
</script>

<template>
  <slot name="qux" kind="a" :open="true" :comp="Picker" />
</template>
"#;

// Loosely typed targets, as `<RouterView v-slot="{ Component }">` hands out.
const LOOSE: &str = r#"<script setup lang="ts">
import type { Component, VNode } from "vue";
defineSlots<{ default: (props: { comp: Component; node: VNode; tag: "a" | "button" }) => unknown }>();
</script>

<template>
  <slot :comp="{}" :node="undefined as unknown as VNode" tag="a" />
</template>
"#;

const SETUP: &str = r#"<script setup lang="ts">
import Picker from "./Picker.vue";
import Panel from "./Panel.vue";
import Loose from "./Loose.vue";

const map = { a: Picker, b: Picker };
const foo = "a" as const;
const items = [{ kind: "a" as const, open: true, keys: ["b" as const], comp: Picker }];
const rows = [[{ kind: "b" as const }]];
const take = (value: string) => value;
const count = (value: number) => value;
void count;
</script>
"#;

const CLEAN: &str = r#"
<template>
  <Panel v-slot:qux="{ kind, open }">
    <component :is="map[kind]" @pick="(v) => take(v)" @select="(v) => take(v)" />
    <component v-if="open" :is="map[foo]" @pick="(v) => take(v)" @select="(v) => take(v)" />
  </Panel>
  <component v-for="item in items" :key="item.kind" :is="map[item.kind]" @pick="(v) => take(v)" @select="(v) => take(v)" />
  <Panel>
    <template #qux="{ kind: k, open }">
      <div v-if="open">
        <component :is="map[k]" :x="1" @pick="(v) => take(v)" />
      </div>
      <component v-for="n in 2" :key="n" :is="map[k]" @pick="(v) => take(v)" />
    </template>
  </Panel>
  <template v-for="(item, index) in items" :key="index">
    <component v-if="item.open" :is="map[item.keys[index]]" @pick="(v) => take(v)" />
    <Panel>
      <template #qux="{ open }">
        <component v-if="open" :is="map[item.kind]" @pick="(v) => take(v)" />
      </template>
    </Panel>
  </template>
  <div v-for="(row, r) in rows" :key="r">
    <component v-for="{ kind } in row" :key="kind" :is="map[kind]" @pick="(v) => take(v)" />
  </div>
  <Panel #qux="{ kind }">
    <Panel #qux="{ open }">
      <component v-if="open" :is="map[kind]" @pick="(v) => take(v)" />
    </Panel>
  </Panel>
  <Panel #qux="{ comp }">
    <component :is="comp" @pick="(v) => take(v)" />
  </Panel>
  <component v-for="item in items" :key="item.kind" :is="item.comp" @pick="(v) => take(v)" />
  <Loose v-slot="{ comp, node, tag }">
    <component :is="comp" foo="1" />
    <component :is="node" class="page" @click="() => {}" />
    <component :is="tag" href="/x" />
  </Loose>
</template>
"#;

const ERRORS: &str = r#"
<template>
  <Panel v-slot:qux="{ kind, open }">
    <component :is="map[kind]" @pick="(v) => count(v)" />
    <component v-if="open" :is="map[foo]" @pick="(v) => v.toFixed()" />
  </Panel>
  <component v-for="item in items" :key="item.kind" :is="map[item.kind]" @pick="(v: number) => v" />
  <Panel #qux="{ kind }">
    <component :is="map[kind]" :x="'one'" />
  </Panel>
  <Panel #qux="{ comp }">
    <component :is="comp" @pick="(v) => count(v)" />
  </Panel>
  <component v-for="item in items" :key="item.kind" :is="item.comp" @pick="(v) => v.toFixed()" />
</template>
"#;

#[test]
fn dynamic_component_targets_follow_the_scope_that_binds_their_names() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let clean = cstr!("{SETUP}{CLEAN}");
    let errors = cstr!("{SETUP}{ERRORS}");
    let project_root = super::real_vue::create_project_case(
        "dynamic-component-scoped-aliases",
        &[
            ("src/Picker.vue", PICKER),
            ("src/Panel.vue", PANEL),
            ("src/Loose.vue", LOOSE),
            ("src/Clean.vue", clean.as_str()),
            ("src/Errors.vue", errors.as_str()),
        ],
    );
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "module": "ESNext",
    "moduleResolution": "bundler"
  },
  "include": ["src/**/*.vue"]
}"#,
    )
    .unwrap();

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    // The same codes and positions as vue-tsc 3.3.11; Clean.vue reports none.
    let wrong_listener = "Type '(v: number) => number' is not assignable to type '(value: string) => any'.\nTypes of parameters 'v' and 'value' are incompatible.\nType 'string' is not assignable to type 'number'.";
    let expected = [
        (
            2322,
            cstr!("20:75:error {wrong_listener}"),
        ),
        (
            2322,
            cstr!("22:33:error Type 'string' is not assignable to type 'number'."),
        ),
        (
            2345,
            cstr!("17:52:error Argument of type 'string' is not assignable to parameter of type 'number'."),
        ),
        (
            2345,
            cstr!("25:47:error Argument of type 'string' is not assignable to parameter of type 'number'."),
        ),
        (
            2551,
            cstr!("18:59:error Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?"),
        ),
        (
            2551,
            cstr!("27:85:error Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?"),
        ),
    ]
    .map(|(code, message)| (String::from("src/Errors.vue"), Some(code), message));
    assert_eq!(snapshot, Some(expected.to_vec()));
}
