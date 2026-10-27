/** Preserve JSON integer lexemes beyond the IEEE-754 safe range. */
import assert from "node:assert/strict";
export const isRecord = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === "object" && !Array.isArray(value);
export function parseEvidenceJson(text: string): unknown {
  return JSON.parse(text, (_key: string, value: unknown, context?: { source?: string }) => {
    if (typeof value !== "number" || Number.isSafeInteger(value)) return value;
    const raw = context?.source;
    assert.ok(raw, "runtime JSON source context required for unsafe numeric values");
    if (/^-?\d+$/u.test(raw)) return BigInt(raw);
    assert.ok(Number.isFinite(value), "non-finite JSON number");
    return value;
  }) as unknown;
}
export function stringifyEvidenceJson(value: unknown, pretty = false) {
  const raw: unknown = Reflect.get(JSON, "rawJSON");
  assert.equal(typeof raw, "function", "runtime raw JSON support required");
  assert.ok(typeof raw === "function");
  return JSON.stringify(
    value,
    (_key: string, item: unknown) => (typeof item === "bigint" ? raw(item.toString()) : item),
    pretty ? 2 : undefined,
  );
}
export function integerEvidence(value: unknown): bigint {
  if (typeof value === "bigint") return value;
  assert.ok(typeof value === "number" && Number.isSafeInteger(value));
  return BigInt(value);
}
export function recordEvidence(value: unknown): Record<string, unknown> {
  assert.ok(isRecord(value));
  return value;
}
export function arrayEvidence(value: unknown): unknown[] {
  assert.ok(Array.isArray(value));
  return value;
}
