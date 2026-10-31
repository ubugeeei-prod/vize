import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const root = fileURLToPath(new URL("../../../", import.meta.url));
export const inputRoot = path.join(
  root,
  "tests/_fixtures/differential/linter/macro-order-config-6097",
);
export const rule = "script/define-macros-order";
export const filename = "MacroOptions.vue";
export const help =
  "Declare `defineProps` and then `defineEmits` first in `<script setup>`, right after imports and type declarations.";
export const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");

export interface MacroOptions {
  order?: string[];
}
export interface Finding {
  message: string;
  [key: string]: unknown;
}
export interface MacroCase {
  id: string;
  source: string;
  options: MacroOptions | null;
  statement: string | null;
  reference: {
    messages: Finding[];
    fixResult: { fixed: boolean; output: string; messages: Finding[] };
  };
}
export interface Corpus {
  schema: string;
  version: number;
  provider: Record<string, string>;
  ruleSource: { path: string; sha256: string };
  provenance: {
    originalTests: string;
    sources: {
      file: string;
      originalTest: string;
      sourceBytes: number;
      sourceSha256: string;
      originalScriptSha256: string;
    }[];
  };
  cases: MacroCase[];
}
export const corpus = JSON.parse(
  fs.readFileSync(path.join(inputRoot, "cases.json"), "utf8"),
) as Corpus;
assert.equal(corpus.schema, "vize.patina.macro-order-config-6097");
assert.equal(corpus.version, 1);
assert.equal(corpus.cases.length, 6);

export function sourceFor(row: MacroCase): string {
  assert.ok(["OptionsFirst.vue.txt", "EmitsFirst.vue.txt"].includes(row.source));
  const source = fs.readFileSync(path.join(inputRoot, row.source), "utf8");
  const pin = corpus.provenance.sources.find((entry) => entry.file === row.source);
  assert.ok(pin);
  assert.equal(Buffer.byteLength(source), pin.sourceBytes);
  assert.equal(hash(source), pin.sourceSha256);
  return source;
}

function position(source: string, offset: number) {
  const lines = Buffer.from(source).subarray(0, offset).toString().split("\n");
  return { line: lines.length, column: lines.at(-1)!.length + 1, offset };
}

// Complete current public NAPI shape. Rust companions also check its labels,
// which this existing public NAPI format does not expose.
export function nativeExpected(row: MacroCase, includeFixes = true) {
  const source = sourceFor(row);
  const diagnostics = [];
  if (row.statement !== null) {
    assert.equal(row.reference.messages.length, 1);
    const character = source.indexOf(row.statement);
    assert.ok(character >= 0);
    assert.equal(source.indexOf(row.statement, character + 1), -1);
    const start = Buffer.byteLength(source.slice(0, character));
    const end = start + Buffer.byteLength(row.statement);
    diagnostics.push({
      rule,
      severity: "warning",
      message: row.reference.messages[0]!.message,
      location: { start: position(source, start), end: position(source, end) },
      help,
      ...(includeFixes ? { fix: null } : {}),
    });
  } else assert.equal(row.reference.messages.length, 0);
  return { filename, errorCount: 0, warningCount: diagnostics.length, diagnostics };
}

export const cleanExpected = () => ({ filename, errorCount: 0, warningCount: 0, diagnostics: [] });
