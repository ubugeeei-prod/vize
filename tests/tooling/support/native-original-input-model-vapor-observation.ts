// Diagnostic-only own-descriptor graph, with no Vue/provider/DOM imports.
import assert from "node:assert/strict";
import type { ObjectValue } from "./native-original-input-model-vapor-boundary.ts";

export function vaporDiagnostic(value: unknown, seen = new Map<object, number>()): unknown {
  if (value === undefined) return { kind: "undefined" };
  if (typeof value === "bigint") return { kind: "bigint", decimal: value.toString() };
  if (typeof value === "symbol") return { kind: "symbol", description: value.description ?? null };
  if (typeof value === "number" && (!Number.isFinite(value) || Object.is(value, -0)))
    return { kind: "number", spelling: Object.is(value, -0) ? "-0" : String(value) };
  if (value === null || (typeof value !== "object" && typeof value !== "function")) return value;
  const prior = seen.get(value);
  if (prior !== undefined) return { kind: "reference", id: prior };
  const id = seen.size;
  seen.set(value, id);
  return {
    kind: typeof value === "function" ? "function" : Array.isArray(value) ? "array" : "object",
    id,
    ...(typeof value === "function" ? { source: Function.prototype.toString.call(value) } : {}),
    entries: Reflect.ownKeys(value).map((key) => {
      const descriptor = Object.getOwnPropertyDescriptor(value, key);
      assert(descriptor);
      return [
        typeof key === "symbol" ? { symbol: key.description ?? null } : key,
        {
          enumerable: descriptor.enumerable,
          configurable: descriptor.configurable,
          ...("value" in descriptor
            ? { writable: descriptor.writable, value: vaporDiagnostic(descriptor.value, seen) }
            : {
                get: vaporDiagnostic(Reflect.get(descriptor, "get"), seen),
                set: vaporDiagnostic(Reflect.get(descriptor, "set"), seen),
              }),
        },
      ];
    }),
  };
}
export function modelVaporError(error: unknown): ObjectValue {
  return {
    name: error instanceof Error ? error.name : typeof error,
    message: error instanceof Error ? error.message : String(error),
    stack: error instanceof Error ? (error.stack ?? null) : null,
    completeOwnDescriptors: vaporDiagnostic(error),
  };
}
