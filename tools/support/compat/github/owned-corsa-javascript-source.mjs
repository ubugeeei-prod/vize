import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import {
  authenticateJavaScriptSfcSource,
  javascriptSfcSourcePaths,
  javascriptSfcContextPaths,
} from "./javascript-sfc-source-context.mjs";
const sha = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const read = (name) => {
  const url = new URL(name, import.meta.url),
    stat = fs.lstatSync(url);
  assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
  return fs.readFileSync(url);
};
const bytes = read("./owned-corsa-javascript-source.json");
assert.equal(sha(bytes), "17f2b614ffb85b4a90c56378b6b0772541ee2fd05f56134d324699fdade42ef8");
const data = JSON.parse(bytes);
assert.equal(data.schema, 1);
assert.equal(data.mainSource, "bd5702c0a4d5fb3bb64487794fc45f6f876e6e38");
assert.equal(data.sourceOnly, true);
assert.equal(data.runtimeCredit, false);
assert.equal(data.performanceCredit, false);
const rows = data.parts.flatMap(([name, checksum], i) => {
  assert.equal(name, `owned-corsa-javascript-source-part-${i}.json`);
  const body = read(name);
  assert.equal(sha(body), checksum);
  return JSON.parse(body);
});
assert.equal(rows.length, 8);
assert.equal(new Set(rows.map(({ file }) => file)).size, 8);
const cells = new Map();
for (const row of rows) {
  assert(javascriptSfcSourcePaths.includes(row.file));
  const pair = {};
  for (const side of row.retainedSdk ? ["main", "owned", "retainedSdk"] : ["main", "owned"]) {
    const cell = row[side],
      body = Buffer.from(cell.rawBase64, "base64");
    assert.equal(body.toString("base64"), cell.rawBase64);
    assert.equal(cell.mode, "100644");
    assert.equal(body.length, cell.bytes);
    assert.equal(sha(body), cell.sha256);
    assert.equal(blob(body), cell.blob);
    pair[side] = { mode: cell.mode, blob: cell.blob, body };
  }
  const original = pair.main.body.toString().split(/(?<=\n)/u);
  const restored = pair.owned.body.toString().split(/(?<=\n)/u);
  for (const [i, j, k, l, before, after] of row.clauses.toReversed()) {
    assert.equal(original.slice(i, j).join(""), before);
    assert.equal(restored.slice(k, l).join(""), after);
    restored.splice(k, l - k, ...(before ? before.split(/(?<=\n)/u) : []));
  }
  assert.deepEqual(Buffer.from(restored.join("")), pair.main.body);
  cells.set(row.file, pair);
}
export const ownedJavaScriptLexicalPaths = Object.freeze([
  ...javascriptSfcSourcePaths,
  ...javascriptSfcContextPaths,
  "tools/support/compat/github/owned-corsa-javascript-source.mjs",
  "tools/support/compat/github/owned-corsa-javascript-source.json",
  ...data.parts.map(([name]) => "tools/support/compat/github/" + name),
]);

/** The complete current JS scope precedes any historical view; overlapping SDK bodies are literal. */
export function authenticateOwnedJavaScriptSource(reader) {
  for (const [file, pair] of cells)
    assert.deepEqual(reader("after", file), pair.owned, "whole actual JS/SDK composition: " + file);
  const context = authenticateJavaScriptSfcSource((file) =>
    cells.has(file) ? Buffer.from(cells.get(file).main.body) : reader("after", file).body,
  );
  return {
    ...context,
    retainedOwnedAfter(file) {
      assert.equal(file, "tests/_fixtures/differential/typechecker/manifest.json");
      return Buffer.from(cells.get(file).retainedSdk.body);
    },
  };
}
