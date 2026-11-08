import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

void test("the corpus pair workflow builds both SHAs and measures on the 32-vCPU runner", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "typecheck-corpus-pair.yml")) as {
    on: { workflow_dispatch: { inputs: Record<string, { default?: string; required?: boolean }> } };
    jobs: Record<string, { "runs-on": string; steps: Array<{ run?: string }> }>;
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
    /git submodule update --init --depth 1 -- tests\/_fixtures\/_git\/misskey tests\/_fixtures\/_git\/n8n/u,
  );
  assert.match(script, /build:native/u);
  assert.match(script, /node tools\/benchmarks\/scripts\/typecheck-corpus-pair\.ts/u);
  assert.match(script, /--workloads "\$OWNED_WORKLOADS"/u);
});
