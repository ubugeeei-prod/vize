//! The committed TS-34 battery: one SFC per rule family, each rule of the
//! `Bindings` spec and each clause of the `UndefinedRefs` rule exercised.

pub const SOURCES: &[(&str, &str)] = &[
    ("imports-and-declarations", IMPORTS),
    ("variable-initializers", INITIALIZERS),
    ("macros", MACROS),
    ("props-destructure", PROPS_DESTRUCTURE),
    ("reactivity-and-aliases", REACTIVITY),
    ("patterns", PATTERNS),
    ("authored-owns-props", AUTHORED),
    ("template-reads", TEMPLATE_READS),
    ("template-only", TEMPLATE_ONLY),
];

const IMPORTS: &str = r#"<script setup lang="ts">
import Default from "./Default.vue";
import * as Namespace from "./namespace";
import { named, other as renamed } from "./named";
import type { Only } from "./types";
import { type Inline, value } from "./mixed";
import { ref } from "vue";
function helper() {}
class Model {}
enum Color { Red }
const enum Erased { A }
declare enum Declared { B }
declare function ambient(): void;
</script>
<template><p>{{ named }} {{ renamed }} {{ value }} {{ Color.Red }}</p></template>
"#;

const INITIALIZERS: &str = r#"<script setup lang="ts">
const text = "a";
const count = 1;
const negative = -1;
const plain = `plain`;
const templated = `${text}!`;
const nothing = null;
const arrow = () => count;
const fn = function () {};
const object = { a: 1 };
const list = [1, 2];
const called = compute();
const member = api.call();
const wrapped = (compute() as number)!;
const awaited = await compute();
const alias = text;
let mutable = 1;
var legacy;
declare const ambient: number;
</script>
<template><p>{{ text }}{{ called }}{{ mutable }}</p></template>
"#;

const MACROS: &str = r#"<script setup lang="ts">
const props = defineProps<{ title: string; count?: number }>();
const emit = defineEmits<{ (e: "change"): void }>();
const model = defineModel<string>();
let named = defineModel("named");
defineExpose({ props });
defineOptions({ name: "Macros" });
const slots = defineSlots<{ default(): unknown }>();
</script>
<template><p @click="emit('change')">{{ title }} {{ model }} {{ named }}</p></template>
"#;

const PROPS_DESTRUCTURE: &str = r#"<script setup lang="ts">
const { a, b: bee = 1, nested: { deep }, ...rest } = withDefaults(defineProps<{
  a: string;
  b?: number;
  nested: { deep: string };
  c?: boolean;
}>(), { b: 1 });
</script>
<template><p>{{ a }} {{ bee }} {{ rest }} {{ c }} {{ deep }}</p></template>
"#;

const REACTIVITY: &str = r#"<script setup lang="ts">
import { ref, computed, reactive, readonly, toRefs, shallowRef, useTemplateRef } from "vue";
const r = ref;
const count = ref(0);
const aliased = r(1);
const doubled = computed(() => count.value * 2);
const state = reactive({ n: 1 });
const frozen = readonly(state);
const refs = toRefs(state);
let shallow = shallowRef(1);
const el = useTemplateRef("el");
const notReactive = watch;
</script>
<template><p ref="el">{{ count }} {{ aliased }} {{ doubled }} {{ state.n }}</p></template>
"#;

const PATTERNS: &str = r#"<script setup lang="ts">
const { x, y: why } = useFoo();
const [first, , third] = useBar();
const { f } = () => ({ f: 1 });
let [m, ...others] = list;
const [model, modifiers] = defineModel();
let { deep: { inner } } = source;
</script>
<template><p>{{ x }} {{ why }} {{ first }} {{ model }} {{ inner }}</p></template>
"#;

const AUTHORED: &str = r#"<script setup lang="ts">
const early = 1;
defineProps(["early", "late", "only"]);
const late = ref(2);
</script>
<template><p>{{ early }} {{ late }} {{ only }}</p></template>
"#;

const TEMPLATE_READS: &str = r#"<script setup lang="ts">
import { ref } from "vue";
import type { Shape } from "./types";
const items = ref([{ id: 1, label: "a" }]);
const handle = (_event: unknown) => {};
</script>
<template>
  <ul>
    <li v-for="(item, index) in items" :key="item.id" :title="missing + item.label">
      {{ index }} {{ item.label }} {{ ghost }} {{ ghost }}
    </li>
  </ul>
  <Comp v-slot="{ row }">{{ row.id }} {{ absent.row }}</Comp>
  <button @click="handle($event); undefinedHandler()">{{ Math.max(1, 2) }}</button>
  <p :class="{ active: isActive }">{{ items.length }} {{ $attrs.id }} {{ Shape }}</p>
  <p v-if="flag">{{ a.b.c }} {{ nothere . prop }}</p>
</template>
"#;

const TEMPLATE_ONLY: &str = r#"<template>
  <div :id="unknown">{{ alsoUnknown }}</div>
</template>
"#;
