import assert from "node:assert/strict";
import { chmodSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { test } from "node:test";

import {
  createReductionPlan,
  runReduction,
} from "../../tools/support/compat/github/canonical-corpus-divergence-reduction.mjs";

const fixture = "tests/_fixtures/_git/airi";
const ledger = {
  schema: "vize.fixtureCompatibilityLedger",
  version: 1,
  fixtures: [{ fixturePath: fixture }],
};
const context = {
  sourceCommit: "a".repeat(40),
  runId: "1234",
  runAttempt: "2",
  runUrl: "https://github.com/ubugeeei-prod/vize/actions/runs/1234",
};

function log(divergences, samples = []) {
  return [
    `davinci DOM corpus sweep: files=1 unreadable=0 parsed=1 templates=1 compared=1 patch_fact_entries=1 old_error_skips=0 s2_refusals=0 divergences=${divergences}`,
    ...samples,
  ].join("\n");
}

await test("clean evidence does not request a reducer", () => {
  assert.deepEqual(createReductionPlan(log(0), ledger, `${fixture}\n`, context), {
    schema: "vize.realProjectDivergenceReductionPlan",
    version: 1,
    status: "clean",
    divergenceCount: 0,
    sourceCommit: context.sourceCommit,
    runUrl: context.runUrl,
  });
});

await test("the first fixture-owned sample becomes a bounded reduction plan", () => {
  const plan = createReductionPlan(
    log(1, [
      `${fixture}/src/A.vue: old_len=20 new_len=21 first_diff=4 old_window=old new_window=new`,
    ]),
    ledger,
    `${fixture}\n`,
    context,
  );
  assert.equal(plan.status, "required");
  assert.equal(plan.fixturePath, fixture);
  assert.equal(plan.sourcePath, `${fixture}/src/A.vue`);
  assert.match(plan.fingerprint, /^[a-f0-9]{12}$/);
});

await test("a reducer invocation records only the minimized witness and provenance", () => {
  const root = mkdtempSync("tests/.canonical-dom-reduction-");
  try {
    const sourcePath = `${root}/source.vue`;
    const outputDirectory = `${root}/out`;
    const reducer = `${root}/vize`;
    mkdirSync(outputDirectory);
    writeFileSync(sourcePath, "<template><div><span>large source</span></div></template>");
    writeFileSync(
      reducer,
      "#!/bin/sh\nprintf '<template><span/></template>' > \"$5\"\nprintf 'reduce: 58 -> 28 bytes, 1-minimal\\n' >&2\n",
    );
    chmodSync(reducer, 0o755);
    const plan = {
      schema: "vize.realProjectDivergenceReductionPlan",
      version: 1,
      status: "required",
      divergenceCount: 1,
      fingerprint: "abc123abc123",
      fixturePath: root,
      sourcePath,
      sourceCommit: context.sourceCommit,
      runUrl: context.runUrl,
    };
    const planPath = `${outputDirectory}/reduction-plan.json`;
    writeFileSync(planPath, JSON.stringify(plan));
    const receipt = runReduction(planPath, outputDirectory, reducer);
    assert.equal(receipt.status, "reduced");
    assert.equal(receipt.reducedFile, "canonical-dom-divergence.reduced.vue");
    assert.equal(receipt.sourcePath, sourcePath);
    assert.match(receipt.reducedSha256, /^[a-f0-9]{64}$/);
    assert.equal(
      readFileSync(`${outputDirectory}/${receipt.reducedFile}`, "utf8"),
      "<template><span/></template>",
    );
    assert.equal(
      readFileSync(`${outputDirectory}/reduction.json`, "utf8").includes("large source"),
      false,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
