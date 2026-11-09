import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import {
  divergenceObservers,
  parseSsrDivergences,
} from "../../tools/support/compat/github/canonical-corpus-divergence-observers.ts";
import { createDivergenceIssue } from "../../tools/support/compat/github/canonical-corpus-divergence-issue.mjs";
import { createPromotionPlan } from "../../tools/support/compat/github/canonical-corpus-divergence-promotion.mjs";
import {
  createReductionPlan,
  runReduction,
} from "../../tools/support/compat/github/canonical-corpus-divergence-reduction.mjs";

const { ssr } = divergenceObservers;
const fixtureA = "tests/_fixtures/_git/airi";
const fixtureB = "tests/_fixtures/_git/create-vue";
const ledger = {
  schema: "vize.fixtureCompatibilityLedger",
  version: 1,
  fixtures: [{ fixturePath: fixtureA }, { fixturePath: fixtureB }],
};
const selected = `${fixtureA}\n${fixtureB}\n`;
const context = {
  sourceCommit: "a".repeat(40),
  runId: "1234",
  runAttempt: "2",
  runUrl: "https://github.com/ubugeeei-prod/vize/actions/runs/1234",
};
const root = "/home/runner/work/vize/vize/crates/vize_atelier_ssr/../..";

type Shard = { counter: number | null; lines?: string[] };

// The shape `canonical-corpus-shards.ts run ssr-pug` writes: one segment per
// process, each production sweep printing its counter and, on a divergence,
// the panic that names every divergent path (code dumps for the first ten).
function ssrLog(shards: Shard[], total = shards.length): string {
  return [
    "canonical observer process: test=davinci_ssr_corpus role=base",
    "canonical observer process exit: test=davinci_ssr_corpus role=base status=0 signal=none elapsed_ms=11",
    ...shards.flatMap((shard, index) => [
      `canonical observer process: test=davinci_ssr_corpus role=shard index=${index} count=${total} vector=ssr-pug-davinci_ssr_corpus-shard-${index}-of-${total}.nul`,
      "davinci-differential corpus scope: root=tests/_fixtures/_git scope=canonical closure_evidence=true submodules=2",
      ...(shard.counter === null
        ? []
        : [
            `davinci SSR production sweep: scope=canonical templates=3 compared=3 s4=3 error_skips=0 rejected=0 divergences=${shard.counter}`,
            'davinci SSR production lanes: {"s4": 3}',
          ]),
      ...(shard.lines ?? []),
      `canonical observer process exit: test=davinci_ssr_corpus role=shard index=${index} status=0 signal=none elapsed_ms=1`,
    ]),
    "canonical observer process: test=davinci_pug_corpus role=full",
    `${root}/${fixtureA}/src/Pug.vue ["s4"]`,
    "canonical observer process exit: test=davinci_pug_corpus role=full status=0 signal=none elapsed_ms=1",
  ].join("\n");
}

const divergent = ssrLog([
  { counter: 0 },
  {
    counter: 2,
    lines: [
      "thread '<unnamed>' panicked at crates/vize_atelier_ssr/tests/davinci_ssr_corpus/production.rs:73:5:",
      "production: 2 production SFC divergences:",
      `${root}/${fixtureB}/src/B.vue ["s4", "legacy.operation"]`,
      "--- selected",
      `_sfc_main.__file = "${root}/${fixtureB}/src/B.vue" ["s4"]`,
      "--- legacy",
      `${root}/${fixtureA}/src/A.vue []: selected lane failed: SfcError { message: "x" }`,
    ],
  },
]);

void test("SSR evidence sums every shard and keeps only fixture-relative sample paths", () => {
  assert.deepEqual(parseSsrDivergences(ssrLog([{ counter: 0 }, { counter: 0 }])), {
    divergenceCount: 0,
    samples: [],
  });
  assert.deepEqual(parseSsrDivergences(divergent), {
    divergenceCount: 2,
    samples: [
      { sourcePath: `${fixtureA}/src/A.vue`, lanes: [], selectedFailed: true },
      {
        sourcePath: `${fixtureB}/src/B.vue`,
        lanes: ["s4", "legacy.operation"],
        selectedFailed: false,
      },
    ],
  });
});

void test("SSR evidence fails closed on missing shards, counters or samples", () => {
  assert.throws(
    () => parseSsrDivergences(ssrLog([{ counter: 0 }], 2)),
    /SSR shard evidence is incomplete/,
  );
  assert.throws(
    () => parseSsrDivergences(ssrLog([{ counter: 0 }, { counter: null }])),
    /one canonical SSR production counter in shard 1/,
  );
  assert.throws(
    () =>
      parseSsrDivergences(
        ssrLog([{ counter: 2, lines: [`${root}/${fixtureA}/src/A.vue ["s4"]`] }]),
      ),
    /samples are incomplete in shard 0/,
  );
});

void test("an SSR difference becomes a fixture-owned issue without compiled output", () => {
  const plan = createDivergenceIssue(ssr, divergent, ledger, selected, context);
  assert.ok(plan);
  assert.match(plan.title, /^fix\(davinci\): triage canonical SSR divergence [a-f0-9]{12}$/);
  assert.equal(plan.observer, "ssr");
  assert.deepEqual(plan.fixtures, [fixtureA, fixtureB]);
  assert.match(plan.body, /legacy-vs-Davinci SSR output divergence/);
  assert.match(plan.body, /canonical-corpus-ssr-pug-1234-2-/);
  assert.match(plan.body, /production lanes=`s4, legacy\.operation`/);
  assert.match(plan.body, /the selected lane failed to compile/);
  assert.doesNotMatch(plan.body, /\/home\/runner|_sfc_main|SfcError/);
  assert.deepEqual(plan.labels, ["area:ci", "area:testing", "area:davinci"]);
});

void test("an SSR reduction runs the production predicate under the corpus path", () => {
  const plan = createReductionPlan(ssr, divergent, ledger, selected, context);
  assert.equal(plan.observer, "ssr");
  assert.equal(plan.sourcePath, `${fixtureA}/src/A.vue`);
  const directory = mkdtempSync("tests/.canonical-ssr-reduction-");
  try {
    const sourcePath = `${directory}/O'Brien.vue`;
    const output = `${directory}/out`;
    const vize = `${directory}/bin/vize`;
    mkdirSync(output);
    mkdirSync(`${directory}/bin`);
    writeFileSync(sourcePath, "<template><div><span>large source</span></div></template>");
    writeFileSync(
      vize,
      `#!/bin/sh\nprintf '%s\\n' "$@" > "${output}/args.txt"\nprintf '<template><span/></template>' > "$6"\n`,
    );
    chmodSync(vize, 0o755);
    const planPath = `${output}/reduction-plan.json`;
    writeFileSync(planPath, JSON.stringify({ ...plan, fixturePath: directory, sourcePath }));
    const receipt = runReduction(planPath, output, vize);
    assert.equal(receipt.status, "reduced");
    assert.equal(receipt.observer, "ssr");
    assert.equal(receipt.reducedFile, "canonical-ssr-divergence.reduced.vue");
    assert.deepEqual(readFileSync(`${output}/args.txt`, "utf8").split("\n").slice(0, 4), [
      "reduce",
      sourcePath,
      "--script",
      `'${directory}/bin/examples/ssr_lane_divergence' --name '${directory}/O'\\''Brien.vue'`,
    ]);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

void test("a reduced SSR witness is promoted beside its observer's fixture test", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-ssr-promotion-"));
  try {
    const witness = "<template><b>bold</b></template>";
    writeFileSync(join(directory, "canonical-ssr-divergence.reduced.vue"), witness);
    writeFileSync(
      join(directory, "reduction.json"),
      JSON.stringify({
        schema: "vize.realProjectDivergenceReductionReceipt",
        version: 1,
        observer: "ssr",
        status: "reduced",
        divergenceCount: 1,
        fingerprint: "abc123abc123",
        sourcePath: `${fixtureA}/src/App.vue`,
        sourceBytes: 200,
        reducedFile: "canonical-ssr-divergence.reduced.vue",
        reducedBytes: Buffer.byteLength(witness),
        reducedSha256: createHash("sha256").update(witness).digest("hex"),
        sourceCommit: context.sourceCommit,
        runUrl: context.runUrl,
      }),
    );
    const plan = createPromotionPlan(directory);
    assert.ok(plan);
    assert.equal(plan.branch, "automation/davinci-ssr-abc123abc123");
    assert.match(plan.title, /preserve real-project SSR divergence abc123abc123$/);
    assert.match(plan.body, /promoted_real_project_ssr_fixtures_agree/);
    const prefix = "tests/_fixtures/differential/compiler/real-project-ssr-reductions/abc123abc123";
    assert.deepEqual(
      plan.files.map((file: { path: string }) => file.path),
      [`${prefix}/witness.vue.txt`, `${prefix}/provenance.json`],
    );
    const provenance = JSON.parse(plan.files[1].content);
    assert.equal(provenance.observer, "ssr");
    assert.equal(provenance.sourcePath, `${fixtureA}/src/App.vue`);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
