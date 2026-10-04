// Failure receipts preserve non-JSON scalars and graph identity explicitly.
// This representation is diagnostic only and grants no output eligibility.
export function diagnosticValue(value: any, seen = new Map<object, number>()): any {
  if (value === undefined) return { kind: "undefined" };
  if (typeof value === "bigint") return { kind: "bigint", decimal: String(value) };
  if (typeof value === "number" && (!Number.isFinite(value) || Object.is(value, -0)))
    return { kind: "number", spelling: Object.is(value, -0) ? "-0" : String(value) };
  if (typeof value === "symbol") return { kind: "symbol", description: value.description };
  if (typeof value === "function") return { kind: "function", source: value.toString() };
  if (value === null || typeof value !== "object") return value;
  const existing = seen.get(value);
  if (existing !== undefined) return { kind: "reference", id: existing };
  const id = seen.size;
  seen.set(value, id);
  return {
    kind: Array.isArray(value) ? "array" : "object",
    id,
    entries: Object.entries(value).map(([key, item]) => [key, diagnosticValue(item, seen)]),
  };
}

export function runtimeErrorDetails(error: any) {
  return {
    name: error?.name ?? typeof error,
    message: error?.message ?? String(error),
    stack: error?.stack ?? null,
    code: error?.code ?? null,
    operator: error?.operator ?? null,
    actualPresent: Object.hasOwn(error ?? {}, "actual"),
    actual: diagnosticValue(error?.actual),
    expectedPresent: Object.hasOwn(error ?? {}, "expected"),
    expected: diagnosticValue(error?.expected),
  };
}
