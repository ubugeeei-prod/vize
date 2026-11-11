import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

void test("the type-check profile workflow builds unstripped frame-pointer Vize and records whole trees", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "typecheck-profile.yml")) as {
    on: {
      push?: { branches?: string[] };
      workflow_dispatch: {
        inputs: Record<string, { default?: string | boolean; required?: boolean }>;
      };
    };
    jobs: Record<
      string,
      {
        "runs-on": string;
        env: Record<string, string>;
        steps: Array<{ name?: string; if?: string; run?: string; uses?: string }>;
      }
    >;
  };
  const inputs = workflow.on.workflow_dispatch.inputs;
  // A bench branch push profiles the pushed commit; trailers stand in for the inputs.
  assert.deepEqual(workflow.on.push?.branches, ["bench/typecheck-profile/**"]);
  assert.equal(inputs.head_sha?.required, true);
  assert.equal(inputs.base_sha?.default, "");
  assert.equal(inputs.corpora?.default, "");
  assert.equal(inputs.lanes?.default, "tsc,tsrs,check");
  const job = workflow.jobs.profile;
  assert.equal(job?.["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.match(job?.env.FLAMEGRAPH_SHA ?? "", /^[0-9a-f]{40}$/u);
  const script = (job?.steps ?? []).map((step) => step.run ?? "").join("\n");
  // Corpora come from the corpus pair's list, so a corpus added there is hydrated here.
  assert.match(script, /typecheck-profile\.ts --print-submodules --corpora "\$CORPORA"/u);
  assert.match(script, /npm ci --prefix tools\/benchmarks\/typecheck-hosts/u);
  assert.match(script, /npm ci --prefix tools\/benchmarks\/typecheck-corpora/u);
  // Rust stacks unwind through frame pointers and keep their symbol table.
  assert.match(script, /RUSTFLAGS="\$\{RUSTFLAGS:-\} -C force-frame-pointers=yes"/u);
  assert.match(script, /CARGO_PROFILE_CI_OPT_STRIP=none/u);
  assert.match(script, /napi build --platform --profile ci-opt/u);
  // Kernel frames and kernel symbols are visible to the unprivileged recorder.
  assert.match(script, /kernel\.perf_event_paranoid=-1 kernel\.kptr_restrict=0/u);
  assert.match(script, /linux-tools-generic libc6-dbg/u);
  assert.match(script, /--perf "\$PERF_BIN" --flamegraph "\$RUNNER_TEMP\/flamegraph"/u);
  assert.match(script, /node --test tests\/tooling\/typecheck-profile\.test\.ts/u);
  const upload = job?.steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.equal(upload?.if, "always()");
});
