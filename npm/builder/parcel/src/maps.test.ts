import assert from "node:assert/strict";
import { test } from "node:test";
import { blockMap, byteOffset, editMap } from "./maps.ts";

void test("native byte boundaries reject the inside of astral and non-ASCII characters", () => {
  const source = "é😺\r\nx";
  for (const [bytes, utf16] of [
    [0, 0],
    [2, 1],
    [6, 3],
    [8, 5],
    [9, 6],
  ]) {
    assert.equal(byteOffset(source, bytes), utf16);
  }
  for (const bytes of [-1, 1, 3, 4, 5, 10, 1.5]) {
    assert.throws(() => byteOffset(source, bytes), /Invalid native UTF-8/);
  }
});

void test("style maps point into the full authored CRLF SFC after an astral prefix", () => {
  const source = "<!--😺-->\r\n<style scoped>é.foo { color: red }\r\n.bar{display:block}</style>";
  const start = source.indexOf("é.foo");
  const end = source.indexOf("</style>");
  const map = blockMap(
    source,
    Buffer.byteLength(source.slice(0, start)),
    Buffer.byteLength(source.slice(0, end)),
    "/project/é.vue",
    "/project",
  );
  assert.deepEqual(map.findClosestMapping(1, 0)?.original, { line: 2, column: 14 });
  assert.deepEqual(map.findClosestMapping(1, 2)?.original, { line: 2, column: 16 });
  assert.deepEqual(map.findClosestMapping(2, 1)?.original, { line: 3, column: 1 });
  assert.equal(map.getSourceContent("é.vue"), source);
});

void test("inserted code is unmapped while unchanged UTF-16 columns keep their authored owner", () => {
  const source = ".😺foo { color: red }";
  const original = blockMap(
    source,
    0,
    Buffer.byteLength(source),
    "/project/Component.vue",
    "/project",
  );
  const after = ".😺foo[data-v-123] { color: red }";
  const map = editMap(source, after, original, "/project/Component.vue", "/project");
  assert.ok(map);
  assert.deepEqual(map.findClosestMapping(1, after.indexOf("color"))?.original, {
    line: 1,
    column: source.indexOf("color"),
  });
  assert.equal(map.getSourceContent("Component.vue"), source);
});

void test("multiple removals and insertions reconstruct exact code and keep independent spans", () => {
  const source = "const a = './a.svg';\r\nconst b = './b.svg';\r\nconst keep = 17;";
  const after = "const a = a;\r\nconst b = b;\r\nconst keep = 17;";
  const original = blockMap(
    source,
    0,
    Buffer.byteLength(source),
    "/project/Component.vue",
    "/project",
  );
  const map = editMap(source, after, original, "/project/Component.vue", "/project");
  assert.deepEqual(map?.findClosestMapping(3, 6)?.original, { line: 3, column: 6 });
});
