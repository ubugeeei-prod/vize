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
