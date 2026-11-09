import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import {
  divergenceObservers,
  parseReachDivergences,
} from "../../tools/support/compat/github/canonical-corpus-divergence-observers.ts";
import { createDivergenceIssue } from "../../tools/support/compat/github/canonical-corpus-divergence-issue.mjs";
import { createPromotionPlan } from "../../tools/support/compat/github/canonical-corpus-divergence-promotion.mjs";
import {
  createReductionPlan,
  runReduction,
} from "../../tools/support/compat/github/canonical-corpus-divergence-reduction.mjs";

const { reach } = divergenceObservers;
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

type Shard = { inline: number | null; module?: number; lines?: string[] };

function counter(scope: string, shape: string, divergences: number): string {
  return `davinci production reach: shape=${shape} stage="L2" templates=3 accepted=3 permille=1000 ready=0 legacy={} rejected=0 routed_vapor=0 unrecorded=0 sfc_errors={} compared=3 divergences=${divergences}${scope}`;
}

// One committed and one canonical block per shard, as the reach sweep prints.
function reachLog(shards: Shard[]): string {
  return shards
    .flatMap((shard, index) => [
      `canonical observer process: test=davinci_production_reach role=shard index=${index} count=${shards.length} vector=reach-${index}.nul`,
      "davinci production reach: scope=committed files=2 unreadable=0 parse_errors=0 without_template=0",
      counter("", "dom_inline", 0),
      counter("", "dom_module", 0),
      "davinci production reach: scope=canonical files=3 unreadable=0 parse_errors=0 without_template=0",
      ...(shard.inline === null ? [] : [counter("", "dom_inline", shard.inline)]),
      counter("", "dom_module", shard.module ?? 0),
      'davinci production reach: shape=ssr stage="L4 plan" templates=3 accepted=3 permille=1000 ready=0 legacy={} rejected=0 routed_vapor=0 unrecorded=0 sfc_errors={} compared=0 divergences=0',
      ...(shard.lines ?? []),
      `canonical observer process exit: test=davinci_production_reach role=shard index=${index} status=0 signal=none elapsed_ms=1`,
    ])
    .join("\n");
}

const divergent = reachLog([
  { inline: 0 },
  {
    inline: 1,
    module: 1,
    lines: [
      "production reach selection/parity failures:",
      "airi/src/B.vue [dom_inline]: code differs at byte 12:",
      "--- s2",
      'x.vue [dom_module]: css differs <div class="a">',
      "--- legacy",
      "",
      'airi/src/A.vue [dom_module]: errors differ: s2=["x"] legacy=[]',
    ],
  },
]);

void test("reach evidence reads only canonical DOM shapes and fixture-relative names", () => {
  assert.deepEqual(parseReachDivergences(reachLog([{ inline: 0 }])), {
    divergenceCount: 0,
    samples: [],
  });
  assert.deepEqual(parseReachDivergences(divergent), {
    divergenceCount: 2,
    samples: [
      {
        sourcePath: `${fixture}/src/A.vue`,
        name: "airi/src/A.vue",
        shape: "dom_module",
        kind: "errors differ",
      },
      {
        sourcePath: `${fixture}/src/B.vue`,
        name: "airi/src/B.vue",
        shape: "dom_inline",
        kind: "code differs",
      },
    ],
  });
});

void test("reach evidence fails closed on missing counters or samples", () => {
  assert.throws(
    () => parseReachDivergences(reachLog([{ inline: null }])),
    /one canonical reach dom_inline counter in shard 0/,
  );
  assert.throws(
    () => parseReachDivergences(reachLog([{ inline: 1 }])),
    /dom_inline divergence samples are incomplete in shard 0/,
  );
});

void test("a reach difference is filed and reduced on its recorded shape and name", () => {
  const issue = createDivergenceIssue(reach, divergent, ledger, `${fixture}\n`, context);
  assert.ok(issue);
  assert.match(issue.title, /^fix\(davinci\): triage canonical production DOM divergence /);
  assert.match(issue.body, /canonical-corpus-reach-1234-2-/);
  assert.match(
    issue.body,
    /`tests\/_fixtures\/_git\/airi\/src\/A\.vue` \[dom_module\]: errors differ/,
  );
  assert.doesNotMatch(issue.body, /legacy=\[\]|class=/);

  const plan = createReductionPlan(reach, divergent, ledger, `${fixture}\n`, context);
  assert.deepEqual(plan.oracle, { shape: "dom_module", name: "airi/src/A.vue" });
  const directory = mkdtempSync("tests/.canonical-reach-reduction-");
  try {
    const sourcePath = `${directory}/A.vue`;
    const output = `${directory}/out`;
    const vize = `${directory}/vize`;
    mkdirSync(output);
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
    assert.deepEqual(receipt.oracle, plan.oracle);
    assert.equal(
      readFileSync(`${output}/args.txt`, "utf8").split("\n")[3],
      `'${directory}/examples/dom_production_divergence' --shape 'dom_module' --name 'airi/src/A.vue'`,
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

void test("a reduced reach witness keeps its shape and name in provenance", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-reach-promotion-"));
  try {
    const witness = "<template><b>bold</b></template>";
    writeFileSync(join(directory, "canonical-reach-divergence.reduced.vue"), witness);
    writeFileSync(
      join(directory, "reduction.json"),
      JSON.stringify({
        schema: "vize.realProjectDivergenceReductionReceipt",
        version: 1,
        observer: "reach",
        status: "reduced",
        divergenceCount: 1,
        fingerprint: "abc123abc123",
        sourcePath: `${fixture}/src/A.vue`,
        oracle: { shape: "dom_module", name: "airi/src/A.vue" },
        sourceBytes: 200,
        reducedFile: "canonical-reach-divergence.reduced.vue",
        reducedBytes: Buffer.byteLength(witness),
        reducedSha256: createHash("sha256").update(witness).digest("hex"),
        sourceCommit: context.sourceCommit,
        runUrl: context.runUrl,
      }),
    );
    const plan = createPromotionPlan(directory);
    assert.ok(plan);
    assert.equal(plan.branch, "automation/davinci-reach-abc123abc123");
    assert.match(plan.body, /promoted_real_project_reach_fixtures_agree/);
    const provenance = JSON.parse(plan.files[1].content);
    assert.equal(
      plan.files[1].path,
      "tests/_fixtures/differential/compiler/real-project-reach-reductions/abc123abc123/provenance.json",
    );
    assert.equal(provenance.observer, "reach");
    assert.equal(provenance.shape, "dom_module");
    assert.equal(provenance.name, "airi/src/A.vue");
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
