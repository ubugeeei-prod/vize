import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { join, resolve } from "node:path";
import { outputDirectory } from "./pins.ts";

export const sha256 = (bytes: Uint8Array | string) =>
  createHash("sha256").update(bytes).digest("hex");
export const git = (root: string, ...args: string[]) => execFileSync("git", ["-C", root, ...args]);
export function source(root: string, expected: string) {
  assert.match(expected, /^[0-9a-f]{40}$/u, "Require the literal source OID");
  assert.equal(
    git(root, "rev-parse", "HEAD").toString().trim(),
    expected,
    "Checkout source differs",
  );
}
export function directory(root: string) {
  const path = join(resolve(root), outputDirectory);
  mkdirSync(path, { recursive: true });
  return path;
}
export function save(root: string, name: string, value: unknown) {
  writeFileSync(join(directory(root), name), JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
}
export function readJson(path: string): any {
  return JSON.parse(readFileSync(path, "utf8"));
}
export function cliArgs() {
  const [mode, root, expected] = process.argv.slice(2);
  assert.ok(mode && root && expected, "Require mode, checkout root and literal source OID");
  return { mode, root: resolve(root), expected };
}
export function existingJson(root: string, name: string) {
  const path = join(directory(root), name);
  return existsSync(path) ? readJson(path) : null;
}

export const isMain = (url: string) =>
  Boolean(process.argv[1] && url === pathToFileURL(process.argv[1]).href);
