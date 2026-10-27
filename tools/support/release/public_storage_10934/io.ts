import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
export { asArray, asNumber, asRecord, asString } from "../../json-value.ts";
import { asRecord, asString } from "../../json-value.ts";

export const REPO = "ubugeeei-prod/vize";
export const REQUIRED_MERGE = "b251264b6aad47380ddf0b9f5f913f634c6b3a97";
export const HARNESS_BLOB = "3267f6345739578eeb149c58dccfcf6936cad149";
export const HARNESS_SHA = "64439d48aed048e6efd526af1c60bed1309dc00f9f20697a922db57a1b9d5090";
export function sha256(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}
export function decode(bytes: Uint8Array): string {
  return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
}
export function parse(bytes: Uint8Array): unknown {
  const value: unknown = JSON.parse(decode(bytes));
  return value;
}
export function read(file: string): unknown {
  return parse(fs.readFileSync(file));
}
export function save(file: string, value: unknown): void {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
}
export function env(name: string): string {
  const value = process.env[name];
  assert.ok(value, "missing " + name);
  return value;
}
export async function github(route: string): Promise<unknown> {
  assert.ok(!route.split("/").includes("..") && !route.startsWith("/"));
  const url = "https://api.github.com/repos/" + REPO + "/" + route;
  const response = await fetch(url, {
    headers: {
      Authorization: "Bearer " + env("GH_TOKEN"),
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
    },
  });
  const bytes = new Uint8Array(await response.arrayBuffer());
  assert.equal(response.status, 200, url + ": " + decode(bytes).slice(0, 500));
  return parse(bytes);
}
export async function blob(sha: string): Promise<Buffer> {
  assert.match(sha, /^[a-f0-9]{40}$/);
  const record = asRecord(await github("git/blobs/" + sha));
  assert.equal(record.sha, sha);
  assert.equal(record.encoding, "base64");
  const bytes = Buffer.from(asString(record.content).replace(/\s/g, ""), "base64");
  const actual = createHash("sha1")
    .update(Buffer.from("blob " + bytes.length + "\0"))
    .update(bytes)
    .digest("hex");
  assert.equal(actual, sha);
  return bytes;
}
export function inside(root: string, file: string): string {
  const actual = fs.realpathSync(file);
  assert.ok(actual.startsWith(fs.realpathSync(root) + path.sep), actual);
  return actual;
}
export function run(
  command: string,
  args: string[],
  cwd: string,
  packet: string,
  environment: NodeJS.ProcessEnv = process.env,
  timeout = 480000,
) {
  const result = spawnSync(command, args, {
    cwd,
    env: environment,
    encoding: "buffer",
    timeout,
    maxBuffer: 32 * 1024 * 1024,
  });
  const stdout = result.stdout ?? Buffer.alloc(0);
  const stderr = result.stderr ?? Buffer.alloc(0);
  fs.mkdirSync(path.dirname(packet), { recursive: true });
  fs.writeFileSync(packet + ".stdout", stdout);
  fs.writeFileSync(packet + ".stderr", stderr);
  const record = {
    command: [command, ...args],
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdoutSha256: sha256(stdout),
    stderrSha256: sha256(stderr),
    stdout: decode(stdout),
    stderr: decode(stderr),
  };
  save(packet + ".json", record);
  assert.equal(result.error, undefined, record.error ?? command);
  assert.equal(result.signal, null, command);
  assert.equal(result.status, 0, command + "\n" + record.stdout + "\n" + record.stderr);
  return record;
}
