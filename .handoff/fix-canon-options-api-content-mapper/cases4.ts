// Options API matrix, part 4: Vue 2.7 projects.
import * as fs from "node:fs";
import * as path from "node:path";
import { write } from "./cases1.ts";

const S = path.dirname(new URL(import.meta.url).pathname);
const DEPS2 = path.join(S, "deps2/node_modules");

const tsconfig = (extra: Record<string, unknown> = {}) => JSON.stringify({
  compilerOptions: {
    target: "ESNext", module: "ESNext", moduleResolution: "bundler", strict: true, noEmit: true,
    skipLibCheck: true, lib: ["ESNext", "DOM"], types: [], ...extra,
  },
  vueCompilerOptions: { target: 2.7 },
  include: ["src/**/*.ts", "src/**/*.vue", "src/**/*.js"],
  contentMappers: [{ package: "vize", extensions: [".vue"] }],
}, null, 2);

function write2(id: string, files: Record<string, string>) {
  write(id, files);
  const nm = path.join(S, "matrix", id, "node_modules");
  fs.mkdirSync(nm, { recursive: true });
  for (const name of ["vue", "vue-router", "vuex", "@vue", "csstype"]) {
    if (fs.existsSync(path.join(DEPS2, name))) fs.symlinkSync(path.join(DEPS2, name), path.join(nm, name));
  }
}

write2("vue27-define-component", {
  "tsconfig.json": tsconfig(),
  "src/Comp.vue": `
<script lang="ts">
import { defineComponent, type PropType } from 'vue'
export default defineComponent({
  props: {
    title: { type: String, required: true },
    items: { type: Array as PropType<number[]>, default: () => [] },
  },
  data() { return { local: 1 } },
  computed: { upper(): string { return this.title.toUpperCase() } },
  methods: { inc(by: number) { this.local += by; this.$emit('change', by) } },
})
</script>

<template>
  <div>
    {{ title.toUpperCase() }} {{ items.length }} {{ local.toFixed() }} {{ upper.length }}
    <button @click="inc(1)">+</button>
    {{ title.toFixed() }} {{ local.length }} {{ nope }}
    <button @click="inc('x')">x</button>
  </div>
</template>
`,
  "src/Parent.vue": `
<script lang="ts">
import { defineComponent } from 'vue'
import Comp from './Comp.vue'
export default defineComponent({ components: { Comp } })
</script>

<template>
  <div>
    <Comp title="a" />
    <Comp :title="1" />
  </div>
</template>
`,
});

write2("vue27-vue-extend", {
  "tsconfig.json": tsconfig(),
  "src/Comp.vue": `
<script lang="ts">
import Vue from 'vue'
export default Vue.extend({
  props: { title: { type: String, required: true } },
  data() { return { local: 1 } },
  methods: { inc(by: number) { this.local += by } },
})
</script>

<template>
  <div>
    {{ title.toUpperCase() }} {{ local.toFixed() }}
    <button @click="inc(1)">+</button>
    {{ title.toFixed() }} {{ nope }}
  </div>
</template>
`,
});

write2("vue27-plain-object-filters", {
  "tsconfig.json": tsconfig(),
  "src/Comp.vue": `
<script lang="ts">
export default {
  props: { price: { type: Number, required: true } },
  data() { return { label: 'x' } },
  filters: { money(n: number) { return n.toFixed(2) } },
}
</script>

<template>
  <div>{{ price | money }} {{ label.toUpperCase() }} {{ label.toFixed() }}</div>
</template>
`,
});
