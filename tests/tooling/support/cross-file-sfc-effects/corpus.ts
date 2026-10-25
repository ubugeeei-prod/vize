export interface EffectCase {
  id: string;
  files: Record<string, string>;
  child: string | null;
  graph: { nodes: number; edges: number; cycles: number; score: number };
  total: number;
  band: "low" | "moderate";
  wasmComparable: boolean;
  blocks: number;
  diagnostics: boolean;
}

const app = `<script setup lang="ts">
import Child from './Child.vue'
</script>
<template><Child /></template>
`;
const child = `<script setup lang="ts">
import { computed, ref } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
</script>
<template><p>{{ doubled }}</p></template>
`;
const graph = { nodes: 2, edges: 1, cycles: 0, score: 4 };
const primary: EffectCase = {
  id: "setup-lf",
  files: { "App.vue": app, "Child.vue": child },
  child: "Child.vue",
  graph,
  total: 6,
  band: "low",
  wasmComparable: true,
  blocks: 2,
  diagnostics: false,
};

export const cases: EffectCase[] = [
  primary,
  {
    ...primary,
    id: "setup-crlf",
    files: Object.fromEntries(
      Object.entries(primary.files).map(([name, text]) => [name, text.replaceAll("\n", "\r\n")]),
    ),
  },
  {
    ...primary,
    id: "normal-and-setup",
    blocks: 3,
    files: {
      "App.vue": app,
      "Child.vue": `<script>
import { ref } from 'vue'
const count = ref(0)
export default {}
</script>
<script setup lang="ts">
import { computed } from 'vue'
const doubled = computed(() => count.value * 2)
</script>
<template><p>{{ doubled }}</p></template>
`,
    },
  },
  {
    ...primary,
    id: "cycle",
    graph: { nodes: 2, edges: 2, cycles: 1, score: 16 },
    total: 18,
    band: "moderate",
    files: {
      "App.vue": app,
      "Child.vue": `<script setup lang="ts">
import { computed } from 'vue'
const left = computed(() => right.value)
const right = computed(() => left.value)
</script>
<template><p>{{ left }}</p></template>
`,
    },
  },
  {
    ...primary,
    id: "diagnostic-unicode-crlf",
    diagnostics: true,
    files: {
      "App.vue": app
        .replace(
          "<template><Child /></template>",
          '<template><div title="日本語🙂" id="shared"><Child /></div></template>',
        )
        .replaceAll("\n", "\r\n"),
      "Child.vue": child
        .replace("const count", "// 日本語🙂\nconst count")
        .replace("<p>", '<p id="shared">')
        .replaceAll("\n", "\r\n"),
    },
  },
  {
    ...primary,
    id: "jsx-normal",
    wasmComparable: false,
    files: {
      "App.vue": app,
      "Child.vue": `<script lang="jsx">
import { computed, ref } from 'vue'
const view = <div>日本語🙂</div>
const count = ref(0)
const doubled = computed(() => count.value * 2)
export default {}
</script>
<template><p>{{ doubled }}</p></template>
`,
    },
  },
  {
    ...primary,
    id: "tsx-setup",
    wasmComparable: false,
    files: {
      "App.vue": app,
      "Child.vue": child
        .replace('lang="ts"', 'lang="tsx"')
        .replace(
          "const count = ref(0)",
          "const view = <div>日本語🙂</div>\nconst count = ref<number>(0)",
        ),
    },
  },
  {
    ...primary,
    id: "dependency-edit",
    graph: { nodes: 0, edges: 0, cycles: 0, score: 0 },
    total: 2,
    files: {
      "App.vue": app,
      "Child.vue": `<script setup>const label = '日本語🙂'</script>
<template><p>{{ label }}</p></template>
`,
    },
  },
  {
    ...primary,
    id: "script-delete",
    graph: { nodes: 0, edges: 0, cycles: 0, score: 0 },
    total: 2,
    blocks: 1,
    files: {
      "App.vue": app,
      "Child.vue": "<template><p>ref(0), computed(() => count.value)</p></template>\n",
    },
  },
  {
    ...primary,
    id: "dependency-delete",
    child: null,
    graph: { nodes: 0, edges: 0, cycles: 0, score: 0 },
    total: 1,
    blocks: 1,
    files: { "App.vue": app },
  },
  {
    ...primary,
    id: "dependency-rename",
    child: "Renamed.vue",
    files: { "App.vue": app.replace("./Child.vue", "./Renamed.vue"), "Renamed.vue": child },
  },
  {
    ...primary,
    id: "comment-and-string-control",
    graph: { nodes: 0, edges: 0, cycles: 0, score: 0 },
    total: 2,
    files: {
      "App.vue": app,
      "Child.vue": `<script setup>
// const count = ref(0); const doubled = computed(() => count.value)
const label = 'ref(0), computed(() => count.value)'
</script>
<template><p>{{ label }}</p></template>
`,
    },
  },
  { ...primary, id: "repair" },
];
