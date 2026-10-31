import assert from "node:assert/strict";
import { test } from "node:test";
import { coldScaleAffinity } from "../../tools/benchmarks/scripts/cold-typechecker-scale-affinity.ts";
import {
  linuxPeakRss,
  nearestRank,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-phase-baseline.ts";

test("cold baseline obeys nonzero and sparse cgroup CPU sets", () => {
  assert.equal(
    coldScaleAffinity("Name:\tnode\nCpus_allowed_list:\t16-19,32-39\n"),
    "16,17,18,19,32,33,34,35",
  );
  assert.equal(coldScaleAffinity("Cpus_allowed_list:\t7,11\n"), "7,11");
  for (const value of [
    "",
    "Cpus_allowed_list:\t9-3\n",
    "Cpus_allowed_list:\t1,1\n",
    "Cpus_allowed_list:\t1-2,2-4\n",
    "Cpus_allowed_list:\t-1\n",
  ])
    assert.throws(() => coldScaleAffinity(value));
});

test("cold baseline retains small-sample tail and explicitly scoped GNU time RSS", () => {
  const input = [80, 10, 70, 20, 60, 30, 50, 40];
  assert.equal(nearestRank(input, 0.5), 40);
  assert.equal(nearestRank(input, 0.95), 80);
  assert.deepEqual(input, [80, 10, 70, 20, 60, 30, 50, 40]);
  assert.equal(linuxPeakRss("\tMaximum resident set size (kbytes): 1234\n"), 1263616);
  assert.throws(() => linuxPeakRss(""));
  assert.throws(() =>
    linuxPeakRss("Maximum resident set size (kbytes): 1\nMaximum resident set size (kbytes): 2\n"),
  );
  assert.throws(() => nearestRank([], 0.95));
});
