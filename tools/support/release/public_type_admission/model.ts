import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";
export type RecordValue = Record<string, unknown>;
export type Expected = { file: string; line: number; column: number; code: number };
export type Finding = Expected & { severity: "error" | "warning"; message: string; raw: string };
export type NativeImage = { path: string; bytes: number; sha256: string };
export type MapperHost = {
  workspace: string;
  backend: string;
  version: string;
  sha256: string;
  lockSha256: string;
  packagePins: RecordValue[];
  setupReceipt: RecordValue;
  probes: unknown[];
  route: string;
  projectedPackageGraph: string;
  sriScope: string;
  runtimeCapability: string;
};
export type Runtime = {
  workspace: string;
  version: string;
  candidate: string;
  cli: string;
  backend: string;
  backendSha256: string;
  mapperHost: MapperHost;
  node: string;
  native: NativeImage;
  observer: string;
  output: string;
};
export type Packet = { result: RecordValue; diagnostics: Finding[] };
function isRecord(value: unknown): value is RecordValue {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
export function record(value: unknown): RecordValue {
  assert.ok(isRecord(value), "expected JSON object");
  return value;
}
export function array(value: unknown): unknown[] {
  assert.ok(Array.isArray(value));
  return value;
}
export function string(value: unknown): string {
  assert.equal(typeof value, "string");
  assert.ok(typeof value === "string");
  return value;
}
export function integer(value: unknown): number {
  assert.ok(typeof value === "number" && Number.isSafeInteger(value));
  return value;
}
export function parse(bytes: Uint8Array): unknown {
  return JSON.parse(new TextDecoder("utf8", { fatal: true }).decode(bytes));
}
export function hash(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}
export function write(file: string, value: unknown): void {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
}
export function equalFindings(
  actual: Finding[],
  expected: readonly Expected[],
  label: string,
): void {
  const key = (row: Expected) => JSON.stringify([row.file, row.line, row.column, row.code]);
  const sort = (a: Expected, b: Expected) => key(a).localeCompare(key(b));
  assert.deepEqual(
    actual.map(({ file, line, column, code }) => ({ file, line, column, code })).sort(sort),
    [...expected].sort(sort),
    label,
  );
  assert.equal(actual.length, expected.length, label + " multiplicity");
  for (const row of actual) {
    assert.equal(row.severity, "error");
    assert.ok(row.message.length > 0);
  }
}
export function cliPacket(stdout: Buffer, status: number | null, directory: string): Packet {
  const result = record(parse(stdout));
  const diagnostics: Finding[] = [];
  for (const item of array(result.files)) {
    const file = record(item);
    const name = string(file.file);
    assert.ok(
      file.virtualTs === undefined || file.virtualTs === null || typeof file.virtualTs === "string",
    );
    for (const value of array(file.diagnostics)) {
      const raw = string(value);
      const match = /^(error|warning):(\d+):(\d+) \[TS(\d+)\] ([\s\S]+)$/u.exec(raw);
      assert.ok(match, "unclassified complete diagnostic: " + raw);
      const [, severity, line, column, code, message] = match;
      assert.ok(severity === "error" || severity === "warning");
      assert.ok(line && column && code && message);
      diagnostics.push({
        file: path.relative(directory, path.resolve(directory, name)).replaceAll("\\", "/"),
        severity,
        line: Number(line),
        column: Number(column),
        code: Number(code),
        message,
        raw,
      });
    }
  }
  assert.equal(
    integer(result.errorCount),
    diagnostics.filter((row) => row.severity === "error").length,
  );
  if (result.warningCount !== undefined)
    assert.equal(
      integer(result.warningCount),
      diagnostics.filter((row) => row.severity === "warning").length,
    );
  assert.ok(status === 0 || status === 1);
  assert.equal(status === 0, result.errorCount === 0);
  return { result, diagnostics };
}
export function mapperPacket(
  stdout: Buffer,
  stderr: Buffer,
  status: number | null,
  directory: string,
): Finding[] {
  const output = new TextDecoder("utf8", { fatal: true }).decode(stdout);
  assert.equal(
    new TextDecoder("utf8", { fatal: true }).decode(stderr),
    "",
    "unclassified mapper stderr",
  );
  const diagnostics: Finding[] = [];
  for (const raw of output.split(/\r?\n/u)) {
    if (!raw) continue;
    const m = /^(.+)\((\d+),(\d+)\): (error|warning) TS(\d+): ([\s\S]+)$/u.exec(raw);
    assert.ok(m, "unclassified complete native checker output: " + raw);
    const [, file, line, column, severity, code, message] = m;
    assert.ok(file && line && column && code && message);
    assert.ok(severity === "error" || severity === "warning");
    diagnostics.push({
      file: path.relative(directory, path.resolve(directory, file)).replaceAll("\\", "/"),
      line: Number(line),
      column: Number(column),
      code: Number(code),
      severity,
      message,
      raw,
    });
  }
  assert.ok(status === 0 || status === 2);
  assert.equal(status === 0, diagnostics.filter((row) => row.severity === "error").length === 0);
  return diagnostics;
}
