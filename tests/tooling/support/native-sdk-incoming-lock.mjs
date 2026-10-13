import assert from "node:assert/strict";
import { createHash } from "node:crypto";

const sha = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");

/** Decode only the two whole bounded lock cells; callers must qualify actual source before loans. */
export function authenticateHistoricalIncomingLock(finalContext, originalFrames) {
  assert.equal(finalContext.ancestryCredit, false);
  assert.equal(finalContext.squashCredit, false);
  assert.equal(finalContext.dependencyCredit, false);
  assert.equal(finalContext.toolchainCredit, false);
  assert.equal(finalContext.wholeIncomingLock.file, "pnpm-lock.yaml");
  const lockBodies = {};
  for (const side of ["retainedHistoricalAfter", "actualIncomingAfter"]) {
    const expected = finalContext.wholeIncomingLock[side];
    assert.equal(expected.file, "pnpm-lock.yaml");
    assert.equal(expected.mode, "100644");
    const body = Buffer.from(expected.rawBase64, "base64");
    assert.equal(body.toString("base64"), expected.rawBase64);
    assert.equal(body.length, expected.bytes);
    assert.equal(sha(body), expected.sha256);
    assert.equal(blob(body), expected.blob);
    lockBodies[side] = { ...expected, body };
  }
  assert.deepEqual(
    Object.fromEntries(
      Object.entries(finalContext.wholeIncomingLock.retainedHistoricalAfter).filter(
        ([key]) => key !== "rawBase64",
      ),
    ),
    originalFrames.find(({ file }) => file === "pnpm-lock.yaml"),
  );
  return lockBodies;
}
