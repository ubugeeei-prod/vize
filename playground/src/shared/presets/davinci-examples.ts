/** Small programs that expose different lowering and reactivity decisions. */
export const DAVINCI_MORE_EXAMPLES = [
  {
    key: "branches",
    label: "Conditional branches",
    code: `<script setup lang="ts">
import { ref } from "vue";
const state = ref<"loading" | "error" | "ready">("ready");
const message = ref("Connection restored");
</script>

<template>
  <section aria-live="polite">
    <p v-if="state === 'loading'">Connecting…</p>
    <p v-else-if="state === 'error'" role="alert">Try again.</p>
    <div v-else><strong>Ready</strong><p>{{ message }}</p></div>
    <button @click="state = 'loading'">Reconnect</button>
  </section>
</template>
`,
  },
  {
    key: "nested-loops",
    label: "Nested keyed loops",
    code: `<script setup lang="ts">
import { ref } from "vue";
const groups = ref([
  { id: "compiler", name: "Compiler", tasks: [{ id: 1, title: "Lower IR", done: true }] },
  { id: "tools", name: "Tooling", tasks: [{ id: 2, title: "Trace spans", done: false }] },
]);
</script>

<template>
  <section v-for="group in groups" :key="group.id">
    <h2>{{ group.name }}</h2>
    <ul>
      <li v-for="(task, index) in group.tasks" :key="task.id" :class="{ done: task.done }">
        <span>{{ index + 1 }}. {{ task.title }}</span>
        <input type="checkbox" v-model="task.done" />
      </li>
    </ul>
  </section>
</template>
`,
  },
  {
    key: "events",
    label: "Events & modifiers",
    code: `<script setup lang="ts">
import { ref } from "vue";
const query = ref("");
const submitted = ref("");
function submit() { submitted.value = query.value; }
function clear() { query.value = ""; }
</script>

<template>
  <form @submit.prevent="submit">
    <input v-model="query" @keydown.esc="clear" aria-label="Search" />
    <button type="submit">Search</button>
    <button type="button" @click.stop="clear">Clear</button>
    <output>{{ submitted }}</output>
  </form>
</template>
`,
  },
  {
    key: "form-models",
    label: "Form model lowering",
    code: `<script setup lang="ts">
import { ref } from "vue";
const name = ref("");
const quantity = ref(1);
const enabled = ref(true);
const priority = ref("normal");
</script>

<template>
  <fieldset>
    <legend>Build settings</legend>
    <label>Name <input v-model.trim="name" /></label>
    <label>Workers <input type="number" v-model.number="quantity" /></label>
    <label><input type="checkbox" v-model="enabled" /> Cache</label>
    <select v-model="priority" aria-label="Priority">
      <option value="normal">Normal</option>
      <option value="high">High</option>
    </select>
    <output>{{ name }} / {{ quantity }} / {{ enabled }} / {{ priority }}</output>
  </fieldset>
</template>
`,
  },
  {
    key: "component-models",
    label: "Component models",
    code: `<script setup lang="ts">
import { ref } from "vue";
import RangeControl from "./RangeControl.vue";
const lower = ref(10);
const upper = ref(90);
</script>

<template>
  <RangeControl v-model:min="lower" v-model:max="upper" :step="5">
    <template #label>Analysis window</template>
  </RangeControl>
  <p>{{ lower }} → {{ upper }}</p>
</template>
`,
  },
  {
    key: "scoped-slots",
    label: "Scoped slots",
    code: `<script setup lang="ts">
import DataTable from "./DataTable.vue";
const rows = [{ id: 1, name: "parse", duration: 12 }, { id: 2, name: "lower", duration: 8 }];
</script>

<template>
  <DataTable :rows="rows">
    <template #header><h2>Compiler passes</h2></template>
    <template #row="{ row, index }">
      <tr><td>{{ index }}</td><td>{{ row.name }}</td><td>{{ row.duration }} ms</td></tr>
    </template>
    <template #empty>No measurements</template>
  </DataTable>
</template>
`,
  },
  {
    key: "dynamic-bindings",
    label: "Dynamic bindings",
    code: `<script setup lang="ts">
import { ref } from "vue";
const attribute = ref("title");
const label = ref("Inspect operation");
const active = ref(true);
const color = ref("teal");
const extras = { id: "node", tabindex: 0 };
</script>

<template>
  <button v-bind="extras" :[attribute]="label" :class="{ active }"
    :style="{ color }" @click="active = !active">
    {{ label }}
  </button>
</template>
`,
  },
  {
    key: "svg",
    label: "SVG & namespaces",
    code: `<script setup lang="ts">
import { ref } from "vue";
const progress = ref(72);
const color = ref("#12a594");
</script>

<template>
  <svg viewBox="0 0 120 120" role="img" aria-label="Progress">
    <circle cx="60" cy="60" r="48" fill="none" stroke="#ddd" />
    <circle cx="60" cy="60" r="48" fill="none" :stroke="color"
      :stroke-dasharray="progress + ' 302'" stroke-width="8" />
    <text x="60" y="64" text-anchor="middle">{{ progress }}%</text>
  </svg>
</template>
`,
  },
  {
    key: "unicode",
    label: "Unicode & inline units",
    code: `<script setup lang="ts">
import { ref } from "vue";
const greeting = ref("こんにちは 🌸");
const name = ref("世界");
</script>

<template>
  <p title="日本語" :lang="'ja'">{{ greeting }}<strong>{{ name }}</strong>！</p>
  <p>UTF-8 bytes ≠ UTF-16 code units</p>
  <input v-model="name" aria-label="名前" />
</template>
`,
  },
  {
    key: "derived-state",
    label: "Derived reactive state",
    code: `<script setup lang="ts">
import { computed, ref } from "vue";
const price = ref(1200);
const quantity = ref(2);
const discount = ref(0.1);
const subtotal = computed(() => price.value * quantity.value);
const total = computed(() => Math.round(subtotal.value * (1 - discount.value)));
</script>

<template>
  <article>
    <h2>Order summary</h2>
    <p>{{ price }} × {{ quantity }} = {{ subtotal }}</p>
    <output :class="{ discounted: discount > 0 }">Total: {{ total }}</output>
    <button @click="quantity++">Add one</button>
    <small>Tax included</small>
  </article>
</template>
`,
  },
];
