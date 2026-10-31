import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { readTarGz } from "../../archive/tar.ts";

export interface Image {
  bytes: number;
  sha256: string;
}
export interface Authority {
  version: string;
  candidate: string;
  producerRun: string;
  nativeIdentity: { kind: "native"; token: string };
  entrypoint: Image;
  packageJson: Image;
  binary: Image & { package: string };
}
export interface Custody {
  merge: string;
  runtimeScope: string;
  pins: Record<string, string>;
  referenceDependencies: Record<string, string>;
}
export const sha256 = (bytes: Uint8Array): string =>
  createHash("sha256").update(bytes).digest("hex");
export const image = (file: string): Image => {
  const bytes = fs.readFileSync(file);
  return { bytes: bytes.length, sha256: sha256(bytes) };
};
export function read<T>(file: string): T {
  return JSON.parse(fs.readFileSync(file, "utf8")) as T;
}
export function write(file: string, value: unknown): void {
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
}
export function request(
  version: string,
  candidate: string,
  producerRun: string,
  env: NodeJS.ProcessEnv,
): void {
  assert.match(version, /^0\.[0-9]+\.0$/);
  assert.match(candidate, /^[0-9a-f]{40}$/);
  assert.match(producerRun, /^[1-9][0-9]*$/);
  assert.equal(env.GITHUB_ACTIONS, "true");
  assert.equal(env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
  assert.equal(env.GITHUB_REF, "refs/heads/main");
  assert.equal(env.GITHUB_EVENT_NAME, "workflow_dispatch");
  assert.equal(
    env.GITHUB_WORKFLOW_REF,
    "ubugeeei-prod/vize/.github/workflows/public-jsx-child-values.yml@refs/heads/main",
  );
  for (const key of [
    "NODE_OPTIONS",
    "NODE_PATH",
    "VIZE_NATIVE_BINDING_PATH",
    "VIZE_PREFER_WORKSPACE_BINDING",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  ])
    assert(!env[key], `${key} must not override the installed public provider`);
}
export function checkSources(root: string, custody: Custody): void {
  for (const [name, expected] of Object.entries(custody.pins))
    assert.equal(image(path.join(root, name)).sha256, expected, name);
}
export function within(root: string, file: string): string {
  const actual = fs.realpathSync(file);
  assert(actual.startsWith(fs.realpathSync(root) + path.sep), actual);
  return actual;
}
export function lockEntry(
  value: unknown,
  name: string,
  version: string,
): {
  version: string;
  resolved: string;
  integrity: string;
} {
  assert(value && typeof value === "object");
  const row = value as Record<string, unknown>;
  assert.equal(row.version, version, name);
  assert.equal(typeof row.resolved, "string");
  const url = new URL(row.resolved as string);
  assert.equal(url.origin, "https://registry.npmjs.org");
  assert(!url.username && !url.password && !url.search && !url.hash);
  assert.equal(url.pathname, `/${name}/-/${name.slice(name.lastIndexOf("/") + 1)}-${version}.tgz`);
  assert.equal(typeof row.integrity, "string");
  assert.match(row.integrity as string, /^sha512-[A-Za-z0-9+/]{86}==$/);
  return row as { version: string; resolved: string; integrity: string };
}
export function cachedArchive(cache: string, integrity: string): Buffer {
  assert.match(integrity, /^sha512-[A-Za-z0-9+/]{86}==$/);
  const digest = Buffer.from(integrity.slice(7), "base64").toString("hex");
  const file = path.join(
    cache,
    "_cacache/content-v2/sha512",
    digest.slice(0, 2),
    digest.slice(2, 4),
    digest.slice(4),
  );
  const bytes = fs.readFileSync(file);
  assert(bytes.length > 0 && bytes.length <= 128 * 1024 * 1024);
  assert.equal("sha512-" + createHash("sha512").update(bytes).digest("base64"), integrity);
  return bytes;
}
/** Compare every archive payload file with the genuine installed package, without extracting it. */
export function validateArchive(
  bytes: Buffer,
  installed: string,
): {
  path: string;
  bytes: number;
  sha256: string;
}[] {
  const members = readTarGz(bytes);
  assert(members.length > 0 && members.length <= 10000);
  const seen = new Set<string>();
  const files = [];
  for (const member of members) {
    if (member.name === "package" && member.kind === "directory") continue;
    assert(member.name.startsWith("package/"));
    const name = member.name.slice(8);
    assert(name && !name.split("/").some((part) => ["", ".", ".."].includes(part)));
    assert(!seen.has(name));
    seen.add(name);
    assert(member.kind === "file" || member.kind === "directory");
    if (member.kind === "directory") continue;
    const actual = within(installed, path.join(installed, name));
    assert(fs.lstatSync(actual).isFile());
    const observed = image(actual);
    assert.deepEqual(observed, { bytes: member.size, sha256: sha256(member.data) }, name);
    files.push({ path: name, ...observed });
  }
  assert(files.some((file) => file.path === "package.json"));
  return files;
}
