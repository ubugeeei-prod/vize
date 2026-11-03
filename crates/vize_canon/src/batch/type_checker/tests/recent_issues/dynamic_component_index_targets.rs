//! `<component :is="map[key]">` keeps its component target for generic keys
//! (#11448) and inside slots under script-owned guards or member keys (#11449).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_carton::{String, cstr};

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
    let generic_clean = cstr!("{GENERIC_SETUP}{GENERIC_CLEAN}");
    let generic_errors = cstr!("{GENERIC_SETUP}{GENERIC_ERRORS}");
    let slot_clean = cstr!("{SLOT_SETUP}{SLOT_CLEAN}");
    let slot_errors = cstr!("{SLOT_SETUP}{SLOT_ERRORS}");
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
                (file, code, String::from(message))
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
            String::from(file),
            Some(code),
            String::from(message),
        )
    });
    assert_eq!(snapshot, Some(expected.to_vec()));
}

const REPORTS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/"
);

fn check_report(name: &str, dir: &str, probe: &str) -> Option<Vec<(String, Option<u32>, String)>> {
    let read =
        |file: &str| std::fs::read_to_string(cstr!("{REPORTS}{dir}/{file}.txt").as_str()).unwrap();
    let (bar, baz) = (read("Bar.vue"), read("Baz.vue"));
    let project_root = super::real_vue::create_project_case(
        name,
        &[
            ("src/Bar.vue", bar.as_str()),
            ("src/Baz.vue", baz.as_str()),
            ("src/Probe.vue", probe),
        ],
    );
    std::fs::write(
        project_root.join("tsconfig.json"),
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
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    snapshot
}

#[test]
fn reported_inputs_keep_their_component_targets() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let read = |dir: &str| {
        std::fs::read_to_string(cstr!("{REPORTS}{dir}/Probe.vue.txt").as_str()).unwrap()
    };
    let generic = read("dynamic-generic-key-11448");
    let slot = read("dynamic-slot-guard-11449");
    let replace = |source: &str, from: &str, to: &str| {
        assert_eq!(source.matches(from).count(), 1, "{from}");
        source.replacen(from, to, 1)
    };
    let missing_key = replace(&generic, "{ a: Bar, b: Baz }", "{ a: Bar }");
    let wrong_payload = replace(&slot, "value.toFixed()", "value.nope()");

    let first = |snapshot: Option<Vec<(String, Option<u32>, String)>>| {
        snapshot.map(|diagnostics| {
            diagnostics
                .into_iter()
                .map(|(file, code, message)| {
                    let message = message.split(" cannot be used").next().unwrap_or("");
                    (file, code, String::from(message))
                })
                .collect::<Vec<_>>()
        })
    };
    let probe = |message: &str, code: u32| {
        Some(vec![(
            String::from("src/Probe.vue"),
            Some(code),
            String::from(message),
        )])
    };
    let dir = "dynamic-generic-key-11448";
    assert_eq!(
        check_report("reported-11448", dir, &generic),
        Some(Vec::new())
    );
    assert_eq!(
        first(check_report(
            "reported-11448-missing-key",
            dir,
            &missing_key
        )),
        probe("10:19:error Type 'TFoo'", 2536)
    );
    let dir = "dynamic-slot-guard-11449";
    assert_eq!(check_report("reported-11449", dir, &slot), Some(Vec::new()));
    assert_eq!(
        check_report("reported-11449-wrong-payload", dir, &wrong_payload),
        probe(
            "13:74:error Property 'nope' does not exist on type 'number'.",
            2339
        )
    );
}
