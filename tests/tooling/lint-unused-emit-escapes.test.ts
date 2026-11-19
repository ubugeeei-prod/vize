import assert from "node:assert/strict";
import { test } from "node:test";

import { baselineIntentionalReason } from "../../tools/support/fixtures/lint_divergence/records.ts";

/** An upstream unused-emit finding on `event` in `sfc`. */
function unused(sfc: string, event: string): string | null {
  return baselineIntentionalReason(
    {
      file: "src/AutofillableField.vue",
      ruleId: "vue/no-unused-emit-declarations",
      severity: "warning",
      line: 3,
      column: 3,
      endLine: 3,
      endColumn: 3 + event.length,
      message: `\`${event}\` is defined as emit but never used.`,
    },
    sfc,
  );
}

const instance = `<script setup lang="ts">
import { getCurrentInstance } from 'vue'
const emit = defineEmits<{ (e: 'click:prepend'): void; (e: 'submit'): void }>()
const inst = getCurrentInstance()
</script>`;

test("an unused-emit report beside getCurrentInstance() is Patina's escape", () => {
  assert.match(unused(instance, "click:prepend") ?? "", /getCurrentInstance\(\)/u);
});

test("unused-emit reports without an instance call stay counted", () => {
  for (const sfc of [
    instance.replace("const inst = getCurrentInstance()", "// getCurrentInstance()"),
    instance.replace("const inst = getCurrentInstance()", "const inst = app.getCurrentInstance()"),
    `<script lang="ts">\nconst inst = getCurrentInstance()\n</script>\n${instance.replace("const inst = getCurrentInstance()", "")}`,
  ])
    assert.equal(unused(sfc, "click:prepend"), null);
});

test("computed and Patina-resolved emit names classify upstream reports", () => {
  const computed = `<script setup lang="ts">
const emit = defineEmits(['ready', 'resize'])
const emitEvent = (name: string) => {
  emit(name, {})
}
</script>`;
  assert.match(unused(computed, "ready") ?? "", /computed event name/u);
  const template = `<script setup>
const emit = defineEmits(['save', 'close'])
</script>
<template><button @click="emit(action)" /><p>emit(prose) only</p></template>`;
  assert.match(unused(template, "close") ?? "", /computed event name/u);
  const resolved = `<script setup lang="ts">
const emit = defineEmits(['change', 'open', 'close', 'save', 'unused'])
const CHANGE = 'change'
const EVENTS = { save: 'save' } as const
declare const isOpen: boolean
emit(CHANGE)
emit(isOpen ? 'open' : 'close')
emit(EVENTS.save)
</script>`;
  for (const event of ["change", "open", "close", "save"])
    assert.match(unused(resolved, event) ?? "", /resolves this event name/u, event);
  assert.equal(unused(resolved, "unused"), null);
  const literal = `<script setup>\nconst emit = defineEmits(['a', 'b'])\nemit('a')\nemit(\`a\`)\n</script>\n<template><p>emit(prose)</p></template>`;
  assert.equal(unused(literal, "b"), null);
  const nested = `<script setup>\nconst emit = defineEmits(['a', 'b'])\nfunction f() {\n  const A = 'a'\n  emit(A)\n}\n</script>`;
  assert.match(unused(nested, "b") ?? "", /computed event name/u);
});

test("a dynamic template $emit classifies an unbound defineEmits declaration", () => {
  const sfc = `<script setup>
defineEmits(['rename', 'delete'])
const menu = [{ name: 'rename' }, { name: 'delete' }]
</script>
<template><button v-for="item in menu" @click="$emit(item.name)" /></template>`;
  for (const event of ["rename", "delete"])
    assert.match(unused(sfc, event) ?? "", /computed event name/u, event);
});
