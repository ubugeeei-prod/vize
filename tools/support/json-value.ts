/**
 * Narrowing guards for parsed JSON in strictly typed tooling (#8840).
 *
 * `JSON.parse` and `createRequire()` return `any`. Typed tools assign those
 * results to `unknown` and narrow them here, so a malformed artifact fails
 * where it is read instead of being typed as whatever it claims to be.
 */
import assert from "node:assert/strict";
import fs from "node:fs";

export type JsonRecord = { [key: string]: unknown };

export function isRecord(value: unknown): value is JsonRecord {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function asRecord(value: unknown, label = "value"): JsonRecord {
  assert.ok(isRecord(value), `${label} must be an object`);
  return value;
}

export function asArray(value: unknown, label = "value"): unknown[] {
  assert.ok(Array.isArray(value), `${label} must be an array`);
  return value;
}

export function asString(value: unknown, label = "value"): string {
  assert.equal(typeof value, "string", `${label} must be a string`);
  return value as string;
}

export function asNumber(value: unknown, label = "value"): number {
  assert.equal(typeof value, "number", `${label} must be a number`);
  return value as number;
}

export function asStringArray(value: unknown, label = "value"): string[] {
  return asArray(value, label).map((entry, index) => asString(entry, `${label}[${index}]`));
}

/** Follow `keys` through nested objects and arrays; a missing step yields `undefined`. */
export function at(value: unknown, ...keys: readonly (string | number)[]): unknown {
  let current = value;
  for (const key of keys) {
    if (typeof key === "number") current = Array.isArray(current) ? current[key] : undefined;
    else current = isRecord(current) ? current[key] : undefined;
  }
  return current;
}

export function parseJson(text: string): unknown {
  return JSON.parse(text) as unknown;
}

export function readJson(file: string | URL): unknown {
  return parseJson(fs.readFileSync(file, "utf8"));
}

export function readJsonRecord(file: string | URL): JsonRecord {
  return asRecord(readJson(file), String(file));
}
