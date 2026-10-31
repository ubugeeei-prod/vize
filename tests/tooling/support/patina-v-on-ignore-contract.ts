import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";

export const root = fileURLToPath(new URL("../../../", import.meta.url));
export const directory = path.join(
  root,
  "tests/_fixtures/differential/linter/v-on-event-ignore-6097",
);
export const rule = "vue/v-on-event-hyphenation";
export const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
export type Policy = "always" | "never" | { style?: "always" | "never"; ignore?: string[] } | null;
export interface IgnoreCase {
  id: string;
  file: string;
  filename: string;
  policy: Policy;
  sourceSha256: string;
  expectedCount: number;
  reference: { file: string; sha256: string };
}
export interface InvalidCase extends Omit<IgnoreCase, "policy" | "expectedCount"> {
  policy: unknown;
}
export interface Finding {
  ruleId: string;
  severity: number;
  message: string;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  messageId: string;
}
export interface Reference {
  messages: Finding[];
  fixResult: { fixed: boolean; messages: Finding[]; output: string };
  requery: Finding[];
}
interface Corpus {
  schema: string;
  version: number;
  provider: Record<string, string>;
  ruleSource: { path: string; sha256: string };
  original: { file: string; sha256: string; bytes: number };
  files: { file: string; sha256: string; bytes: number }[];
  valid: IgnoreCase[];
  invalid: InvalidCase[];
}
const bytes = fs.readFileSync(path.join(directory, "cases.json"));
assert.equal(hash(bytes), "6896d69d431c71e762e493891f777e2ad62f68eac687d5ad1960f9145a1df285");
export const corpus = JSON.parse(bytes.toString()) as Corpus;
assert.equal(corpus.schema, "vize.patina.v-on-event-ignore-6097");
assert.equal(corpus.version, 1);
assert.equal(corpus.valid.length, 13);
assert.equal(corpus.invalid.length, 10);
assert.equal(new Set([...corpus.valid, ...corpus.invalid].map((row) => row.id)).size, 23);
assert.equal(
  corpus.original.sha256,
  "967b8c09becc87143749ae3d20fab2bbfe4dced301e02b77f9243e907ad95ed5",
);
assert.equal(
  corpus.original.file,
  "tests/_fixtures/differential/linter/v-on-hyphenation-custom-tags-10822/ColorPalette.vue.txt",
);
assert.deepEqual(
  fs.readFileSync(path.join(root, corpus.original.file)),
  fs.readFileSync(path.join(directory, "ColorPalette.vue.txt")),
);

export function sourceFor(row: IgnoreCase | InvalidCase) {
  const pin = corpus.files.find((file) => file.file === row.file);
  assert.ok(pin && ["ColorPalette.vue.txt", "Never.vue.txt"].includes(pin.file));
  const bytes = fs.readFileSync(path.join(directory, row.file));
  assert.equal(bytes.length, pin.bytes);
  assert.equal(hash(bytes), pin.sha256);
  assert.equal(hash(bytes), row.sourceSha256);
  assert.equal(row.filename, row.file.replace(/\.txt$/, ""));
  return bytes.toString();
}
export function referenceFor(row: IgnoreCase): Reference {
  return referenceBytes(row) as Reference;
}
export function referenceBytes(row: IgnoreCase | InvalidCase): unknown {
  assert.equal(row.reference.file, "reference/" + row.id + ".json");
  assert.match(row.id, /^[a-z0-9-]+$/);
  const bytes = fs.readFileSync(path.join(directory, row.reference.file));
  assert.equal(hash(bytes), row.reference.sha256);
  return JSON.parse(bytes.toString()) as unknown;
}
export function referenceOptions(policy: unknown): unknown[] {
  if (policy === null) return [];
  if (typeof policy === "string") return [policy];
  assert.ok(typeof policy === "object" && !Array.isArray(policy));
  const record = policy as Record<string, unknown>;
  const { style: _style, ...payload } = record;
  return [Object.hasOwn(record, "style") ? record.style : "always", payload];
}
function position(source: string, line: number, column: number) {
  const prefix = source
    .split("\n")
    .slice(0, line - 1)
    .join("\n");
  const characters = prefix.length + (line > 1 ? 1 : 0) + column - 1;
  return { line, column, offset: Buffer.byteLength(source.slice(0, characters)) };
}
const presentation: Record<string, { message: string; help: string }> = {
  "v-on event '@update:modelValue' must be hyphenated.": {
    message: "Custom event listeners on components should be hyphenated: use 'update:model-value'",
    help: "Rename the listener to kebab-case so it matches the emitted event name.",
  },
  "v-on event '@rowClick' must be hyphenated.": {
    message: "Custom event listeners on components should be hyphenated: use 'row-click'",
    help: "Rename the listener to kebab-case so it matches the emitted event name.",
  },
  "v-on event '@close-window' can't be hyphenated.": {
    message: "Custom event listeners on components should not be hyphenated: 'close-window'",
    help: "Use camelCase or a single-word event name instead of kebab-case.",
  },
  "v-on event '@update:model-value' can't be hyphenated.": {
    message: "Custom event listeners on components should not be hyphenated: 'update:model-value'",
    help: "Use camelCase or a single-word event name instead of kebab-case.",
  },
};
export function expected(row: IgnoreCase, includeFixes = true) {
  const source = sourceFor(row);
  const reference = referenceFor(row);
  assert.equal(reference.messages.length, row.expectedCount);
  assert.equal(reference.fixResult.fixed, false);
  assert.equal(reference.fixResult.output, source);
  assert.deepEqual(reference.fixResult.messages, reference.messages);
  assert.deepEqual(reference.requery, reference.messages);
  const diagnostics = reference.messages.map((finding) => {
    assert.equal(finding.ruleId, rule);
    assert.equal(finding.severity, 1);
    const text = presentation[finding.message];
    assert.ok(text, "only the four independently pinned authored event names are projected");
    return {
      rule,
      severity: "warning",
      message: text.message,
      location: {
        start: position(source, finding.line, finding.column),
        end: position(source, finding.endLine, finding.endColumn),
      },
      help: text.help,
      ...(includeFixes ? { fix: null } : {}),
    };
  });
  return { filename: row.filename, errorCount: 0, warningCount: diagnostics.length, diagnostics };
}
export function clean(row: IgnoreCase) {
  return { filename: row.filename, errorCount: 0, warningCount: 0, diagnostics: [] };
}
export function rowFor(id: string): IgnoreCase {
  const row = corpus.valid.find((row) => row.id === id);
  assert.ok(row);
  return row;
}
