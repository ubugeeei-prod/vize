/** Exact integer evidence parsing; JSON integer tokens retain all u64 bits. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

export type RecordValue = Record<string, unknown>;
export function record(value: unknown): RecordValue {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as RecordValue;
}
export function list(value: unknown): unknown[] {
  assert(Array.isArray(value));
  return value;
}
export function string(value: unknown): string {
  assert.equal(typeof value, "string");
  return value as string;
}
export function integer(value: unknown): bigint {
  assert.equal(typeof value, "bigint", "expected a JSON integer token");
  return value as bigint;
}
export function positive(value: unknown): bigint {
  const result = integer(value);
  assert(result > 0n);
  return result;
}
export function parseExact(source: string): unknown {
  return JSON.parse(source, (_key: string, value: unknown, context?: { source?: string }) => {
    if (typeof value !== "number") return value;
    assert(context?.source != null, "the evidence reader requires numeric source context");
    if (/^-?\d+$/.test(context.source)) return BigInt(context.source);
    assert(Number.isFinite(value));
    return value;
  });
}
export function readExact(file: string): unknown {
  return parseExact(fs.readFileSync(file, "utf8"));
}
export function render(value: unknown, pretty = false): string {
  return JSON.stringify(
    value,
    (_key, item: unknown) => {
      if (typeof item !== "bigint") return item;
      const safe = BigInt(Number.MAX_SAFE_INTEGER);
      return item >= -safe && item <= safe ? Number(item) : item.toString();
    },
    pretty ? 2 : undefined,
  );
}
export function writeExact(file: string, value: unknown): void {
  fs.writeFileSync(file, render(value, true) + "\n");
}
export function digest(file: string): string {
  return createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}
export function isMain(url: string): boolean {
  return process.argv[1] != null && pathToFileURL(path.resolve(process.argv[1])).href === url;
}
export function checkExactJson(): void {
  assert.equal(parseExact("9007199254740993"), 9007199254740993n);
  assert.notEqual(parseExact("9007199254740992"), parseExact("9007199254740993"));
  assert.equal(parseExact("18446744073709551615"), 18446744073709551615n);
  assert.equal(parseExact("1.0"), 1);
  assert.throws(() => positive(parseExact("1.0")));
  assert.throws(() => positive(true));
  assert.equal(render({ exact: 18446744073709551615n }), '{"exact":"18446744073709551615"}');
}
if (isMain(import.meta.url)) {
  checkExactJson();
  console.log("exact JSON integer controls passed");
}
