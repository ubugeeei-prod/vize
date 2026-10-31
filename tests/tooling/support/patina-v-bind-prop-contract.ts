import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const root = fileURLToPath(new URL("../../../", import.meta.url));
export const directory = path.join(
  root,
  "tests/_fixtures/differential/linter/v-bind-prop-longform-6097",
);
export const rule = "vue/v-bind-style";
export const filename = "VBindProp.vue";
export const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
export interface Finding {
  ruleId: string;
  severity: number;
  message: string;
  messageId: "expectedLonghandForProp" | "expectedLonghand" | "unexpectedLonghand";
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  fix: { range: [number, number]; text: string };
}
export interface BindCase {
  id: string;
  file: string;
  style: "shorthand" | "longform" | null;
  sourceSha256: string;
  reference: {
    messages: Finding[];
    fixResult: { fixed: boolean; output: string; messages: Finding[] };
    fixedRequery: Finding[];
    fixedPoint: { fixed: boolean; output: string; messages: Finding[] };
  };
}
interface Corpus {
  schema: string;
  version: number;
  provider: Record<string, string>;
  ruleSource: { path: string; sha256: string };
  originalUnit: { template: string };
  files: { file: string; bytes: number; sha256: string }[];
  cases: BindCase[];
}
const bytes = fs.readFileSync(path.join(directory, "cases.json"));
assert.equal(hash(bytes), "c9b6aa51d99e794802efdb43d27907e54201aa806a6ed0665d0e084b2526b5fa");
export const corpus = JSON.parse(bytes.toString()) as Corpus;
assert.equal(corpus.schema, "vize.patina.v-bind-prop-longform-6097");
assert.equal(corpus.version, 1);
assert.equal(corpus.cases.length, 9);
assert.equal(corpus.originalUnit.template, '<a .title="t"></a>');
assert.equal(new Set(corpus.cases.map((row) => row.id)).size, 9);

export function sourceFor(row: BindCase): string {
  const pin = corpus.files.find((file) => file.file === row.file);
  assert.ok(pin && pin.file === path.basename(pin.file));
  const source = fs.readFileSync(path.join(directory, row.file));
  assert.equal(source.length, pin.bytes);
  assert.equal(hash(source), pin.sha256);
  assert.equal(hash(source), row.sourceSha256);
  return source.toString();
}
function character(source: string, line: number, column: number): number {
  const lines = source.split("\n");
  assert.ok(line >= 1 && line <= lines.length && column >= 1);
  return lines.slice(0, line - 1).reduce((sum, value) => sum + value.length + 1, 0) + column - 1;
}
function position(source: string, line: number, column: number) {
  return {
    line,
    column,
    offset: Buffer.byteLength(source.slice(0, character(source, line, column))),
  };
}
export function expected(row: BindCase, includeFixes = true) {
  const source = sourceFor(row);
  const diagnostics = row.reference.messages.map((finding) => {
    assert.equal(finding.ruleId, rule);
    assert.equal(finding.severity, 1);
    const prop = finding.messageId === "expectedLonghandForProp";
    const short = finding.messageId === "unexpectedLonghand";
    assert.ok(prop || short || finding.messageId === "expectedLonghand");
    if (prop) assert.equal(finding.message, "Expected 'v-bind:' instead of '.'.");
    const start = position(source, finding.line, finding.column);
    // Preserve Vize's existing single-prefix edit for the two older spellings.
    // Their whole repaired bytes still equal the independent reference output.
    const edit = prop
      ? {
          start: Buffer.byteLength(source.slice(0, finding.fix.range[0])),
          end: Buffer.byteLength(source.slice(0, finding.fix.range[1])),
          newText: finding.fix.text,
        }
      : {
          start: start.offset,
          end: start.offset + (short ? 7 : 1),
          newText: short ? ":" : "v-bind:",
        };
    return {
      rule,
      severity: "warning",
      message: prop
        ? finding.message
        : short
          ? "Prefer shorthand `:` over `v-bind:`"
          : "Prefer `v-bind:` over shorthand `:`",
      location: {
        start,
        end: position(source, finding.endLine, finding.endColumn),
      },
      help: prop
        ? 'Use `v-bind:attr.prop="value"` instead of `.attr="value"`'
        : short
          ? 'Use `:attr="value"` instead of `v-bind:attr="value"`'
          : 'Use `v-bind:attr="value"` instead of `:attr="value"`',
      ...(includeFixes
        ? {
            fix: {
              message: short ? "Use shorthand syntax" : "Use longform syntax",
              edits: [
                {
                  ...edit,
                },
              ],
            },
          }
        : {}),
    };
  });
  return { filename, errorCount: 0, warningCount: diagnostics.length, diagnostics };
}
export function clean() {
  return { filename, errorCount: 0, warningCount: 0, diagnostics: [] };
}
export function repaired(source: string, packet: ReturnType<typeof expected>): string {
  const edits = packet.diagnostics.flatMap((diagnostic) => diagnostic.fix?.edits ?? []);
  const bytes = Buffer.from(source);
  let result = bytes;
  let previous = bytes.length;
  for (const edit of edits.sort((a, b) => b.start - a.start)) {
    assert.ok(edit.start <= edit.end && edit.end <= previous);
    result = Buffer.concat([
      result.subarray(0, edit.start),
      Buffer.from(edit.newText),
      result.subarray(edit.end),
    ]);
    previous = edit.start;
  }
  return result.toString();
}
