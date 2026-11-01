// Source and closed primitive boundaries; importing this leaf loads no provider.
import assert from "node:assert/strict";
import {
  assertFixture,
  type ModelSsrFixture,
  type ModelSsrMode,
} from "./native-original-input-model-sfc-ssr-boundary.ts";
export type CheckboxMode = ModelSsrMode;
type Transitions = { afterProgrammatic: boolean; secondClick: boolean; thirdClick: boolean };
export type CheckboxFixture = ModelSsrFixture & {
  checkboxRuntime: {
    initialChecked: boolean;
    firstClick: boolean;
    programmatic: boolean;
    dom: Transitions;
    vapor: Transitions;
  };
};
function transitions(value: unknown): asserts value is Transitions {
  assert(value && typeof value === "object" && !Array.isArray(value));
  assert.deepEqual(Object.keys(value), ["afterProgrammatic", "secondClick", "thirdClick"]);
  for (const field of Object.values(value)) assert.equal(typeof field, "boolean");
}
export function assertCheckboxFixture(value: unknown): asserts value is CheckboxFixture {
  assertFixture(value);
  const runtime: unknown = Reflect.get(value, "checkboxRuntime");
  assert(runtime && typeof runtime === "object" && !Array.isArray(runtime));
  assert.deepEqual(Object.keys(runtime), [
    "initialChecked",
    "firstClick",
    "programmatic",
    "dom",
    "vapor",
  ]);
  for (const field of ["initialChecked", "firstClick", "programmatic"])
    assert.equal(typeof Reflect.get(runtime, field), "boolean");
  transitions(Reflect.get(runtime, "dom"));
  transitions(Reflect.get(runtime, "vapor"));
}
