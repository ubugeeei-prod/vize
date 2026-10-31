import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { asRecord, asString, parseJson, type JsonRecord } from "../../json-value.ts";
import { sha256 } from "../../config/windows_consumer/common.ts";

export const requiredMerge = "87d4e6a22631aa9d0ca28c9226e3c912e38b628d";
export const fixtureSha256 = "6f0b9e637e9df46d067058a50d84798a4adf813ff5b72fcf9691cf51d1aabe5d";
export const overrideNames = [
  "NODE_OPTIONS",
  "NODE_PATH",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_NATIVE_PATH",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
] as const;
export type Phase = "original-no-provider" | "stable-user-provider" | "reported-tsrs";

export function readOriginal(
  file = new URL(
    "../../../../tests/tooling/fixtures/patina-boolean-public-original.json",
    import.meta.url,
  ),
): JsonRecord {
  const raw = readFileSync(file);
  assert.equal(sha256(raw), fixtureSha256, "whole original issue fixture bytes");
  const original = asRecord(parseJson(raw.toString("utf8")));
  assert.equal(original.issue, 11147);
  return original;
}
export function rejectOverrides(env: NodeJS.ProcessEnv = process.env): void {
  for (const name of overrideNames)
    assert.ok(!env[name], "ambient runtime/source override: " + name);
}
export function assertHost(
  platform = process.platform,
  arch = process.arch,
  node = process.version,
): void {
  assert.equal(platform, "darwin", "original macOS host required");
  assert.equal(arch, "arm64", "original ARM64 host required");
  assert.equal(node, "v26.6.0", "Node26 patch pinned explicitly, not specified by reporter");
}
export function assertReport(original: JsonRecord, phase: Phase, stdout: Buffer): void {
  const expected =
    phase === "original-no-provider"
      ? original.expectedNoProviderReport
      : original.expectedNullableReport;
  assert.deepEqual(
    parseJson(stdout.toString("utf8")),
    expected,
    "complete ordered original report: " + phase,
  );
}
export function originalFiles(original: JsonRecord): readonly (readonly [string, string])[] {
  const files = Object.entries(asRecord(original.files)).map(
    ([name, value]) => [name, asString(value)] as const,
  );
  assert.deepEqual(
    files.map(([name]) => name),
    ["tsconfig.json", "vite.config.ts", "src/card.vue"],
  );
  return files;
}

export interface ProcessRow {
  readonly pid: number;
  readonly ppid: number;
  readonly command: string;
}
export function descendants(raw: string, parent: number): ProcessRow[] {
  assert.ok(Number.isSafeInteger(parent) && parent > 0);
  const rows = raw
    .split("\n")
    .filter((line) => line.trim())
    .map((line) => {
      const row = /^\s*(\d+)\s+(\d+)\s+(.+?)\s*$/u.exec(line);
      assert(row, "bounded ps record is malformed");
      return { pid: Number(row[1]), ppid: Number(row[2]), command: row[3]! };
    });
  assert(rows.length <= 8192, "bounded host process inventory");
  assert.equal(
    new Set(rows.map((row) => row.pid)).size,
    rows.length,
    "duplicate ps process identity",
  );
  assert(rows.every((row) => Number.isSafeInteger(row.pid) && Number.isSafeInteger(row.ppid)));
  const owned = new Set([parent]);
  for (let depth = 0; depth < 16; depth++) {
    const before = owned.size;
    for (const row of rows) if (owned.has(row.ppid)) owned.add(row.pid);
    if (before === owned.size) break;
  }
  return rows.filter((row) => row.pid !== parent && owned.has(row.pid));
}
export function assertTextImage(raw: string, pid: number, path: string): void {
  assert.ok(path.startsWith("/"));
  const lines = raw.trimEnd().split("\n");
  assert.equal(lines[0], "p" + pid, "lsof identity belongs to the observed descendant PID");
  assert.equal(lines.filter((line) => line.startsWith("p")).length, 1);
  assert(
    lines.includes("n" + path),
    "selected loaded text image differs from the installed provider",
  );
}

export function assertJournalPid(raw: Buffer, pid: number | null): void {
  assert.ok(pid !== null && Number.isSafeInteger(pid) && pid > 0);
  const first = raw.toString("utf8").split("\n")[0];
  assert(first);
  assert.equal(
    asRecord(parseJson(first)).pid,
    pid,
    "native journal belongs to this independently captured CLI PID",
  );
}
