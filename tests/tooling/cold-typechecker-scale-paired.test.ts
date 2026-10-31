import assert from "node:assert/strict";
import { test } from "node:test";
import { pairedColdStatistics } from "../../tools/benchmarks/scripts/cold-typechecker-scale-paired-stats.ts";

test("block resampling keeps both sides of every complete pair and retains all samples", () => {
  const pairs = Array.from({ length: 40 }, (_, index) => ({
    base: index + 1,
    head: (index + 1) / 2,
  }));
  const original = structuredClone(pairs);
  const result = pairedColdStatistics(pairs);
  assert.equal(result.p50Ratio, 0.5);
  assert.equal(result.p95Ratio, 0.5);
  assert.deepEqual(result.p50Ratio95CI, [0.5, 0.5]);
  assert.deepEqual(result.p95Ratio95CI, [0.5, 0.5]);
  assert.deepEqual(pairs, original);
  assert.deepEqual(pairedColdStatistics(pairs), result);
  assert.throws(() => pairedColdStatistics(pairs.slice(1)));
  assert.throws(() => pairedColdStatistics([...pairs.slice(1), { base: 0, head: 1 }]));
});
