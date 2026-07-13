import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { TraceMap, originalPositionFor } from "@jridgewell/trace-mapping";
import { compileBatch, compileFile } from "./compiler.ts";

const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-map-"));
const file = path.join(root, "App.vue");
const template = path.join(root, "template.html");
const templateSource = `<p>{{ label }}</p>`;
fs.writeFileSync(template, templateSource);

const source = `<script>
export default { data: () => ({ label: "ready" }) }
</script>
<template src="./template.html"></template>`;
const options = { sourceMap: true, ssr: false, vapor: false };
const compiled = compileFile(file, new Map(), options, source);

assert.ok(compiled.map, "single-file compilation should surface a composed map");
const generatedAnchor = ["$data.label", "$setup.label", "_ctx.label"]
  .map((text) => ({ text, index: compiled.code.lastIndexOf(text) }))
  .find((candidate) => candidate.index >= 0);
assert.ok(generatedAnchor, "compiled render code should contain the template expression");
const generatedOffset = generatedAnchor.index + generatedAnchor.text.lastIndexOf("label");
const generated = lineColumnAt(compiled.code, generatedOffset);
const original = originalPositionFor(new TraceMap(compiled.map), {
  line: generated.line + 1,
  column: generated.column,
});
assert.equal(original.source, template);
assert.equal(original.line, 1);
assert.equal(original.column, templateSource.indexOf("label"));

const cache = new Map();
const batch = compileBatch([{ path: file, source }], cache, options);
assert.equal(batch.failedCount, 0);
assert.ok(cache.get(file)?.map?.sources.includes(template));

function lineColumnAt(value: string, offset: number): { line: number; column: number } {
  const prefix = value.slice(0, offset);
  const line = prefix.split("\n").length - 1;
  const newline = prefix.lastIndexOf("\n");
  return { line, column: prefix.length - newline - 1 };
}
