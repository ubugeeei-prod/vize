import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

const sha = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const file = new URL("./owned-corsa-caller-metadata.json", import.meta.url);
const stat = fs.lstatSync(file);
assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
const raw = fs.readFileSync(file);
export const ownedCorsaCallerMetadataSHA256 =
  "5c2f55e5c0425a8c939e9501f92bd72c72ebbd7dd9eeb33e90c374d63ab448fc";
assert.equal(sha(raw), ownedCorsaCallerMetadataSHA256);
const metadata = JSON.parse(raw);
assert.equal(metadata.version, 1);
assert.equal(metadata.beforeSource, "2489128d5f8fab1b7442a1e5c09b4e1701cbf185");
assert.equal(metadata.previousSource, "cd7fa45e69994877d08da7bb66883f76bf4f4d37");
export const ownedCorsaCallerPaths = Object.freeze(metadata.files.map(({ file }) => file));
assert.deepEqual(ownedCorsaCallerPaths, [
  "tests/performance/support/warm-type-backed-native-terminal-final.ts",
  "tests/performance/support/warm-type-backed-workflow.ts",
]);

function capsule(identity) {
  if (identity === null) return null;
  const body = Buffer.from(identity.rawBase64, "base64");
  assert.equal(body.toString("base64"), identity.rawBase64);
  assert.equal(identity.mode, "100644");
  assert.equal(body.length, identity.bytes);
  assert.equal(sha(body), identity.sha256);
  assert.equal(blob(body), identity.blob);
  return { mode: "100644", blob: identity.blob, body };
}
function clauses(body, replacements) {
  let current = body.toString();
  for (const [before, after] of replacements) {
    assert.equal(current.split(before).length, 2, "one exact whole caller clause");
    current = current.replace(before, after);
  }
  return Buffer.from(current);
}
function equal(actual, expected) {
  if (expected === null) assert.equal(actual, null);
  else {
    assert(actual && actual.mode === "100644" && Buffer.isBuffer(actual.body));
    assert.equal(actual.blob, expected.blob);
    assert.deepEqual(actual.body, expected.body);
  }
}

/** Actual incoming Reader clauses survive in current source; only the named private view is historical. */
export function verifyOwnedCorsaCallers(reader) {
  const retained = new Map();
  for (const entry of metadata.files) {
    assert.equal(entry.previousSource, metadata.previousSource);
    const previous = capsule({
      mode: entry.previousMode,
      blob: entry.previousBlob,
      bytes: entry.previousBytes,
      sha256: entry.previousSHA256,
      rawBase64: entry.previousRawBase64,
    });
    const actual = reader("after", entry.file);
    assert(actual && actual.mode === "100644" && Buffer.isBuffer(actual.body));
    assert.equal(blob(actual.body), actual.blob);
    assert.equal(sha(actual.body), entry.afterSHA256, "complete current caller: " + entry.file);
    assert.deepEqual(
      clauses(previous.body, entry.forwardClauses),
      actual.body,
      "complete named caller inverse",
    );
    const actualBefore = capsule(entry.actualBefore);
    equal(reader("before", entry.file), actualBefore);
    const historicalBefore = capsule(entry.historicalBefore);
    const historicalAfter = capsule(entry.historicalAfter);
    if (actualBefore)
      assert.deepEqual(
        clauses(actualBefore.body, entry.incomingInverseClauses),
        historicalBefore.body,
      );
    else assert.equal(historicalBefore, null);
    assert.deepEqual(clauses(previous.body, entry.incomingInverseClauses), historicalAfter.body);
    retained.set("before:" + entry.file, historicalBefore);
    retained.set("after:" + entry.file, historicalAfter);
  }
  return {
    receipt: {
      authority: "complete-reviewed-owned-corsa-two-caller-source",
      manifestSHA256: ownedCorsaCallerMetadataSHA256,
      actualBeforeSource: metadata.beforeSource,
      previousAfterSource: metadata.previousSource,
      completeCallerBodies: 2,
      completeCallerPairSides: 4,
      incomingReaderClausesPreserved: true,
      runtimeCredit: false,
      performanceCredit: false,
    },
    reader: (side, file) => {
      const key = side + ":" + file;
      if (!retained.has(key)) return reader(side, file);
      const value = retained.get(key);
      return value ? { ...value, body: Buffer.from(value.body) } : null;
    },
  };
}
