import assert from "node:assert/strict";
import { strictJson } from "./verify-json.ts";

export type ObjectValue = Record<string, unknown>;
function isObject(value: unknown): value is ObjectValue {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
export function object(value: unknown): ObjectValue {
  assert.ok(isObject(value), "expected JSON object");
  return value;
}
export function array(value: unknown): unknown[] {
  assert.ok(Array.isArray(value), "expected JSON array");
  return value;
}
export function text(value: unknown): string {
  assert.equal(typeof value, "string");
  if (typeof value !== "string") throw new Error("expected string");
  return value;
}
export function integer(value: unknown): number {
  assert.ok(typeof value === "number" && Number.isSafeInteger(value), "expected integer");
  if (typeof value !== "number") throw new Error("expected number");
  return value;
}
export function first<T>(values: T[]): T {
  const result = values[0];
  if (result === undefined) throw new Error("missing first value");
  return result;
}
export function canonical(value: unknown): string {
  if (
    value === null ||
    typeof value === "boolean" ||
    typeof value === "number" ||
    typeof value === "string"
  )
    return JSON.stringify(value);
  if (Array.isArray(value))
    return "[" + value.map((entry: unknown) => canonical(entry)).join(",") + "]";
  const record = object(value);
  return (
    "{" +
    Object.keys(record)
      .sort()
      .map((key) => JSON.stringify(key) + ":" + canonical(record[key]))
      .join(",") +
    "}"
  );
}
export function frames(data: Buffer): ObjectValue[] {
  let offset = 0;
  const packets: ObjectValue[] = [];
  const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
  while (offset < data.length) {
    const end = data.indexOf("\r\n\r\n", offset, "ascii");
    assert.ok(end >= offset, "truncated raw header");
    const raw = data.subarray(offset, end);
    assert.ok(
      raw.every((byte) => byte < 128),
      "header must be ASCII",
    );
    const lengths = raw
      .toString("ascii")
      .split("\r\n")
      .filter((line) => line.toLowerCase().startsWith("content-length:"))
      .map((line) => line.slice(line.indexOf(":") + 1).trim());
    assert.equal(lengths.length, 1);
    const token = first(lengths);
    assert.match(token, /^[0-9]+$/u);
    const length = Number(token);
    assert.ok(Number.isSafeInteger(length));
    const start = end + 4;
    assert.ok(start + length <= data.length, "truncated raw body");
    packets.push(object(strictJson(decoder.decode(data.subarray(start, start + length)))));
    offset = start + length;
  }
  assert.equal(offset, data.length);
  return packets;
}
