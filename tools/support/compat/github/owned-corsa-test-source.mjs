import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

const sha = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const url = new URL("./owned-corsa-test-source.json", import.meta.url);
const stat = fs.lstatSync(url);
assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
const bytes = fs.readFileSync(url);
assert.equal(sha(bytes), "6dc0fbe52fbb65a958745961fe0198390318ec497e8e397da2634ec2ddecdc65");
const data = JSON.parse(bytes);
assert.equal(data.schema, 1);
assert.equal(data.sourceOnly, true);
assert.equal(data.runtimeCredit, false);
assert.equal(data.performanceCredit, false);
assert.equal(
  data.original237ManifestSHA256,
  "bffe80f1e770ad10df3af2670e01b5f53a9574f357d42df6948a458f5311040c",
);
assert.deepEqual(
  data.entries.map(({ file }) => file),
  ["vendor/corsa/tests/policy.rs", "vendor/corsa/tests/real_corsa_baseline.rs"],
);
const cells = new Map();
for (const row of data.entries) {
  const values = {};
  for (const side of ["retainedBefore", "currentAfter"]) {
    const cell = row[side],
      body = Buffer.from(cell.rawBase64, "base64");
    assert.equal(body.toString("base64"), cell.rawBase64);
    assert.equal(cell.mode, "100644");
    assert.equal(body.length, cell.bytes);
    assert.equal(sha(body), cell.sha256);
    assert.equal(blob(body), cell.blob);
    values[side] = { ...cell, body };
  }
  let restored = values.currentAfter.body.toString().split(/(?<=\n)/u);
  const original = values.retainedBefore.body.toString().split(/(?<=\n)/u);
  for (const [i, j, k, l, before, after] of [...row.wholeLineClauses].reverse()) {
    assert.equal(original.slice(i, j).join(""), before);
    assert.equal(restored.slice(k, l).join(""), after);
    restored.splice(k, l - k, ...(before ? before.split(/(?<=\n)/u) : []));
  }
  assert.deepEqual(Buffer.from(restored.join("")), values.retainedBefore.body);
  cells.set(row.file, values);
}
const same = (actual, expected) =>
  !!actual &&
  Buffer.isBuffer(actual.body) &&
  actual.mode === expected.mode &&
  actual.blob === expected.blob &&
  actual.body.equals(expected.body) &&
  blob(actual.body) === expected.blob;
const testSourceIdentities = new Map(
  [...cells].map(([file, pair]) => [
    file,
    Object.freeze({
      mode: pair.currentAfter.mode,
      blob: pair.currentAfter.blob,
      bytes: pair.currentAfter.bytes,
      sha256: pair.currentAfter.sha256,
    }),
  ]),
);

export const ownedCorsaTestSourceIdentities = Object.freeze({
  get: (file) => testSourceIdentities.get(file),
  has: (file) => testSourceIdentities.has(file),
});

/** Both whole test bodies must belong to one atomic class before the original237 gate runs. */
export function projectOwnedCorsaTestSources(rows, reader) {
  assert.equal(rows.length, 237);
  for (const [file, value] of cells) {
    const row = rows.find((row) => row.file === file);
    assert(row && row.after.blob === value.retainedBefore.blob);
    assert.equal(row.after.sha256, value.retainedBefore.sha256);
  }
  const old = [...cells].every(([file, value]) => same(reader(file), value.retainedBefore));
  const current = [...cells].every(([file, value]) => same(reader(file), value.currentAfter));
  if (!old && !current) return null;
  return {
    reader: (file) =>
      current && cells.has(file)
        ? {
            ...cells.get(file).retainedBefore,
            body: Buffer.from(cells.get(file).retainedBefore.body),
          }
        : reader(file),
    context: current
      ? {
          authority: data.authority,
          manifestSHA256: sha(bytes),
          completeWholeTestBodies: 2,
          completePinnedUnownedPolicyBodies: 6,
          sourceOnly: true,
          runtimeCredit: false,
          performanceCredit: false,
        }
      : null,
  };
}
