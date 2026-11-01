import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { presentation, type ReferenceResult } from "./patina-attribute-ignore-contract.ts";
import {
  directory,
  hash,
  inputs,
  inputsBytes,
  root,
  sourceFor,
  type TagCase,
} from "./patina-attribute-ignore-tags-reference.ts";

export { directory, hash, inputs, root, sourceFor, type TagCase };
export const rule = "vue/attribute-hyphenation";
export interface ReferenceTagRow extends TagCase {
  source: string;
  sourceSha256: string;
  verify: ReferenceResult[];
  fix: ReferenceResult[];
  fixedSource: string;
  requery: ReferenceResult[];
}
interface Original {
  schema: string;
  sourceRoot: string;
  inputsSha256: string;
  rows: ReferenceTagRow[];
}
const referenceBytes = fs.readFileSync(path.join(directory, "official.original.json"));
assert.equal(referenceBytes.length, 89043);
assert.equal(
  hash(referenceBytes),
  "e930f5d35f6fd9b4044f4ae19ad03947367a2cca6c60cd2c7064e0dbc551e3f9",
);
export const original = JSON.parse(referenceBytes.toString()) as Original;
assert.equal(original.schema, "vize.attribute-ignore-tags.reference.v1");
assert.equal(original.inputsSha256, hash(inputsBytes));
const processBytes = fs.readFileSync(path.join(directory, "reference-process.original.json"));
assert.equal(
  hash(processBytes),
  "352cc00f7c621fedafed977c4acb38443cb7bc27984630d18b7efbb4971c2f9f",
);
const process = JSON.parse(processBytes.toString()) as {
  head: string;
  run: string;
  attempt: string;
  status: number;
  signal: unknown;
  error: unknown;
  stdout: string;
  stderr: string;
  producerSha256: string;
  inputsSha256: string;
};
assert.equal(process.head, "23226010068c284a2a05770aec458396b62d3f30");
assert.equal(process.run, "41276234481");
assert.equal(process.attempt, "1");
assert.equal(process.status, 0);
assert.equal(process.signal, null);
assert.equal(process.error, null);
assert.equal(process.stderr, "");
assert.deepEqual(Buffer.from(process.stdout), referenceBytes);
assert.equal(process.inputsSha256, hash(inputsBytes));
assert.equal(
  process.producerSha256,
  hash(fs.readFileSync(new URL("./patina-attribute-ignore-tags-reference.ts", import.meta.url))),
);
assert.equal(original.rows.length, 18);
for (const [index, row] of original.rows.entries()) {
  const input = inputs.rows[index];
  assert(input);
  for (const [key, value] of Object.entries(input))
    assert.deepEqual(row[key as keyof TagCase], value);
  assert.equal(row.source, sourceFor(row));
  assert.equal(row.sourceSha256, hash(row.source));
  for (const phase of [row.verify, row.fix, row.requery]) {
    assert.equal(phase.length, 1);
    assert.equal(phase[0]!.filePath, path.join(original.sourceRoot, row.filename));
    assert.equal(phase[0]!.errorCount, 0);
    assert.equal(phase[0]!.fatalErrorCount, 0);
  }
  assert.deepEqual(row.requery[0]!.messages, []);
  assert.equal(row.fixedSource, row.fix[0]!.output ?? row.source);
}
export const literalRows = original.rows.filter((row) => row.scope === "literal");
assert.equal(literalRows.length, 16);
assert.equal(original.rows.filter((row) => row.scope === "upstream-regexp-only").length, 2);
export function rowFor(id: string): ReferenceTagRow {
  const row = literalRows.find((row) => row.id === id);
  assert(row);
  return row;
}
function position(source: string, line: number, column: number) {
  const prefix = source
    .split("\n")
    .slice(0, line - 1)
    .join("\n");
  const utf16 = prefix.length + (line > 1 ? 1 : 0) + column - 1;
  return { line, column, offset: Buffer.byteLength(source.slice(0, utf16)) };
}
// The established public API fixes the authored name, preserving : / v-model:.
// Convert independent ESLint UTF-16 edits to that API's UTF-8 byte positions;
// compare complete target DTOs and complete fixed source, without altering them.
export function expected(row: ReferenceTagRow, includeFixes = true) {
  const diagnostics = row.verify[0]!.messages.map((finding) => {
    const text = presentation(finding),
      [start, end] = finding.fix.range;
    const key = row.source.slice(start, end);
    const prefix = key.startsWith(":") ? ":" : key.startsWith("v-model:") ? "v-model:" : "";
    assert(finding.fix.text.startsWith(prefix));
    assert(!key.startsWith("v-") || prefix === "v-model:");
    const edit = {
      start: Buffer.byteLength(row.source.slice(0, start + prefix.length)),
      end: Buffer.byteLength(row.source.slice(0, end)),
      newText: finding.fix.text.slice(prefix.length),
    };
    return {
      rule,
      severity: "warning",
      message: text.message,
      location: {
        start: position(row.source, finding.line, finding.column),
        end: position(row.source, finding.endLine, finding.endColumn),
      },
      help: text.help,
      ...(includeFixes ? { fix: { message: text.help, edits: [edit] } } : {}),
    };
  });
  return { filename: row.filename, errorCount: 0, warningCount: diagnostics.length, diagnostics };
}
export function clean(row: ReferenceTagRow) {
  return { filename: row.filename, errorCount: 0, warningCount: 0, diagnostics: [] };
}
