import assert from "node:assert/strict";
import test from "node:test";

import { analyzeSfcFallback } from "./native-loader.ts";

void test("fallback SFC analysis reads defaults only from withDefaults", () => {
  const result = analyzeSfcFallback(`
<script setup lang="ts">
const value = (event: Event) => {
  return (event.target as HTMLInputElement).value;
};

const props = withDefaults(defineProps<{
  value?: string;
  inputId?: string;
  type?: "text" | "number";
}>(), {
  value: "",
  inputId: "field",
  type: "text",
});
</script>
`);

  assert.deepEqual(
    result.props.map((prop) => [prop.name, prop.default_value]),
    [
      ["value", '""'],
      ["inputId", '"field"'],
      ["type", '"text"'],
    ],
  );
});
