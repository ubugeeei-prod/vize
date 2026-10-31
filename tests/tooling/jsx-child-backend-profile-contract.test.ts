import assert from "node:assert/strict";
import { test } from "node:test";
import { contract, verifyBackendOutputs } from "./support/jsx-child-backend-profile-contract.ts";
const fixtures = () => ({
  before: contract.rows.map((row) => ({
    source: row.source,
    ssr: row.ssr,
    iterationsPerSample: row.iterationsPerSample,
    output: structuredClone(row.before),
    samplesNs: Array.from({ length: 9 }, (_, index) => index + 1),
  })),
  after: contract.rows.map((row) => ({
    source: row.source,
    ssr: row.ssr,
    iterationsPerSample: row.iterationsPerSample,
    output: structuredClone(row.after),
    samplesNs: Array.from({ length: 9 }, (_, index) => index + 2),
  })),
});
test("all six whole backend profiles preserve inputs and exactly three intentional client changes", () => {
  const { before, after } = fixtures();
  assert.deepEqual(verifyBackendOutputs(before, after), [0, 2, 4]);
});
test("backend custody rejects client/SSR output, input, population and raw sample mutations", () => {
  for (const index of [0, 1, 2, 3, 4, 5])
    for (const side of ["before", "after"] as const) {
      const pair = fixtures();
      pair[side][index].output = "foreign output";
      assert.throws(() => verifyBackendOutputs(pair.before, pair.after), assert.AssertionError);
    }
  for (const mutate of [
    (pair: ReturnType<typeof fixtures>) => {
      pair.after[0].source += "foreign";
    },
    (pair: ReturnType<typeof fixtures>) => {
      pair.after[0].ssr = true;
    },
    (pair: ReturnType<typeof fixtures>) => {
      pair.before[2].iterationsPerSample++;
    },
    (pair: ReturnType<typeof fixtures>) => {
      pair.after.pop();
    },
    (pair: ReturnType<typeof fixtures>) => {
      pair.after[0].samplesNs.pop();
    },
    (pair: ReturnType<typeof fixtures>) => {
      pair.after[0].samplesNs[0] = NaN;
    },
  ]) {
    const pair = fixtures();
    mutate(pair);
    assert.throws(() => verifyBackendOutputs(pair.before, pair.after), assert.AssertionError);
  }
});
