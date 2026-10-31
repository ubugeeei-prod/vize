import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const root = fileURLToPath(new URL("../../../", import.meta.url));
export const directory = path.join(
  root,
  "tests/_fixtures/differential/linter/define-expose-last-6097",
);
export const rule = "script/define-macros-order";
export const filename = "MacroExpose.vue";
export const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
export interface ExposeCase {
  id: string;
  file: string;
  options: { order?: string[]; defineExposeLast?: boolean };
  ordinaryScript?: boolean;
  sourceSha256: string;
  statements: string[];
  reference: {
    messages: { message: string; [key: string]: unknown }[];
    fixResult: { fixed: boolean; output: string; messages: unknown[] };
  };
}
interface Corpus {
  schema: string;
  version: number;
  ruleSource: { path: string; sha256: string };
  files: { file: string; bytes: number; sha256: string }[];
  cases: ExposeCase[];
}
export const corpus = JSON.parse(
  fs.readFileSync(path.join(directory, "cases.json"), "utf8"),
) as Corpus;
assert.equal(corpus.schema, "vize.patina.define-expose-last-6097");
assert.equal(corpus.version, 1);
assert.equal(
  corpus.ruleSource.sha256,
  "9b9aaf5b52c2e05b67597bb1c859fb10205e86ebb4e8878487a51bb0c3abfd66",
);
assert.deepEqual(
  corpus.cases.map((row) => row.id),
  [
    "expose-before-true",
    "expose-before-default",
    "expose-before-false",
    "expose-last-repaired",
    "expose-and-order-true",
    "expose-and-order-default",
    "ordinary-script-control",
  ],
);

export function sourceFor(row: ExposeCase): string {
  const pin = corpus.files.find((file) => file.file === row.file);
  assert.ok(
    pin &&
      ["ExposeBefore.vue.txt", "ExposeLast.vue.txt", "ExposeAndOrder.vue.txt"].includes(row.file),
  );
  const bytes = fs.readFileSync(path.join(directory, row.file));
  assert.equal(bytes.length, pin.bytes);
  assert.equal(hash(bytes), pin.sha256);
  const source = row.ordinaryScript
    ? bytes.toString().replace("<script setup", "<script")
    : bytes.toString();
  assert.equal(hash(source), row.sourceSha256);
  return source;
}

function position(source: string, character: number) {
  const lines = source.slice(0, character).split("\n");
  return {
    line: lines.length,
    column: lines.at(-1)!.length + 1,
    offset: Buffer.byteLength(source.slice(0, character)),
  };
}

export function expected(row: ExposeCase) {
  const source = sourceFor(row);
  assert.equal(row.reference.messages.length, row.statements.length);
  const diagnostics = row.statements.map((statement, index) => {
    const start = source.indexOf(statement);
    assert.ok(start >= 0 && source.indexOf(statement, start + 1) < 0);
    const expose = statement.startsWith("defineExpose");
    const message = expose
      ? "`defineExpose` should be the last statement in `<script setup>`."
      : "`defineProps` should be above `defineEmits`.";
    assert.equal(
      row.reference.messages[index]!.message,
      expose ? message : "defineProps should be above defineEmits.",
    );
    return {
      rule,
      severity: "warning",
      message,
      location: { start: position(source, start), end: position(source, start + statement.length) },
      help: expose
        ? "Put `defineExpose` as the last statement in `<script setup>`."
        : "Declare `defineProps` and then `defineEmits` first in `<script setup>`, right after imports and type declarations.",
      fix: null,
    };
  });
  return { filename, errorCount: 0, warningCount: diagnostics.length, diagnostics };
}
