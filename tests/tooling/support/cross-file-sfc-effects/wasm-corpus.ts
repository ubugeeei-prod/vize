import { cases, type EffectCase } from "./corpus.ts";

type Graph = EffectCase["graph"];
export interface WasmCase {
  id: string;
  files: Record<string, string>;
  child: string | null;
  before: Graph;
  after: Graph;
  diagnostics: boolean;
  afterBand: "low" | "moderate";
}

const primary = cases[0];
const app = primary.files["App.vue"];
const edge = { nodes: 2, edges: 1, cycles: 0, score: 4 };
const dropped = { nodes: 2, edges: 0, cycles: 0, score: 2 };
function extra(id: string, source: string, before = edge, after = edge): WasmCase {
  return {
    id,
    files: { "App.vue": app, "Child.vue": source },
    child: "Child.vue",
    before,
    after,
    diagnostics: false,
    afterBand: "low",
  };
}
const shared = primary.files["Child.vue"];

export const wasmCases: WasmCase[] = [
  ...cases.map((test): WasmCase => ({
    id: test.id,
    files: test.files,
    child: test.child,
    after: test.graph,
    before: test.wasmComparable
      ? test.graph
      : { nodes: test.graph.nodes, edges: 0, cycles: 0, score: test.graph.nodes },
    diagnostics: test.diagnostics,
    afterBand: test.band,
  })),
  extra("js-setup", shared.replace('lang="ts"', 'lang="js"')),
  extra("js-normal", shared.replace("script setup", "script").replace('lang="ts"', 'lang="js"')),
  extra("ts-normal", shared.replace("script setup", "script")),
  extra(
    "jsx-setup",
    shared
      .replace('lang="ts"', 'lang="jsx"')
      .replace("const count", "const view = <p>日本語🙂</p>\nconst count"),
    dropped,
    edge,
  ),
  extra(
    "tsx-normal",
    shared
      .replace("script setup", "script")
      .replace('lang="ts"', 'lang="tsx"')
      .replace("const count", "const view = <p>日本語🙂</p>\nconst count"),
    dropped,
    edge,
  ),
  extra(
    "mixed-jsx-ts",
    `<script lang="jsx">
import { ref } from 'vue'
const view = <section />
const count = ref(0)
export default {}
</script>
<script setup lang="ts">
import { computed } from 'vue'
const doubled = computed(() => count.value * 2)
</script>
<template><p>{{ doubled }}</p></template>
`,
    dropped,
    edge,
  ),
  extra(
    "mixed-shadow",
    `<script>
import { ref, computed } from 'vue'
const count = ref(0)
const before = computed(() => count.value)
export default {}
</script>
<script setup>
const count = ref(1)
const after = computed(() => count.value)
</script>
<template><p>{{ after }}</p></template>
`,
    { nodes: 4, edges: 2, cycles: 0, score: 8 },
    { nodes: 4, edges: 2, cycles: 0, score: 8 },
  ),
  extra(
    "js-comparison-control",
    `<script setup lang="js">
import { ref, computed } from 'vue'
const count = ref < number > (0)
const doubled = computed(() => count.value)
</script>
<template><p>{{ doubled }}</p></template>
`,
    { nodes: 2, edges: 1, cycles: 0, score: 4 },
    { nodes: 2, edges: 0, cycles: 0, score: 2 },
  ),
  extra(
    "malformed-setup-retains-valid-normal",
    `<script>
import { ref, computed } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value)
export default {}
</script>
<script setup>const broken =</script>
<template><p>{{ doubled }}</p></template>
`,
    dropped,
    edge,
  ),
  {
    id: "raw-jsx-module-control",
    files: {
      "standalone.jsx":
        "import { ref, computed } from 'vue'\nconst node = <p>日本語🙂</p>\nconst count = ref(0)\nconst doubled = computed(() => count.value * 2)\n",
    },
    child: null,
    before: { nodes: 0, edges: 0, cycles: 0, score: 0 },
    after: { nodes: 0, edges: 0, cycles: 0, score: 0 },
    diagnostics: false,
    afterBand: "low",
  },
];
