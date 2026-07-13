import assert from "node:assert/strict";
import test from "node:test";
import { TraceMap, originalPositionFor } from "@jridgewell/trace-mapping";
import { applySourceProvenance, offsetEmbeddedSourceMap } from "./index.ts";

void test("offsetEmbeddedSourceMap keeps exact generated columns after a host prefix", () => {
  const generated = "const value = 1;";
  const map = {
    version: 3 as const,
    file: "generated.js",
    names: [],
    sources: ["source.vue"],
    sourcesContent: ["<p>value</p>"],
    mappings: "AAAA",
  };
  const host = `// host\n  ${generated}\n// suffix`;
  const relocated = offsetEmbeddedSourceMap(generated, host, map);

  assert.ok(relocated);
  const original = originalPositionFor(new TraceMap(relocated), { line: 2, column: 2 });
  assert.equal(original.source, "source.vue");
  assert.equal(original.line, 1);
  assert.equal(original.column, 0);
});

void test("applySourceProvenance remaps an inlined template anchor to its external file", () => {
  const external = "<p>external</p>";
  const synthetic = `<template>${external}</template>`;
  const map = {
    version: 3 as const,
    file: "component.js",
    names: [],
    sources: ["Component.vue"],
    sourcesContent: [synthetic],
    // Output (0,0) -> synthetic source (0,10), the first external character.
    mappings: "AAAU",
  };
  const remapped = applySourceProvenance(map, synthetic, [
    {
      generatedStart: 10,
      generatedEnd: 10 + external.length,
      source: "/src/template.html",
      sourceContent: external,
      sourceStart: 0,
    },
  ]);

  assert.ok(remapped);
  const original = originalPositionFor(new TraceMap(remapped), { line: 1, column: 0 });
  assert.equal(original.source, "/src/template.html");
  assert.equal(original.line, 1);
  assert.equal(original.column, 0);
  assert.deepEqual(remapped.sourcesContent, [external]);
});
