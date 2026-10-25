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
      ["value", ""],
      ["inputId", "field"],
      ["type", "text"],
    ],
  );
});

void test("older native fallback preserves JSON primitive types and marks expressions unsupported", () => {
  const result = analyzeSfcFallback(`<script setup lang="ts">
  withDefaults(defineProps<{ code?: string; flag?: string; count?: number; enabled?: boolean; value?: null; dynamic?: object }>(), {
    code: "007", flag: "true", count: -2, enabled: false, value: null, dynamic: readState()
  });</script>`);
  assert.deepEqual(
    result.props.map(({ name, default_value, default_source, unsupported_default }) => ({
      name,
      default_value,
      default_source,
      unsupported_default,
    })),
    [
      { name: "code", default_value: "007" },
      { name: "flag", default_value: "true" },
      { name: "count", default_value: -2 },
      { name: "enabled", default_value: false },
      { name: "value", default_value: null },
      { name: "dynamic", default_source: "readState()", unsupported_default: true },
    ].map((prop) => ({
      default_value: undefined,
      default_source: undefined,
      unsupported_default: undefined,
      ...prop,
    })),
  );
});
