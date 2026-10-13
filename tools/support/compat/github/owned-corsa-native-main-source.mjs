import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
const sha = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const url = new URL("./owned-corsa-native-main-source.json", import.meta.url);
const stat = fs.lstatSync(url);
assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
const bytes = fs.readFileSync(url);
assert.equal(sha(bytes), "c385843d4e640b9c614ecaff603f9b19d826192e3333d747be0a2961a86f6d73");
const data = JSON.parse(bytes);
assert.equal(data.schema, 1);
assert.equal(data.sourceOnly, true);
assert.equal(data.runtimeCredit, false);
assert.equal(data.performanceCredit, false);
assert.equal(data.baseline, "bd5702c0a4d5fb3bb64487794fc45f6f876e6e38");
assert.deepEqual(
  data.entries.map(({ file }) => file),
  [
    "crates/vize_canon/src/batch/type_checker/tests/recent_issues.rs",
    ".github/workflows/davinci-canon-scaling.yml",
  ],
);
const values = new Map();
for (const row of data.entries) {
  const pair = {};
  for (const name of ["retained", "before", "after"]) {
    const cell = row[name],
      body = Buffer.from(cell.rawBase64, "base64");
    assert.equal(body.toString("base64"), cell.rawBase64);
    assert.equal(cell.mode, "100644");
    assert.equal(body.length, cell.bytes);
    assert.equal(sha(body), cell.sha256);
    assert.equal(blob(body), cell.blob);
    pair[name] = { mode: cell.mode, blob: cell.blob, body };
  }
  const retained = pair.retained.body.toString().split(/(?<=\n)/u);
  for (const side of ["before", "after"]) {
    const restored = pair[side].body.toString().split(/(?<=\n)/u);
    for (const [i, j, k, l, before, after] of row.clauses[side].toReversed()) {
      assert.equal(retained.slice(i, j).join(""), before);
      assert.equal(restored.slice(k, l).join(""), after);
      restored.splice(k, l - k, ...(before ? before.split(/(?<=\n)/u) : []));
    }
    assert.deepEqual(Buffer.from(restored.join("")), pair.retained.body);
  }
  values.set(row.file, pair);
}
/** Exact incoming additions and the complete request lock call remain in production. */
export function projectOwnedCorsaNativeMainReader(baseline, reader) {
  if (baseline !== data.baseline) return reader;
  for (const [file, pair] of values)
    for (const side of ["before", "after"])
      assert.deepEqual(
        reader(side, file),
        pair[side],
        "whole actual native main overlap: " + side + ":" + file,
      );
  return (side, file) =>
    values.has(file)
      ? { ...values.get(file).retained, body: Buffer.from(values.get(file).retained.body) }
      : reader(side, file);
}

/** Copied source describes only the after side; this grants no before-side authority. */
export function projectOwnedCorsaNativeMainAfter(reader) {
  for (const [file, pair] of values)
    assert.deepEqual(
      reader("after", file),
      pair.after,
      "whole actual native copied source: " + file,
    );
  return (file) =>
    values.has(file)
      ? { ...values.get(file).retained, body: Buffer.from(values.get(file).retained.body) }
      : reader("after", file);
}
