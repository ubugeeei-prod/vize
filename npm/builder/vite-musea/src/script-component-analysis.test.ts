import assert from "node:assert/strict";
import { test } from "node:test";

import { analyzeScriptComponent } from "./script-component-analysis.ts";

void test("analyzes a default exported render-function component without running its module", () => {
  const source = `
import { defineComponent, h } from "vue";
throw new Error("analysis must not execute this file");
export default defineComponent({
  name: "MyButton",
  props: {
    label: { type: String, default: "Button" },
    count: { type: Number, required: true },
    disabled: Boolean,
  },
  emits: ["click"],
  setup: props => () => h("button", props.label),
});`;

  assert.deepEqual(analyzeScriptComponent(source, "my-button.ts"), {
    props: [
      { name: "label", type: "string", required: false, default_value: "Button" },
      { name: "count", type: "number", required: true },
      { name: "disabled", type: "boolean", required: false },
    ],
    emits: ["click"],
  });
});

void test("analyzes aliased default exports, array props, and object emits in TSX", () => {
  const source = `
const props = ["label", "tone"];
const MyButton = defineComponent({
  props,
  emits: { submit: (_value: string) => true },
  setup: () => () => <button />,
});
export default MyButton;`;

  assert.deepEqual(analyzeScriptComponent(source, "my-button.tsx"), {
    props: [
      { name: "label", type: "unknown", required: false },
      { name: "tone", type: "unknown", required: false },
    ],
    emits: ["submit"],
  });
});

void test("ignores unrelated code and dynamic default expressions", () => {
  assert.deepEqual(analyzeScriptComponent("export default makeComponent()", "dynamic.js"), {
    props: [],
    emits: [],
  });
});
