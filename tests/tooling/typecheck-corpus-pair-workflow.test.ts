import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

void test("the corpus pair workflow builds both SHAs and measures on the 32-vCPU runner", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "typecheck-corpus-pair.yml")) as {
    on: {
      workflow_dispatch: {
        inputs: Record<string, { default?: string | boolean; required?: boolean }>;
      };
    };
    jobs: Record<
      string,
      { "runs-on": string; steps: Array<{ name?: string; if?: string; run?: string }> }
    >;
  };
  const inputs = workflow.on.workflow_dispatch.inputs;
  assert.equal(inputs.base_sha?.required, true);
  assert.equal(inputs.head_sha?.required, true);
  assert.equal(inputs.runs?.default, "5");
  assert.equal(inputs.workloads?.default, "");
  const job = workflow.jobs.measure;
  assert.equal(job?.["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  const script = (job?.steps ?? []).map((step) => step.run ?? "").join("\n");
  assert.match(script, /npm ci --prefix tools\/benchmarks\/typecheck-hosts/u);
  assert.match(script, /npm ci --prefix tools\/benchmarks\/typecheck-corpora/u);
  // vue-tsc comes from the benchmark workspace's lockfile pin.
  assert.match(
    script,
    /vp install --frozen-lockfile --prefer-offline --filter '\.\/tools\/benchmarks\/scripts'/u,
  );
  assert.match(
    script,
    /git submodule update --init --depth 1 -- tests\/_fixtures\/_git\/misskey tests\/_fixtures\/_git\/n8n tests\/_fixtures\/_git\/nuxt-ui tests\/_fixtures\/_git\/reka-ui$/mu,
  );
  // The target lane's tsrs 0.23 sits beside typecheck-hosts' 0.20.1.
  assert.match(
    script,
    /npm ci --prefix tools\/benchmarks\/typecheck-hosts-tsrs-0\.23 --ignore-scripts/u,
  );
  assert.equal(inputs.floors?.default, false);
  assert.equal(inputs.corpora?.default, "");
  assert.match(script, /if \[\[ "\$FLOORS" == true \]\]; then extra\+=\(--floors\); fi/u);
  // One SHA twice is an A/A run built once; owned pairs still need two builds.
  assert.match(script, /ln -s "\$RUNNER_TEMP\/pair-head" "\$RUNNER_TEMP\/pair-base"/u);
  assert.match(script, /\[\[ "\$OWNED_PAIR" != true \|\| "\$BASE_SHA" != "\$HEAD_SHA" \]\]/u);
  assert.match(script, /build:native/u);
  assert.match(script, /node tools\/benchmarks\/scripts\/typecheck-corpus-pair\.ts/u);
  assert.match(script, /--workloads "\$OWNED_WORKLOADS"/u);
  assert.equal(inputs.attribution?.default, true);
  assert.match(
    script,
    /node tools\/benchmarks\/scripts\/typecheck-cost-attribution\.ts --pair "\$RUNNER_TEMP\/pair-work" --out "\$RUNNER_TEMP\/pair-out"/u,
  );
  assert.match(
    script,
    /for report in cold summary projection reply-shape attribution batch-vs-mapper/u,
  );
  // Planted diagnostics run by default for misskey and n8n and append their table to summary.md.
  const planted = job?.steps.find(
    (step) => step.name === "Check planted diagnostics in the planted corpora",
  );
  assert.equal(
    planted?.if,
    "${{ !cancelled() && !inputs.owned_pair && steps.measure.outcome != 'skipped' }}",
  );
  assert.equal(
    planted?.run,
    'node tools/benchmarks/scripts/typecheck-corpus-mutations-run.ts --base "$RUNNER_TEMP/pair-base" --head "$RUNNER_TEMP/pair-head" --work "$RUNNER_TEMP/pair-work" --out "$RUNNER_TEMP/pair-out" --corpora "$CORPORA"',
  );
  assert.match(script, /tests\/tooling\/typecheck-corpus-mutations\.test\.ts/u);
  assert.match(script, /tests\/tooling\/typecheck-corpus-pair-cold\.test\.ts/u);
  assert.match(script, /tests\/tooling\/typecheck-corpus-pair-floors\.test\.ts/u);
});
