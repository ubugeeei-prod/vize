import assert from "node:assert/strict";
import { test } from "node:test";
import { sources } from "@rspack/core";
import { originalPositionFor, TraceMap } from "@jridgewell/trace-mapping";
import { MappedModule, parseSourceMap, type SourceMapV3 } from "./source-map.ts";

function trace(code: string, map: SourceMapV3, marker: string) {
  const index = code.indexOf(marker);
  assert.notEqual(index, -1);
  const prefix = code.slice(0, index);
  return originalPositionFor(new TraceMap(JSON.stringify(map)), {
    line: prefix.split("\n").length,
    column: index - prefix.lastIndexOf("\n") - 1,
  });
}

void test("parses native source map JSON at the loader boundary", () => {
  for (const json of [undefined, "{", "null", "42", '{"version":2}']) {
    assert.equal(parseSourceMap(json), null);
  }
  assert.equal(
    parseSourceMap(JSON.stringify({ version: 3, sources: [42], names: [], mappings: "AAAA" })),
    null,
  );
  const map = { version: 3, sources: ["App.vue"], names: [], mappings: "AAAA" };
  assert.deepEqual(parseSourceMap(JSON.stringify(map)), map);
});

void test("default export rewrites move mapped columns on the same line", () => {
  const mapped = new MappedModule("export default { answer: 42 };", {
    version: 3,
    sources: ["App.vue"],
    names: [],
    // The opening brace is at generated column 15 and original column 0.
    mappings: "eAAA",
  });
  mapped.edit("const _sfc_main = { answer: 42 };");
  assert.deepEqual(trace(mapped.code, mapped.map!, "{"), {
    source: "App.vue",
    line: 1,
    column: 0,
    name: null,
  });
  assert.equal(
    originalPositionFor(new TraceMap(JSON.stringify(mapped.map)), { line: 1, column: 15 }).source,
    null,
  );
});

void test("disjoint rewrites preserve mappings between edits and after Unicode text", () => {
  const code =
    'const emoji = "🐱"; const first = "./a.png"; const middle = 1; const last = "./long-b.png"; const tail = 2;';
  const map = new sources.OriginalSource(code, "App.vue").map({ columns: true })!;
  const mapped = new MappedModule(code, map);
  mapped.edit(code.replace('"./a.png"', "_imports_0").replace('"./long-b.png"', "_imports_1"));
  for (const marker of ["const first", "const middle", "const tail"]) {
    assert.deepEqual(trace(mapped.code, mapped.map!, marker), trace(code, map, marker));
  }
  assert.deepEqual(mapped.map!.sourcesContent, [code]);
});

void test("successive line insertions and deletions preserve surviving mappings", () => {
  const code = "const first = 1;\nconst removed = 2;\nconst last = 3;";
  const map = new sources.OriginalSource(code, "App.vue").map({ columns: true })!;
  const mapped = new MappedModule(code, map);
  mapped.prepend('import "./App.vue?vue&type=style";\n');
  const start = mapped.code.indexOf("const removed");
  mapped.replace(start, start + "const removed = 2;\n".length, "");
  mapped.append("\nexport default first;");
  for (const marker of ["const first", "const last"]) {
    assert.deepEqual(trace(mapped.code, mapped.map!, marker), trace(code, map, marker));
  }
  assert.equal(trace(mapped.code, mapped.map!, "import").source, null);
});

void test("module assembly without a map preserves the transformed code", () => {
  const mapped = new MappedModule("export default {};", null);
  mapped.edit("const _sfc_main = {};\nexport default _sfc_main;");
  assert.equal(mapped.code, "const _sfc_main = {};\nexport default _sfc_main;");
  assert.equal(mapped.map, null);
});

void test("large rewrites retain code when mapping alignment exceeds the edit budget", () => {
  const code = "a".repeat(12_000);
  const mapped = new MappedModule(
    code,
    new sources.OriginalSource(code, "App.vue").map({ columns: true })!,
  );
  const next = "b".repeat(12_000);
  mapped.edit(next);
  assert.equal(mapped.code, next);
  assert.equal(mapped.map, null);
});
