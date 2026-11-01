// Number-only lossless test transport; old primitive policy remains unchanged.
import assert from "node:assert/strict";
import { assertTrimFixture, type TrimFixture } from "./native-input-model-trim-boundary.ts";
import { object, primitive, type Primitive } from "./native-original-input-model-vapor-boundary.ts";

export type NumberPrimitive =
  | Primitive
  | { kind: "special-number"; spelling: "-0" | "Infinity" | "-Infinity" };
export type NumberExpected = Extract<
  NumberPrimitive,
  { kind: "string" | "number" | "special-number" }
>;
export type NumberFixture = TrimFixture & {
  numberRuntime: {
    inputModel: NumberExpected;
    compositionEndModel: NumberExpected;
    changeModel: NumberExpected;
    changeDom: string;
    afterChangeRenderDom: string;
  };
};

export function numberPrimitive(value: unknown): NumberPrimitive {
  if (Object.is(value, -0)) return { kind: "special-number", spelling: "-0" };
  if (value === Infinity) return { kind: "special-number", spelling: "Infinity" };
  if (value === -Infinity) return { kind: "special-number", spelling: "-Infinity" };
  return primitive(value);
}

export function numberExpectedValue(value: NumberExpected): string | number {
  if (value.kind !== "special-number") return value.value;
  if (value.spelling === "-0") return -0;
  return value.spelling === "Infinity" ? Infinity : -Infinity;
}

function assertExpected(value: unknown): asserts value is NumberExpected {
  object(value);
  if (value.kind === "special-number") {
    assert.deepEqual(Reflect.ownKeys(value), ["kind", "spelling"]);
    assert(
      value.spelling === "-0" || value.spelling === "Infinity" || value.spelling === "-Infinity",
    );
  } else {
    assert.deepEqual(Reflect.ownKeys(value), ["kind", "value"]);
    if (value.kind === "string") assert(typeof value.value === "string");
    else {
      assert.equal(value.kind, "number");
      assert(typeof value.value === "number" && Number.isFinite(value.value));
      assert(!Object.is(value.value, -0), "special numeric values use explicit lossless tags");
    }
  }
}

export function assertNumberFixture(value: unknown): asserts value is NumberFixture {
  assertTrimFixture(value);
  const runtime: unknown = Reflect.get(value, "numberRuntime");
  object(runtime);
  assert.deepEqual(Reflect.ownKeys(runtime), [
    "inputModel",
    "compositionEndModel",
    "changeModel",
    "changeDom",
    "afterChangeRenderDom",
  ]);
  for (const field of ["inputModel", "compositionEndModel", "changeModel"])
    assertExpected(runtime[field]);
  assert(typeof runtime.changeDom === "string" && typeof runtime.afterChangeRenderDom === "string");
}
