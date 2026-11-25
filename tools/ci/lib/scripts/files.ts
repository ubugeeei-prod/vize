import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, statSync, writeFileSync } from "node:fs";

export function requiredEnv(name: string): string {
  const value = process.env[name];
  assert.ok(value !== undefined, `Missing environment variable ${name}`);
  return value;
}

export function textFile(file: string): string {
  return readFileSync(file, "utf8");
}

export function writeJson(file: string, value: unknown): void {
  writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
}

export function fileSha256(file: string): string {
  return createHash("sha256").update(readFileSync(file)).digest("hex");
}

export function jsonLines<T>(file: string): T[] {
  return textFile(file)
    .trimEnd()
    .split("\n")
    .map((line) => JSON.parse(line) as T);
}

export function readJson<T>(file: string): T {
  return JSON.parse(textFile(file)) as T;
}
export function fileIdentity(file: string) {
  return { path: file, bytes: statSync(file).size, sha256: fileSha256(file) };
}
export function environment(names: readonly string[]) {
  return Object.fromEntries(names.map((name) => [name, process.env[name] ?? null]));
}
