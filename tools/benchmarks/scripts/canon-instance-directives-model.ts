import assert from "node:assert/strict";
import path from "node:path";
import { TextDecoder } from "node:util";
import type { DirectiveCase } from "./canon-instance-directives-cases.ts";

export type JsonRecord = Record<string, unknown>;
export type CliFile = JsonRecord & {
  file: string;
  diagnostics: string[];
  virtualTs?: string | null;
};
export type CliResult = JsonRecord & { files: CliFile[]; errorCount: number };
export type Diagnostic = {
  file: string;
  severity: "error" | "warning";
  line: number;
  column: number;
  code: number | null;
  message: string;
  raw: string;
};
export type PacketObservation = { result: CliResult; diagnostics: Diagnostic[] };
export type Manifest = JsonRecord & { version: string };
export type NativeRequest = JsonRecord & { executable: string; cwd: string; argv: string[] };
export function isRecord(value: unknown): value is JsonRecord {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
function strings(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((entry: unknown) => typeof entry === "string");
}
function isCliFile(value: unknown): value is CliFile {
  return (
    isRecord(value) &&
    typeof value.file === "string" &&
    strings(value.diagnostics) &&
    (value.virtualTs === undefined ||
      value.virtualTs === null ||
      typeof value.virtualTs === "string")
  );
}
function isCliResult(value: unknown): value is CliResult {
  return (
    isRecord(value) &&
    Array.isArray(value.files) &&
    value.files.every(isCliFile) &&
    typeof value.errorCount === "number" &&
    Number.isInteger(value.errorCount) &&
    value.errorCount >= 0
  );
}
export function jsonValue(text: string): unknown {
  return JSON.parse(text);
}
function isManifest(value: unknown): value is Manifest {
  return isRecord(value) && typeof value.version === "string";
}
export function manifest(value: unknown): Manifest {
  assert.ok(isManifest(value), "invalid package manifest");
  return value;
}
function isRequest(value: unknown): value is NativeRequest {
  return (
    isRecord(value) &&
    typeof value.executable === "string" &&
    typeof value.cwd === "string" &&
    strings(value.argv)
  );
}
export function request(value: unknown): NativeRequest {
  assert.ok(isRequest(value), "invalid native request");
  return value;
}
export function isExecutableFactory(value: unknown): value is { default: () => unknown } {
  return isRecord(value) && typeof value.default === "function";
}
function point(source: string, token: string, last = false): { line: number; column: number } {
  const at = last ? source.lastIndexOf(token) : source.indexOf(token);
  assert.ok(at >= 0, "missing owned token: " + token);
  const lines = source.slice(0, at).split(/\r\n|[\r\n\u2028\u2029]/u),
    tail = lines.at(-1);
  assert.ok(tail !== undefined);
  return { line: lines.length, column: tail.length + 1 };
}
export function packet(
  stdout: Buffer,
  status: number | null,
  directory: string,
): PacketObservation {
  const result = jsonValue(new TextDecoder("utf-8", { fatal: true }).decode(stdout));
  assert.ok(isCliResult(result), "invalid complete CLI JSON schema");
  const diagnostics: Diagnostic[] = result.files.flatMap((file) =>
    file.diagnostics.map((text) => {
      const match = /^(error|warning):(\d+):(\d+) (?:\[TS(\d+)\] )?([\s\S]+)$/u.exec(text);
      assert.ok(match, "unrecognized complete diagnostic: " + text);
      const severity = match[1],
        lineText = match[2],
        columnText = match[3],
        message = match[5];
      assert.ok(lineText !== undefined && columnText !== undefined && message !== undefined);
      assert.ok(severity === "error" || severity === "warning");
      return {
        file: path.relative(directory, path.resolve(directory, file.file)),
        severity,
        line: +lineText,
        column: +columnText,
        code: match[4] ? +match[4] : null,
        message,
        raw: text,
      };
    }),
  );
  assert.equal(
    result.errorCount,
    diagnostics.filter((diagnostic) => diagnostic.severity === "error").length,
  );
  assert.ok(status === 0 || status === 1);
  assert.equal(status === 0, result.errorCount === 0);
  return { result, diagnostics };
}
type Located = { line: number; column: number; code: number | null };
export function assertErrors(row: DirectiveCase, observed: Diagnostic[]): void {
  const expected = row.errors.map(({ code, token, last }) => ({
    file: "main.vue",
    severity: "error",
    code,
    ...point(row.source, token, last),
  }));
  const order = (left: Located, right: Located) =>
    left.line - right.line || left.column - right.column || (left.code ?? 0) - (right.code ?? 0);
  assert.deepEqual(
    observed.map(({ message: _message, raw: _raw, ...rest }) => rest).sort(order),
    expected.sort(order),
    row.id,
  );
  for (const diagnostic of observed) {
    if (diagnostic.code === 2578)
      assert.match(diagnostic.message, /^Unused '@ts-expect-error' directive\./u);
    if (diagnostic.code === 2339)
      assert.match(
        diagnostic.message,
        /^Property '[^']+' does not exist on (?:type |the component instance\.)/u,
      );
    if (diagnostic.code === 2345) assert.match(diagnostic.message, /^Argument of type /u);
  }
}
