/** Whole-packet and native-diagnostic admission for the generated constructor header. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { isRecord, jsonValue } from "./canon-instance-directives-model.ts";
import { save } from "./canon-instance-directives-io.ts";
export type Row = { file: string; line: number; column: number; code: number; message: string };
type Tuple = [number, number, number, number, number, number];
function tuples(value: unknown): Tuple[] {
  assert.ok(Array.isArray(value));
  return value.map((row: unknown) => {
    assert.ok(
      Array.isArray(row) &&
        row.length === 6 &&
        row.every((n: unknown) => typeof n === "number" && Number.isSafeInteger(n) && n >= 0),
    );
    const [a, b, c, d, e, f] = row;
    assert.ok(
      typeof a === "number" &&
        typeof b === "number" &&
        typeof c === "number" &&
        typeof d === "number" &&
        typeof e === "number" &&
        typeof f === "number",
    );
    return [a, b, c, d, e, f];
  });
}
function directives(result: Record<string, unknown>): Tuple[] {
  if (result.diagnosticDirectives === undefined) return [];
  assert.ok(isRecord(result.diagnosticDirectives));
  return tuples(result.diagnosticDirectives.directives);
}
export function native(stdout: Buffer, status: number | null, project: string): Row[] {
  const source = new TextDecoder("utf8", { fatal: true }).decode(stdout),
    rows: Row[] = [];
  for (const match of source.matchAll(/^(.+?)\((\d+),(\d+)\): error TS(\d+): (.*)$/gmu)) {
    const [, file, line, column, code, message] = match;
    assert.ok(file && line && column && code && message !== undefined);
    rows.push({
      file: path.relative(project, path.resolve(project, file)).replaceAll("\\", "/"),
      line: +line,
      column: +column,
      code: +code,
      message,
    });
  }
  assert.equal(rows.length, (source.match(/error TS\d+:/gu) ?? []).length);
  assert.equal(status, rows.length ? 2 : 0);
  return rows;
}
export function sdk(project: string, provider: string, binary: string) {
  const modules = path.join(project, "node_modules");
  fs.mkdirSync(modules);
  for (const name of fs.readdirSync(path.join(provider, "node_modules")))
    if (name !== "vize")
      fs.symlinkSync(path.join(provider, "node_modules", name), path.join(modules, name), "dir");
  const source = path.join(provider, "node_modules/vize"),
    target = path.join(modules, "vize");
  fs.mkdirSync(target);
  for (const name of fs.readdirSync(source))
    if (name !== "package.json") fs.symlinkSync(path.join(source, name), path.join(target, name));
  const manifest = jsonValue(fs.readFileSync(path.join(source, "package.json"), "utf8"));
  assert.ok(
    isRecord(manifest) &&
      isRecord(manifest.typescript) &&
      isRecord(manifest.typescript.contentMapper),
  );
  manifest.typescript.contentMapper.exec = [binary, "content-mapper"];
  save(path.join(target, "package.json"), manifest);
}
export function selectSdk(project: string, binary: string) {
  const file = path.join(project, "node_modules/vize/package.json"),
    value = jsonValue(fs.readFileSync(file, "utf8"));
  assert.ok(
    isRecord(value) && isRecord(value.typescript) && isRecord(value.typescript.contentMapper),
  );
  value.typescript.contentMapper.exec = [binary, "content-mapper"];
  save(file, value);
}
export function auditPackets(
  before: Record<string, unknown>,
  after: Record<string, unknown>,
  source: string,
) {
  assert.equal(typeof before.text, "string");
  assert.equal(before.text, after.text);
  const text = before.text;
  assert.ok(typeof text === "string");
  const old = directives(before),
    current = directives(after),
    counts = new Map<string, number>();
  for (const row of old)
    counts.set(JSON.stringify(row), (counts.get(JSON.stringify(row)) ?? 0) + 1);
  const retained: Tuple[] = [],
    added: Tuple[] = [];
  for (const row of current) {
    const key = JSON.stringify(row),
      n = counts.get(key) ?? 0;
    if (n) {
      counts.set(key, n - 1);
      retained.push(row);
    } else added.push(row);
  }
  assert.deepEqual(retained, old, "every existing policy tuple stays byte-for-byte and in order");
  assert.ok([...counts.values()].every((n) => n === 0));
  const mappings = tuples(before.mappings),
    bytes = Buffer.from(text),
    authored = Buffer.from(source);
  for (const [start, length, generated, end, policy, unused] of added) {
    assert.deepEqual([end - generated, policy, unused], [1, 0, 0]);
    assert.equal(bytes.subarray(generated, end).toString(), "{");
    const prefix = bytes.subarray(0, generated).toString().split("\n").at(-1)?.trimStart();
    assert.ok(
      prefix &&
        (prefix.startsWith("const __vize_slot_host_") ||
          prefix.startsWith("const __vize_slot_props_host_")) &&
        prefix.includes("__VizeSlotConstructor<") &&
        prefix.endsWith(")("),
    );
    assert.equal(authored.subarray(start, start + length).toString(), "WitnessChild");
    assert.ok(
      mappings.every(([at, size]) => at + size <= generated || end <= at),
      "no authored expression is ignored",
    );
    assert.ok(
      mappings.some(
        ([at, size, original, originalSize]) =>
          size === 1 &&
          original === start &&
          originalSize === length &&
          bytes.subarray(at, at + size).toString() === "{",
      ),
      "actual canonical primary required",
    );
  }
  const cleanBefore = { ...before },
    cleanAfter = { ...after };
  delete cleanBefore.diagnosticDirectives;
  delete cleanAfter.diagnosticDirectives;
  assert.deepEqual(
    cleanAfter,
    cleanBefore,
    "whole VTS/mappings/links/diagnostics/schema fields stay exact",
  );
  return added;
}
export function inHeader(row: Row, text: string, headers: Tuple[]): boolean {
  const lines = text.split("\n"),
    line = lines[row.line - 1];
  if (line === undefined) return false;
  const offset = Buffer.byteLength(
    lines.slice(0, row.line - 1).join("\n") +
      (row.line > 1 ? "\n" : "") +
      line.slice(0, row.column - 1),
  );
  return headers.some((tuple) => tuple[2] <= offset && offset < tuple[3]);
}
