// Closed original source, primitive and authored-token custody boundaries.
import assert from "node:assert/strict";
import {
  assertFixture,
  object,
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

function authoredToken(value: unknown, source: string, token: string) {
  assert(Array.isArray(value) && value.length === 2);
  const [start, end] = value;
  const bytes = Buffer.from(source);
  assert(typeof start === "number" && Number.isSafeInteger(start) && start >= 0);
  assert(
    typeof end === "number" && Number.isSafeInteger(end) && end >= start && end <= bytes.length,
  );
  assert.equal(bytes.subarray(start, end).toString("utf8"), token);
}
export function assertCheckboxCustody(value: unknown) {
  object(value);
  assert.equal(value.inputKind, "checkbox");
  assert(typeof value.source === "string");
  object(value.typeObservation);
  const type = value.typeObservation;
  assert.equal(type.name, "type");
  assert.equal(type.value, "checkbox");
  assert.equal(type.rawValue, "checkbox");
  assert.equal(type.canonicalSlot, 0);
  assert.equal(type.actualClosedAttributePointer, true);
  assert.equal(type.sameOriginalAttributeValueSource, true);
  authoredToken(type.nameSpan, value.source, "type");
  authoredToken(type.valueSpan, value.source, "checkbox");
  for (const field of [
    "actualElementPointer",
    "actualModelPointer",
    "sameFileBinding",
    "sameOriginalAstAndCoordinates",
    "actualCanonicalTargetRole",
  ])
    assert.equal(value[field], true);
}
