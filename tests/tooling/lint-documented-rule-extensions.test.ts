import assert from "node:assert/strict";
import { test } from "node:test";

import {
  baselineIntentionalReason,
  patinaIntentionalReason,
} from "../../tools/support/fixtures/lint_divergence/records.ts";

const source = `<script>
import { nextTick } from 'vue'
export default {
  mounted() {
    this.$nextTick(() => this.measure())
  },
}
</script>
<script lang="ts" setup>
nextTick(() => undefined)
await nextTick(() => undefined)
</script>
`;

function finding(ruleId: string, line: number, column: number, length: number, message = "") {
  return {
    file: "src/Panel.vue",
    ruleId,
    severity: "warning",
    line,
    column,
    endLine: line,
    endColumn: column + length,
    message,
  };
}

test("next-tick-style findings in <script setup> are a documented extension", () => {
  assert.match(
    patinaIntentionalReason(finding("vue/next-tick-style", 10, 1, 8), source) ?? "",
    /#12450.*#12670/u,
  );
});

test("next-tick-style findings in component options stay counted", () => {
  assert.equal(patinaIntentionalReason(finding("vue/next-tick-style", 5, 10, 9), source), null);
});

test("valid-next-tick findings in <script setup> are a documented extension", () => {
  assert.match(
    patinaIntentionalReason(finding("script/valid-next-tick", 11, 7, 8), source) ?? "",
    /#12794/u,
  );
});

test("valid-next-tick findings in component options stay counted", () => {
  assert.equal(patinaIntentionalReason(finding("script/valid-next-tick", 5, 10, 9), source), null);
});

test("custom-event-name-casing keeps the model prop name after update:", () => {
  const reported = (name: string) =>
    baselineIntentionalReason(
      finding(
        "vue/custom-event-name-casing",
        3,
        8,
        name.length + 2,
        `Custom event name '${name}' must be camelCase.`,
      ),
      source,
    );
  assert.match(reported("update:selected-date") ?? "", /#9879.*#12592.*#12587/u);
  assert.equal(reported("selected-date"), null);
  assert.equal(reported("close-dialog:done"), null);
});
