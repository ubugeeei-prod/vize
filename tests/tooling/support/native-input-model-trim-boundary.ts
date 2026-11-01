// Original-input and JSON scalar boundary; no compiler or runtime execution.
import assert from "node:assert/strict";
import {
  assertModelFixture,
  type ModelVaporFixture,
} from "./native-original-input-model-vapor-boundary.ts";
export {
  hash,
  object,
  primitive,
  primitiveValue,
  type ObjectValue,
} from "./native-original-input-model-vapor-boundary.ts";
export type TrimFixture = ModelVaporFixture & { change: string };
export type TrimMode = "development" | "production";
export type TrimTarget = "dom" | "ssr" | "vapor";
export function assertTrimFixture(value: unknown): asserts value is TrimFixture {
  assertModelFixture(value);
  const change: unknown = Reflect.get(value, "change");
  assert(typeof change === "string" && change.length > 0);
}
