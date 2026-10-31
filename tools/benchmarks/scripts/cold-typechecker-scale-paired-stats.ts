/** Preserve adjacent balanced pairs when resampling the measured command. */
import assert from "node:assert/strict";

export function nearestRank(values: number[], p: number): number {
  assert.ok(values.length > 0 && values.every(Number.isFinite) && p > 0 && p <= 1);
  return [...values].sort((a, b) => a - b)[Math.ceil(values.length * p) - 1];
}
export function linuxPeakRss(stderr: string): number {
  const matches = [...stderr.matchAll(/^\s*Maximum resident set size \(kbytes\): (\d+)$/gmu)];
  assert.equal(matches.length, 1, "one GNU time RSS record required");
  const bytes = Number(matches[0][1]) * 1024;
  assert.ok(Number.isSafeInteger(bytes) && bytes > 0);
  return bytes;
}

export type PairedValue = { base: number; head: number };
export function pairedColdStatistics(pairs: PairedValue[], seed = 11176) {
  assert.equal(pairs.length, 40, "forty complete base/head pairs required");
  assert.ok(
    pairs.every(
      (pair) =>
        Number.isFinite(pair.base) && pair.base > 0 && Number.isFinite(pair.head) && pair.head > 0,
    ),
  );
  const quantiles = (input: PairedValue[]) => {
    const base = input.map((pair) => pair.base),
      head = input.map((pair) => pair.head);
    return {
      baseP50: nearestRank(base, 0.5),
      headP50: nearestRank(head, 0.5),
      baseP95: nearestRank(base, 0.95),
      headP95: nearestRank(head, 0.95),
    };
  };
  const observed = quantiles(pairs);
  let state = seed >>> 0;
  const random = () => {
    state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
    return state / 0x100000000;
  };
  const p50: number[] = [],
    p95: number[] = [];
  for (let draw = 0; draw < 4096; draw++) {
    const selected: PairedValue[] = [];
    for (let block = 0; block < 10; block++) {
      const first = Math.floor(random() * 10) * 4;
      selected.push(...pairs.slice(first, first + 4));
    }
    const value = quantiles(selected);
    p50.push(value.headP50 / value.baseP50);
    p95.push(value.headP95 / value.baseP95);
  }
  return {
    pairs: pairs.length,
    ...observed,
    p50Ratio: observed.headP50 / observed.baseP50,
    p95Ratio: observed.headP95 / observed.baseP95,
    p50Ratio95CI: [nearestRank(p50, 0.025), nearestRank(p50, 0.975)],
    p95Ratio95CI: [nearestRank(p95, 0.025), nearestRank(p95, 0.975)],
    estimator: "nearest-rank quantiles; adjacent four-pair block bootstrap",
    bootstrap: { seed, draws: 4096, blockPairs: 4 },
    tailLimit: "40 observations per source provide a limited p95 estimate",
  };
}
