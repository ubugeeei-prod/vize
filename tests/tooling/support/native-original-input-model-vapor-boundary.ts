// Source/scalar boundaries only; importing this file never loads Vue or a DOM.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";

export const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
export const fromVaporVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
export const hash = (value: string) => createHash("sha256").update(value).digest("hex");
export const dataUrl = (value: string) =>
  `data:text/javascript;base64,${Buffer.from(value).toString("base64")}`;
export type ObjectValue = Record<string, unknown>;
export function object(value: unknown): asserts value is ObjectValue {
  assert(value && typeof value === "object" && !Array.isArray(value));
}
export type ModelVaporMode = "development" | "production";
export type Primitive =
  | { kind: "null" }
  | { kind: "undefined" }
  | { kind: "string"; value: string }
  | { kind: "boolean"; value: boolean }
  | { kind: "number"; value: number }
  | { kind: "bigint"; decimal: string };
export function primitive(value: unknown): Primitive {
  if (value === null) return { kind: "null" };
  if (value === undefined) return { kind: "undefined" };
  if (typeof value === "string") return { kind: "string", value };
  if (typeof value === "boolean") return { kind: "boolean", value };
  if (typeof value === "number") {
    assert(Number.isFinite(value) && !Object.is(value, -0));
    return { kind: "number", value };
  }
  assert(typeof value === "bigint", "closed primitive, never coercion");
  return { kind: "bigint", decimal: value.toString() };
}
export function primitiveValue(value: Primitive) {
  if (value.kind === "null") return null;
  if (value.kind === "undefined") return undefined;
  if (value.kind === "bigint") return BigInt(value.decimal);
  return value.value;
}
export type ModelVaporFixture = {
  id: string;
  source: string;
  script: string;
  template: string;
  bindings: string[];
  binding: string;
  initial: Primitive;
  input: string;
  programmatic: string;
  composing: string;
};
export function assertModelFixture(value: unknown): asserts value is ModelVaporFixture {
  object(value);
  for (const name of [
    "id",
    "source",
    "script",
    "template",
    "binding",
    "input",
    "programmatic",
    "composing",
  ])
    assert(typeof value[name] === "string" && value[name].length > 0);
  assert.deepEqual(value.bindings, [value.binding]);
  object(value.initial);
  const initial = value.initial;
  if (initial.kind === "null") assert.deepEqual(Reflect.ownKeys(initial), ["kind"]);
  else if (initial.kind === "bigint") {
    assert.deepEqual(Reflect.ownKeys(initial), ["kind", "decimal"]);
    assert(typeof initial.decimal === "string" && /^-?\d+$/.test(initial.decimal));
  } else {
    assert(initial.kind === "string" || initial.kind === "number" || initial.kind === "boolean");
    assert.deepEqual(Reflect.ownKeys(initial), ["kind", "value"]);
    assert.equal(typeof initial.value, initial.kind);
    if (initial.kind === "number") assert(Number.isFinite(initial.value));
  }
}
