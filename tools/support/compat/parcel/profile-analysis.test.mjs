import assert from "node:assert/strict";
import { test } from "node:test";
import { analyseProfile } from "./profile-analysis.mjs";

function fixture(timeDeltas) {
  return {
    nodes: [
      {
        id: 1,
        callFrame: {
          functionName: "transform",
          url: "file:///consumer/node_modules/@vizejs/parcel-transformer/dist/index.cjs",
          lineNumber: 14,
          columnNumber: 2,
        },
      },
      { id: 2, callFrame: { functionName: "idle", url: "", lineNumber: -1, columnNumber: -1 } },
    ],
    samples: [1, 2, 1],
    timeDeltas,
  };
}

void test("signed SDK timestamps retain every sample and withhold derived time", () => {
  const profile = fixture([1000, -120, 350]);
  const original = JSON.stringify(profile);
  const result = analyseProfile(profile);
  assert.equal(JSON.stringify(profile), original);
  assert.equal(result.samples, 3);
  assert.equal(result.monotonic, false);
  assert.deepEqual(result.negativeDeltas, [{ index: 1, nodeId: 2, delta: -120 }]);
  assert.equal(result.adapter[0].samples, 2);
  assert.equal(result.adapter[0].sampledIntervalsMicros, null);
});

void test("monotonic intervals retain source lines and exact self sample counts", () => {
  const result = analyseProfile(fixture([1000, 200, 350]));
  assert.equal(result.monotonic, true);
  assert.deepEqual(result.negativeDeltas, []);
  assert.equal(result.adapter[0].samples, 2);
  assert.equal(result.adapter[0].sampledIntervalsMicros, 1350);
  assert.equal(result.adapter[0].line, 15);
});

void test("malformed IDs, missing intervals and non-integers remain errors", () => {
  for (const deltas of [[1000], [1000, null, 20], [1000, 0.5, 20]])
    assert.throws(() => analyseProfile(fixture(deltas)));
  const missing = fixture([1000, 200, 350]);
  missing.samples[1] = 999;
  assert.throws(() => analyseProfile(missing), /existing node/);
  const duplicate = fixture([1000, 200, 350]);
  duplicate.nodes.push(duplicate.nodes[0]);
  assert.throws(() => analyseProfile(duplicate), /unique/);
});
