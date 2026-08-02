import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export function canonicalJson(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value != null && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .filter((key) => (value as Record<string, unknown>)[key] !== undefined)
      .map(
        (key) => `${JSON.stringify(key)}:${canonicalJson((value as Record<string, unknown>)[key])}`,
      )
      .join(",")}}`;
  }
  const serialized = JSON.stringify(value);
  if (serialized === undefined) throw new TypeError("value is not JSON-serializable");
  return serialized;
}

export function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

export function byteOrder(left: string, right: string): number {
  return Buffer.from(left).compare(Buffer.from(right));
}

export function assertNormalizedPath(value: unknown, label = "path"): asserts value is string {
  assert.ok(
    typeof value === "string" &&
      value.length > 0 &&
      !value.startsWith("/") &&
      !value.includes("\\") &&
      !value.split("/").some((part) => part === "" || part === "." || part === ".."),
    `invalid ${label} ${String(value)}`,
  );
}
