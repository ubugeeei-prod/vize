import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
const sha = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const url = new URL("./owned-corsa-sdk-repair-source.json", import.meta.url),
  stat = fs.lstatSync(url);
assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
const bytes = fs.readFileSync(url);
assert.equal(sha(bytes), "bb50486a0222f1ff6ae33dae9b0a7a08fb8b92c2e359b2bc6db605756d58b3ca");
const data = JSON.parse(bytes);
assert.equal(data.schema, 1);
assert.equal(data.sourceOnly, true);
assert.equal(data.runtimeCredit, false);
assert.equal(data.performanceCredit, false);
assert.deepEqual(
  data.frames.map(({ file }) => file),
  [
    ".github/workflows/davinci-canon-scaling.yml",
    "crates/vize_canon/Cargo.toml",
    "crates/vize_canon/src/batch/type_checker/tests/recent_issues.rs",
    "tools/support/compat/github/owned-corsa-lexical-source.mjs",
    "tools/support/compat/github/owned-corsa-clippy-source.mjs",
    "tools/support/compat/github/owned-corsa-source-reader.mjs",
  ],
);
const frames = new Map();
for (const row of data.frames) {
  const pair = {};
  for (const side of ["before", "after"]) {
    const cell = row[side],
      body = Buffer.from(cell.rawBase64, "base64");
    assert.equal(body.toString("base64"), cell.rawBase64);
    assert.equal(cell.mode, "100644");
    assert.equal(body.length, cell.bytes);
    assert.equal(sha(body), cell.sha256);
    assert.equal(blob(body), cell.blob);
    pair[side] = { ...cell, body };
  }
  assert.equal(row.oldFrame.mode, pair.before.mode);
  assert.equal(row.oldFrame.blob, pair.before.blob);
  assert.equal(row.oldFrame.bytes, pair.before.bytes);
  assert.equal(row.oldFrame.sha256, pair.before.sha256);
  const original = pair.before.body.toString().split(/(?<=\n)/u);
  const restored = pair.after.body.toString().split(/(?<=\n)/u);
  for (const [i, j, k, l, before, after] of row.clauses.toReversed()) {
    assert.equal(original.slice(i, j).join(""), before);
    assert.equal(restored.slice(k, l).join(""), after);
    restored.splice(k, l - k, ...(before ? before.split(/(?<=\n)/u) : []));
  }
  assert.deepEqual(Buffer.from(restored.join("")), pair.before.body);
  frames.set(row.file, { row, pair });
}
/** Only an atomic complete6 successor can replace cells in the complete original126 frame. */
export function ownedCorsaSdkRepairFrames(originalFrames, reader) {
  const old = [...frames].every(([file, { pair }]) =>
    reader("after", file)?.body.equals(pair.before.body),
  );
  const current = [...frames].every(([file, { pair }]) =>
    reader("after", file)?.body.equals(pair.after.body),
  );
  assert(old || current, "mixed or foreign SDK delivery frames refuse");
  const selected = new Map(originalFrames);
  for (const [file, { row, pair }] of frames) {
    assert.deepEqual(originalFrames.get(file), row.oldFrame);
    if (current) selected.set(file, { ...row.oldFrame, ...pair.after });
  }
  return selected;
}
