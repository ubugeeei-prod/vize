import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
export function object(v: unknown): Record<string, unknown> {
  assert(v !== null && typeof v === "object" && !Array.isArray(v), "object required");
  return v as Record<string, unknown>;
}
export function text(v: unknown): string {
  assert.equal(typeof v, "string");
  return v as string;
}
export function array(v: unknown): unknown[] {
  assert(Array.isArray(v));
  return v;
}
export const sha256 = (v: Uint8Array | string) => createHash("sha256").update(v).digest("hex");
export const gitBlob = (v: Buffer) =>
  createHash("sha1").update(`blob ${v.length}\0`).update(v).digest("hex");
export function hash(v: unknown, length = 64): string {
  const s = text(v);
  assert(new RegExp(`^[a-f0-9]{${length}}$`, "u").test(s));
  return s;
}
export function bytes(v: string): Buffer {
  assert(/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/u.test(v));
  const raw = Buffer.from(v, "base64");
  assert.equal(raw.toString("base64"), v);
  return raw;
}
export function jsonFile(file: string): unknown {
  return JSON.parse(readFileSync(file, "utf8")) as unknown;
}
export function wholeJson(raw: Buffer): unknown {
  return JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(raw)) as unknown;
}
export function sourcePath(p: string): void {
  assert(p && !p.includes("\\") && !p.startsWith("/") && !p.includes("\0"));
  assert(p.split("/").every((s) => s !== "" && s !== "." && s !== ".."));
}
