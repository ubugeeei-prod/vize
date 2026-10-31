import assert from "node:assert/strict";
import { test } from "node:test";
import { coldScaleAffinity } from "../../tools/benchmarks/scripts/cold-typechecker-scale-affinity.ts";
import {
  linuxPeakRss,
  nearestRank,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-phase-baseline.ts";
import { coldScalePacketView } from "../../tools/benchmarks/scripts/cold-typechecker-scale-packet.ts";

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

const corpus = (root: string) => ({
  root,
  app: `${root}/apps/web`,
  entries: [{ path: "apps/web/src/App.vue" }, { path: "packages/a.ts" }],
});
const packet = (root: string) => ({
  files: [
    { file: `${root}/packages/a.ts`, diagnostics: [] },
    { file: "src/App.vue", diagnostics: ["error:1:2 [TS2322] exact message"] },
  ],
  programs: [{ root: ".", files: ["src/App.vue"] }],
  fileCount: 2,
  errorCount: 1,
  warningCount: 0,
  opaque: { retained: true },
});

test("whole cold packets relocate only census-bound filename identities and retain raw objects", () => {
  const before = packet("/fresh/A"),
    after = packet("/fresh/B");
  const retained = structuredClone(after);
  assert.deepEqual(
    coldScalePacketView(before, corpus("/fresh/A")),
    coldScalePacketView(after, corpus("/fresh/B")),
  );
  assert.deepEqual(after, retained);
  for (const name of [
    "/foreign/packages/a.ts",
    "/fresh/B-other/packages/a.ts",
    "/fresh/B/packages/unknown.ts",
    "/fresh/B/packages/../packages/a.ts",
  ]) {
    const changed = packet("/fresh/B");
    changed.files[0].file = name;
    assert.throws(() => coldScalePacketView(changed, corpus("/fresh/B")));
  }
  const duplicate = packet("/fresh/B");
  duplicate.files[1].file = duplicate.files[0].file;
  assert.throws(() => coldScalePacketView(duplicate, corpus("/fresh/B")));
  const missing = packet("/fresh/B");
  missing.files.pop();
  assert.throws(() => coldScalePacketView(missing, corpus("/fresh/B")));
});

test("whole cold comparison still refuses diagnostic, order, unknown metadata and program changes", () => {
  const expected = coldScalePacketView(packet("/fresh/A"), corpus("/fresh/A"));
  for (const mutate of [
    (value: ReturnType<typeof packet>) => {
      value.files[1].diagnostics[0] = "error:1:3 [TS2322] exact message";
    },
    (value: ReturnType<typeof packet>) => {
      value.files.reverse();
    },
    (value: ReturnType<typeof packet>) => {
      value.opaque.retained = false;
    },
    (value: ReturnType<typeof packet>) => {
      value.programs[0].files.push("extra.ts");
    },
  ]) {
    const changed = packet("/fresh/B");
    mutate(changed);
    assert.throws(() =>
      assert.deepEqual(coldScalePacketView(changed, corpus("/fresh/B")), expected),
    );
  }
  const left = { ...packet("/fresh/A"), unknownPath: "/fresh/A/opaque" };
  const right = { ...packet("/fresh/B"), unknownPath: "/fresh/B/opaque" };
  assert.throws(() =>
    assert.deepEqual(
      coldScalePacketView(left, corpus("/fresh/A")),
      coldScalePacketView(right, corpus("/fresh/B")),
    ),
  );
});
