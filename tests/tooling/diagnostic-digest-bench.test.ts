import assert from "node:assert/strict";
import { test } from "node:test";

import {
  SCENARIOS,
  STRICT_TSCONFIG,
  normalizeOutput,
  parseGnuTime,
  parsePerfStat,
  percentile,
  report,
  rotation,
} from "../../tools/benchmarks/scripts/diagnostic-digest-bench.ts";

test("every round visits every variant, each first in turn", () => {
  const variants = ["base", "head", "base-aa"];
  assert.deepEqual(rotation(variants, 0), ["base", "head", "base-aa"]);
  assert.deepEqual(rotation(variants, 1), ["head", "base-aa", "base"]);
  assert.deepEqual(rotation(variants, 2), ["base-aa", "base", "head"]);
  assert.deepEqual(rotation(variants, 3), variants);
});

test("percentiles are nearest-rank", () => {
  assert.equal(percentile([5, 1, 3, 2, 4], 50), 3);
  assert.equal(percentile([5, 1, 3, 2, 4], 95), 5);
  assert.equal(percentile([7], 50), 7);
  assert.ok(Number.isNaN(percentile([], 50)));
});

test("GNU time and perf stat output parse into one sample", () => {
  assert.deepEqual(parseGnuTime("Command exited with non-zero status 1\n1.25 2.00 0.50 204800\n"), {
    wallMs: 1250,
    cpuMs: 2500,
    rssMb: 200,
  });
  assert.throws(() => parseGnuTime("nothing"));
  assert.equal(parsePerfStat("# started\n\n123456789,,instructions:u,1000,100.00,,\n"), 123456789);
  assert.equal(parsePerfStat("<not supported>,,instructions:u,0,0.00,,\n"), null);
});

test("outputs compare without colour, durations or completion order", () => {
  assert.deepEqual(
    normalizeOutput(
      "\x1b[31m✗ 2 error(s)\x1b[0m\n  b.vue\n  a.vue\n✗ 2 file(s) failed, 0 compiled in 0.3251s\n",
    ),
    ["  a.vue", "  b.vue", "✗ 2 error(s)", "✗ 2 file(s) failed, 0 compiled in <duration>"],
  );
  assert.deepEqual(
    normalizeOutput("✓ Type checked 3 files in 487.30ms (collect: 1.20s, imports: 0.00ns)"),
    ["✓ Type checked 3 files in <duration> (collect: <duration>, imports: <duration>)"],
  );
});

test("the scenarios cover check and build on both corpora, with and without digests", () => {
  assert.deepEqual(
    SCENARIOS.map((scenario) => [scenario.id, scenario.corpus, scenario.digest]),
    [
      ["elk-build-strict", "elk", true],
      ["elk-build", "elk", false],
      ["misskey-build", "misskey", false],
      ["misskey-check-strict", "misskey", true],
      ["misskey-check", "misskey", false],
    ],
  );
  assert.deepEqual(JSON.parse(STRICT_TSCONFIG), {
    extends: "./tsconfig.json",
    vueCompilerOptions: { strictTemplates: true },
  });
  const strict = SCENARIOS.find((scenario) => scenario.id === "misskey-check-strict")!;
  assert.deepEqual(strict.args("/bin/tsc"), [
    "check",
    "--no-config",
    "--corsa-path",
    "/bin/tsc",
    "--tsconfig",
    "tsconfig.digest-strict.json",
  ]);
});

test("the report shows each variant's deltas against base", () => {
  const sample = (wallMs: number) => ({ wallMs, cpuMs: wallMs, rssMb: 10, instructions: 1000 });
  const markdown = report(
    [
      { scenario: "s", variant: "base", exit: 0, samples: [sample(100), sample(100)] },
      { scenario: "s", variant: "head", exit: 0, samples: [sample(110), sample(110)] },
    ],
    [{ scenario: "s", ok: true, note: "head prints exactly base's output" }],
    [],
  );
  assert.match(
    markdown,
    /\| head \| 2 \| 0 \| 110\.0 \| 110\.0 \| 110\.0 \| 10\.0 \| 1000 \| \+10\.00% \| \+10\.00% \| \+0\.00% \|/u,
  );
  assert.match(markdown, /Output: ✓ head prints exactly base's output/u);
});
