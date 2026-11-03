//! `<component :is="map[key]">` keeps its component target for generic keys
//! (#11448) and inside slots under script-owned guards or member keys (#11449).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

const EMPTY: &str = "<template>\n  <div />\n</template>\n";

const LEFT: &str = r#"<script setup lang="ts">
defineProps<{ x: number }>();
const emit = defineEmits<{ select: [value: number] }>();
</script>

<template>
  <button @click="emit('select', 1)"><slot /></button>
</template>
"#;

const RIGHT: &str = r#"<script setup lang="ts">
defineProps<{ x: number; y?: string }>();
const emit = defineEmits<{ select: [value: number] }>();
</script>

<template>
  <button @click="emit('select', 2)"><slot /></button>
</template>
"#;

const EMITTER: &str = r#"<script setup lang="ts">
const emit = defineEmits<{ select: [value: number] }>();
</script>

<template>
  <button @click="emit('select', 1)" />
</template>
"#;

const PANEL: &str = r#"<script setup lang="ts">
defineSlots<{ qux: () => unknown }>();
</script>

<template>
  <slot name="qux" />
</template>
"#;

const GENERIC_SETUP: &str = r#"<script setup lang="ts" generic="TFoo extends 'a' | 'b'">
import { defineAsyncComponent } from "vue";
import Bar from "./Bar.vue";
import Baz from "./Baz.vue";
import Left from "./Left.vue";
import Right from "./Right.vue";
import Panel from "./Panel.vue";

const props = defineProps<{ foo: TFoo; item: { type: TFoo }; shown: boolean }>();
const plain = { a: Bar, b: Baz };
const typed = { a: Left, b: Right };
const wide = { a: Left, b: Right, c: Bar };
const tags = { a: "div", b: "span" } as const;
const lazy = {
  a: defineAsyncComponent(() => import("./Bar.vue")),
  b: defineAsyncComponent(() => import("./Baz.vue")),
};
void props;
</script>
"#;

const GENERIC_CLEAN: &str = r#"
<template>
  <component :is="plain[foo]" />
  <component :is="plain[item.type]" />
  <component :is="tags[foo]" />
  <component :is="lazy[foo]" />
  <component :is="typed[props.foo]" :x="1" @select="(value) => value.toFixed()" />
  <component :is="wide[foo]" :x="1" />
  <component :is="shown ? typed[foo] : Left" :x="1" />
  <component :is="typed[foo]" v-if="shown" :x="1" />
  <component :is="typed[foo]" v-for="n in 2" :key="n" :x="n" @select="(value) => value.toFixed()" />
  <Panel>
    <template #qux>
      <component :is="typed[item.type]" v-if="shown" :x="1" @select="(value) => value.toFixed()" />
    </template>
  </Panel>
</template>
"#;

const GENERIC_ERRORS: &str = r#"
<template>
  <component :is="typed[foo]" :x="'one'" />
  <component :is="typed[foo]" :x="1" @select="(value) => value.toUpperCase()" />
  <component :is="typed[foo]" />
</template>
"#;

const SLOT_SETUP: &str = r#"<script setup lang="ts">
import Emitter from "./Emitter.vue";
import Panel from "./Panel.vue";

const map = { foo: Emitter };
const key = "foo";
const isShown = true;
const qux = { key: "foo" as const };
</script>
"#;

const SLOT_CLEAN: &str = r#"
<template>
  <Panel>
    <template #qux>
      <component :is="map[key]" v-if="isShown" @select="(value) => value.toFixed()" />
      <div v-if="!isShown" />
      <component :is="map[key]" v-else-if="isShown" @select="(value) => value.toFixed()" />
      <component :is="map[key]" v-else @select="(value) => value.toFixed()" />
      <template v-if="isShown">
        <component :is="map[qux.key]" @select="(value) => value.toFixed()" />
      </template>
      <component :is="map[key]" v-show="isShown" @select="(value) => value.toFixed()" />
      <component :is="map[key]" v-for="n in 2" :key="n" @select="(value) => value.toFixed()" />
      <Panel v-if="isShown">
        <template #qux>
          <component :is="map[qux.key]" v-if="isShown" @select="(value) => value.toFixed()" />
        </template>
      </Panel>
    </template>
  </Panel>
</template>
"#;

const SLOT_ERRORS: &str = r#"
<template>
  <Panel>
    <template #qux>
      <component :is="map[key]" v-if="isShown" @select="(value) => value.toUpperCase()" />
      <component :is="map[qux.key]" v-if="isShown" @select="(value: string) => value" />
    </template>
  </Panel>
</template>
"#;

#[test]
fn dynamic_component_index_targets_keep_props_slots_and_payloads() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let generic_clean = vize_carton::cstr!("{GENERIC_SETUP}{GENERIC_CLEAN}");
    let generic_errors = vize_carton::cstr!("{GENERIC_SETUP}{GENERIC_ERRORS}");
    let slot_clean = vize_carton::cstr!("{SLOT_SETUP}{SLOT_CLEAN}");
    let slot_errors = vize_carton::cstr!("{SLOT_SETUP}{SLOT_ERRORS}");
    let project_root = super::real_vue::create_project_case(
        "dynamic-component-index-targets",
        &[
            ("src/Bar.vue", EMPTY),
            ("src/Baz.vue", EMPTY),
            ("src/Left.vue", LEFT),
            ("src/Right.vue", RIGHT),
            ("src/Emitter.vue", EMITTER),
            ("src/Panel.vue", PANEL),
            ("src/GenericClean.vue", generic_clean.as_str()),
            ("src/GenericErrors.vue", generic_errors.as_str()),
            ("src/SlotClean.vue", slot_clean.as_str()),
            ("src/SlotErrors.vue", slot_errors.as_str()),
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

    // The missing-prop control names a generated alias; keep its stable prefix.
    let snapshot = snapshot.map(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|(file, code, message)| {
                let message = message.split(" parameter of type").next().unwrap_or("");
                (file, code, vize_carton::String::from(message))
            })
            .collect::<Vec<_>>()
    });
    let expected = [
        (
            "src/GenericErrors.vue",
            2322,
            "22:32:error Type 'string' is not assignable to type 'number'.",
        ),
        (
            "src/GenericErrors.vue",
            2339,
            "23:64:error Property 'toUpperCase' does not exist on type 'number'.",
        ),
        (
            "src/GenericErrors.vue",
            2345,
            "24:4:error Argument of type '{}' is not assignable to",
        ),
        (
            "src/SlotErrors.vue",
            2322,
            "15:53:error Type '(value: string) => string' is not assignable to type '(value: number) => any'.\nTypes of parameters 'value' and 'value' are incompatible.\nType 'number' is not assignable to type 'string'.",
        ),
        (
            "src/SlotErrors.vue",
            2339,
            "14:74:error Property 'toUpperCase' does not exist on type 'number'.",
        ),
    ]
    .map(|(file, code, message)| {
        (
            vize_carton::String::from(file),
            Some(code),
            vize_carton::String::from(message),
        )
    });
    assert_eq!(snapshot, Some(expected.to_vec()));
}
