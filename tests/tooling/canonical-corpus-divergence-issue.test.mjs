import assert from "node:assert/strict";
import { test } from "node:test";

import {
  createDivergenceIssue,
  parseCorpusDivergences,
  publishDivergenceIssue,
} from "../../tools/support/compat/github/canonical-corpus-divergence-issue.mjs";

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

function log(divergences, samples = []) {
  return [
    "davinci-differential corpus scope: root=tests/_fixtures/_git scope=canonical closure_evidence=true submodules=2",
    `davinci DOM corpus sweep: files=2 unreadable=0 parsed=2 templates=2 compared=2 patch_fact_entries=9 old_error_skips=0 s2_refusals=0 divergences=${divergences}`,
    `corpus divergences (${divergences}):`,
    ...samples,
    "",
    "corpus patch facts: compared=2 materialized_entries=9",
  ].join("\n");
}

await test("clean canonical evidence creates no issue plan", () => {
  const clean = log(0);
  assert.deepEqual(parseCorpusDivergences(clean), { divergenceCount: 0, samples: [] });
  assert.equal(createDivergenceIssue(clean, ledger, selected, context), null);
});

await test("canonical differences become one normalized fixture-owned issue", () => {
  const evidence = log(2, [
    `/home/runner/work/vize/vize/${fixtureA}/src/A.vue: old_len=10 new_len=11 first_diff=4 old_window=old\\nwindow new_window=new\\nwindow`,
    `${fixtureB}/src/B.vue: old_len=20 new_len=18 first_diff=7 old_window=before new_window=after`,
  ]);
  const plan = createDivergenceIssue(evidence, ledger, selected, context);
  assert.match(plan.title, /^fix\(davinci\): triage canonical DOM divergence [a-f0-9]{12}$/);
  assert.equal(plan.divergenceCount, 2);
  assert.deepEqual(plan.fixtures, [fixtureA, fixtureB]);
  assert.deepEqual(
    plan.samples.map((sample) => sample.sourcePath),
    [`${fixtureA}/src/A.vue`, `${fixtureB}/src/B.vue`],
  );
  assert.match(plan.body, /canonical-corpus-dom-1234-2-/);
  assert.match(plan.body, /minimize it with `vize reduce`/);
  assert.doesNotMatch(plan.body, /\/home\/runner/);
  assert.deepEqual(plan.labels, ["priority:p0", "area:ci", "area:testing", "area:davinci"]);
});

await test("a matching automatic reduction receipt is linked without copying source", () => {
  const evidence = log(1, [
    `${fixtureA}/src/A.vue: old_len=20 new_len=21 first_diff=4 old_window=old new_window=new`,
  ]);
  const first = createDivergenceIssue(evidence, ledger, selected, context);
  const receipt = {
    schema: "vize.realProjectDivergenceReductionReceipt",
    version: 1,
    status: "reduced",
    divergenceCount: 1,
    fingerprint: first.fingerprint,
    sourcePath: `${fixtureA}/src/A.vue`,
    sourceBytes: 200,
    reducedFile: "canonical-dom-divergence.reduced.vue",
    reducedBytes: 40,
    reducedSha256: "b".repeat(64),
    sourceCommit: context.sourceCommit,
    runUrl: context.runUrl,
  };
  const promotion = {
    schema: "vize.realProjectDivergenceFixturePromotionReceipt",
    version: 1,
    status: "created",
    fingerprint: first.fingerprint,
    pullRequestNumber: 93,
    pullRequestUrl: "https://example.test/pull/93",
    sourceCommit: context.sourceCommit,
  };
  const plan = createDivergenceIssue(evidence, ledger, selected, context, receipt, promotion);
  assert.match(plan.body, /Automatic reduction: `reduced`/);
  assert.match(plan.body, /canonical-corpus-dom-reduction-1234-2-/);
  assert.match(plan.body, /200 -> 40 bytes/);
  assert.match(plan.body, /Fixture promotion: `created`/);
  assert.match(plan.body, /https:\/\/example\.test\/pull\/93/);
  assert.doesNotMatch(plan.body, /<template>/);
  assert.throws(
    () =>
      createDivergenceIssue(evidence, ledger, selected, context, {
        ...receipt,
        fingerprint: "bad",
      }),
    /fingerprint differs/,
  );
});

await test("divergence evidence fails closed when samples or fixture custody differ", () => {
  assert.throws(
    () =>
      createDivergenceIssue(
        log(2, [`${fixtureA}/A.vue: old_len=1 new_len=2 first_diff=0 old_window=a new_window=b`]),
        ledger,
        selected,
        context,
      ),
    /samples are incomplete/,
  );
  assert.throws(
    () =>
      createDivergenceIssue(
        log(1, [
          "tests/_fixtures/_git/foreign/A.vue: old_len=1 new_len=2 first_diff=0 old_window=a new_window=b",
        ]),
        ledger,
        selected,
        context,
      ),
    /outside the fixture ledger/,
  );
  assert.throws(
    () => createDivergenceIssue(log(0), ledger, `${fixtureA}\n`, context),
    /Selected canonical fixture inventory is incomplete/,
  );
});

await test("publication reuses an exact open title and creates only when absent", () => {
  const plan = createDivergenceIssue(
    log(1, [`${fixtureA}/A.vue: old_len=1 new_len=2 first_diff=0 old_window=a new_window=b`]),
    ledger,
    selected,
    context,
  );
  let creates = 0;
  const existing = publishDivergenceIssue(plan, context, {
    findOpenIssue: () => ({ number: 91, html_url: "https://example.test/issues/91" }),
    createIssue: () => {
      creates += 1;
    },
  });
  assert.equal(existing.status, "existing");
  assert.equal(existing.issueNumber, 91);
  assert.equal(creates, 0);

  const created = publishDivergenceIssue(plan, context, {
    findOpenIssue: () => null,
    createIssue: (received) => {
      creates += 1;
      assert.equal(received, plan);
      return { number: 92, html_url: "https://example.test/issues/92" };
    },
  });
  assert.equal(created.status, "created");
  assert.equal(created.issueNumber, 92);
  assert.equal(creates, 1);
});
