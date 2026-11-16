import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

void test("the scaling workflow supports isolated single and same-runner paired measurements", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "typecheck-scaling.yml")) as {
    on: { workflow_dispatch: { inputs: Record<string, { default?: string; required?: boolean }> } };
    jobs: Record<
      string,
      { if?: string; "runs-on": string; steps: Array<{ run?: string; uses?: string }> }
    >;
  };
  const inputs = workflow.on.workflow_dispatch.inputs;
  assert.equal(inputs.sha?.required, true);
  assert.equal(inputs.base_sha?.default, "");
  assert.equal(inputs.sizes?.default, "1000,5000,20000");
  assert.equal(inputs.runs?.default, "3");
  assert.equal(inputs.tools?.default, "");
  assert.equal(inputs.monorepo_sfcs?.default, "5000");
  assert.equal(inputs.nuxt_sizes?.default, "");
  const job = workflow.jobs.measure;
  assert.equal(job?.["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(job?.if, "github.event_name == 'workflow_dispatch' && inputs.base_sha == ''");
  const steps = job?.steps ?? [];
  const script = steps.map((step) => step.run ?? "").join("\n");
  assert.match(script, /npm ci --prefix tools\/benchmarks\/typecheck-hosts/u);
  assert.match(script, /npm ci --prefix tools\/benchmarks\/nuxt-app/u);
  assert.match(script, /GNU time/u);
  assert.match(script, /node --test tests\/tooling\/scale-project-generator\.test\.ts/u);
  assert.match(script, /vp install --frozen-lockfile/u);
  assert.match(script, /build:native/u);
  assert.match(script, /node tools\/benchmarks\/scripts\/typecheck-scaling\.ts/u);
  assert.match(script, /--tools "\$SCALING_TOOLS"/u);
  assert.ok(steps.some((step) => step.uses?.startsWith("actions/upload-artifact@")));

  const paired = workflow.jobs.paired;
  assert.equal(paired?.["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(paired?.if, "github.event_name == 'workflow_dispatch' && inputs.base_sha != ''");
  const pairedSteps = paired?.steps ?? [];
  const pairedScript = pairedSteps.map((step) => step.run ?? "").join("\n");
  assert.match(pairedScript, /pr-performance-build\.ts/u);
  assert.match(pairedScript, /typecheck-scaling-paired\.ts/u);
  assert.match(pairedScript, /--runs 5/u);
  assert.match(pairedScript, /--sizes "\$SIZES"/u);
  assert.match(pairedScript, /--provider-root "\$GITHUB_WORKSPACE\/driver"/u);
  assert.ok(pairedSteps.some((step) => step.uses?.startsWith("actions/upload-artifact@")));
});
