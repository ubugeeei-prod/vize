import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";

export const root = fileURLToPath(new URL("../../../", import.meta.url));
export const directory = path.join(
  root,
  "tests/_fixtures/differential/linter/attribute-hyphenation-ignore-6097",
);
export const rule = "vue/attribute-hyphenation";
export const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
export type Policy = "always" | "never" | { style?: "always" | "never"; ignore?: string[] };
export interface AttributeCase {
  id: string;
  file: string;
  filename: string;
  referenceRow: number;
  policy: Policy;
  expectedCount: number;
}
export interface Finding {
  ruleId: string;
  severity: number;
  message: string;
  messageId: string;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  fix: { range: [number, number]; text: string };
}
export interface ReferenceResult {
  filePath: string;
  messages: Finding[];
  output?: string;
  errorCount: number;
  warningCount: number;
  fatalErrorCount: number;
}
export interface ReferenceRow {
  id: string;
  file: string;
  source: string;
  options: unknown[];
  verify: ReferenceResult[];
  fix: ReferenceResult[];
  fixedSource: string;
  requery: ReferenceResult[];
}
interface Corpus {
  originalReference: { file: string; sha256: string; bytes: number; sourceRoot: string };
  provider: Record<string, string>;
  ruleSource: { path: string; sha256: string; bytes: number };
  files: { file: string; bytes: number; sha256: string; original: string }[];
  valid: AttributeCase[];
  invalid: { id: string; ignore: unknown; referenceRow: number }[];
}
const corpusBytes = fs.readFileSync(path.join(directory, "cases.json"));
assert.equal(hash(corpusBytes), "436d6184399349841a522bf0d08958b4f867b197b7a6fe5428d6bbf2216e6ce4");
export const corpus = JSON.parse(corpusBytes.toString()) as Corpus;
const originalBytes = fs.readFileSync(path.join(directory, corpus.originalReference.file));
assert.equal(originalBytes.length, corpus.originalReference.bytes);
assert.equal(
  hash(originalBytes),
  "f284836de666e741f1cb4b3cb6a86bff8614711b36fdd386c1ebcac9b58a833b",
);
assert.equal(hash(originalBytes), corpus.originalReference.sha256);
export const original = JSON.parse(originalBytes.toString()) as {
  rows: ReferenceRow[];
  invalid: { ignore: unknown; error: { name: string; message: string } }[];
};
assert.equal(corpus.valid.length, 9);
assert.equal(corpus.invalid.length, 6);

export function sourceFor(row: AttributeCase): string {
  const pin = corpus.files.find((item) => item.file === row.file);
  assert(pin);
  const bytes = fs.readFileSync(path.join(directory, row.file));
  assert.equal(bytes.length, pin.bytes);
  assert.equal(hash(bytes), pin.sha256);
  assert.deepEqual(bytes, fs.readFileSync(path.join(root, pin.original)));
  assert.equal(row.filename, row.file.replace(/\.txt$/, ""));
  return bytes.toString();
}
export function referenceFor(row: AttributeCase): ReferenceRow {
  const reference = original.rows[row.referenceRow];
  assert(reference);
  assert.equal(reference.id, row.id);
  assert.equal(reference.file, row.file);
  assert.equal(reference.source, sourceFor(row));
  assert.equal(reference.verify.length, 1);
  assert.equal(reference.fix.length, 1);
  assert.equal(reference.requery.length, 1);
  assert.equal(reference.verify[0]!.messages.length, row.expectedCount);
  assert.deepEqual(reference.requery[0]!.messages, []);
  return reference;
}
export function rowFor(id: string): AttributeCase {
  const row = corpus.valid.find((row) => row.id === id);
  assert(row);
  return row;
}
export function presentation(finding: Finding) {
  assert.equal(finding.ruleId, rule);
  assert.equal(finding.severity, 1);
  if (finding.messageId === "mustBeHyphenated")
    return {
      message: "Attribute should be hyphenated",
      help: "Use kebab-case for attribute names",
    };
  assert.equal(finding.messageId, "cannotBeHyphenated");
  return {
    message: "Attribute should not be hyphenated",
    help: "Use camelCase or a single-word attribute name instead of kebab-case.",
  };
}
function position(source: string, line: number, column: number) {
  const prefix = source
    .split("\n")
    .slice(0, line - 1)
    .join("\n");
  const utf16 = prefix.length + (line > 1 ? 1 : 0) + column - 1;
  return { line, column, offset: Buffer.byteLength(source.slice(0, utf16)) };
}
export function expected(row: AttributeCase, includeFixes = true) {
  const source = sourceFor(row),
    reference = referenceFor(row);
  const diagnostics = reference.verify[0]!.messages.map((finding) => {
    const text = presentation(finding),
      [start, end] = finding.fix.range;
    const key = source.slice(start, end);
    // ESLint edits the complete key. Vize's existing API edits only the authored name.
    const prefix = key.startsWith(":") ? ":" : key.startsWith("v-model:") ? "v-model:" : "";
    assert(finding.fix.text.startsWith(prefix));
    assert(!key.startsWith("v-") || prefix === "v-model:");
    const edit = {
      start: Buffer.byteLength(source.slice(0, start + prefix.length)),
      end: Buffer.byteLength(source.slice(0, end)),
      newText: finding.fix.text.slice(prefix.length),
    };
    return {
      rule,
      severity: "warning",
      message: text.message,
      location: {
        start: position(source, finding.line, finding.column),
        end: position(source, finding.endLine, finding.endColumn),
      },
      help: text.help,
      ...(includeFixes ? { fix: { message: text.help, edits: [edit] } } : {}),
    };
  });
  return { filename: row.filename, errorCount: 0, warningCount: diagnostics.length, diagnostics };
}
export function clean(row: AttributeCase) {
  return { filename: row.filename, errorCount: 0, warningCount: 0, diagnostics: [] };
}
export function applyExpectedEdits(row: AttributeCase): string {
  const source = Buffer.from(sourceFor(row));
  const edits = expected(row)
    .diagnostics.flatMap((d) => d.fix?.edits ?? [])
    .sort((a, b) => b.start - a.start);
  let output = source,
    boundary = source.length;
  for (const edit of edits) {
    assert(edit.start <= edit.end && edit.end <= boundary);
    boundary = edit.start;
    output = Buffer.concat([
      output.subarray(0, edit.start),
      Buffer.from(edit.newText),
      output.subarray(edit.end),
    ]);
  }
  const fixed = output.toString();
  assert.equal(fixed, referenceFor(row).fixedSource);
  return fixed;
}
