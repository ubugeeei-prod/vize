import assert from "node:assert/strict";
import { test } from "node:test";

import {
  createRealProjectSurfaceVerdict,
  realProjectSurfaceNames,
} from "../../tools/fixtures/real-project-surface-verdict.mjs";

const successfulResults = realProjectSurfaceNames.map((name) => ({ name, outcome: "success" }));

test("the real-project surface verdict accepts only a complete successful set", () => {
  const verdict = createRealProjectSurfaceVerdict(successfulResults, {
    GITHUB_SHA: "0123456789abcdef",
    FIXTURE_SHARD_INDEX: "7",
  });
  assert.equal(verdict.status, "success");
  assert.equal(verdict.sourceCommit, "0123456789abcdef");
  assert.equal(verdict.shardIndex, "7");
  assert.deepEqual(verdict.failedSurfaceNames, []);
  assert.deepEqual(verdict.surfaces, successfulResults);
});

for (const outcome of ["failure", "cancelled", "skipped"] as const) {
  test(`the real-project surface verdict fails closed on ${outcome}`, () => {
    const verdict = createRealProjectSurfaceVerdict(
      successfulResults.map((result) =>
        result.name === "core-tools" ? { ...result, outcome } : result,
      ),
    );
    assert.equal(verdict.status, "failure");
    assert.deepEqual(verdict.failedSurfaceNames, ["core-tools"]);
  });
}

test("the real-project surface verdict rejects missing, duplicate, unknown, and empty outcomes", () => {
  assert.throws(
    () => createRealProjectSurfaceVerdict(successfulResults.slice(1)),
    /missing real-project surface verdict.*waiver-audit/,
  );
  assert.throws(
    () => createRealProjectSurfaceVerdict([...successfulResults, successfulResults[0]]),
    /duplicate real-project surface/,
  );
  assert.throws(
    () =>
      createRealProjectSurfaceVerdict([
        ...successfulResults.slice(1),
        { name: "unknown", outcome: "success" },
      ]),
    /unknown real-project surface/,
  );
  assert.throws(
    () =>
      createRealProjectSurfaceVerdict(
        successfulResults.map((result) =>
          result.name === "glyph" ? { ...result, outcome: "" } : result,
        ),
      ),
    /invalid outcome for glyph/,
  );
});
