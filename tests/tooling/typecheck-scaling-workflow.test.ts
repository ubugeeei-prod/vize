import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

void test("the scaling workflow builds one SHA and measures every size on the 32-vCPU runner", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "typecheck-scaling.yml")) as {
    on: { workflow_dispatch: { inputs: Record<string, { default?: string; required?: boolean }> } };
    jobs: Record<string, { "runs-on": string; steps: Array<{ run?: string; uses?: string }> }>;
  };
  const inputs = workflow.on.workflow_dispatch.inputs;
  assert.equal(inputs.sha?.required, true);
  assert.equal(inputs.sizes?.default, "1000,5000,20000");
  assert.equal(inputs.runs?.default, "3");
  assert.equal(inputs.monorepo_sfcs?.default, "5000");
  const job = workflow.jobs.measure;
  assert.equal(job?.["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  const steps = job?.steps ?? [];
  const script = steps.map((step) => step.run ?? "").join("\n");
  assert.match(script, /npm ci --prefix tools\/benchmarks\/typecheck-hosts/u);
  assert.match(script, /GNU time/u);
  assert.match(script, /node --test tests\/tooling\/typecheck-scaling\.test\.ts/u);
  assert.match(script, /vp install --frozen-lockfile/u);
  assert.match(script, /build:native/u);
  assert.match(script, /node tools\/benchmarks\/scripts\/typecheck-scaling\.ts/u);
  assert.ok(steps.some((step) => step.uses?.startsWith("actions/upload-artifact@")));
});
