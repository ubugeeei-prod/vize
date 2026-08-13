import assert from "node:assert/strict";
import { test } from "node:test";

import { recordRunnerBaseline, vueParityAction } from "./support/check-vue-parity-action.ts";

test("the vue-parity runner baseline caps process pools only on GitHub-hosted runners", () => {
  const step = vueParityAction().runs?.steps?.[0];
  assert.equal(step?.name, "Record runner process budget baseline");
  const script = step?.run ?? "";
  assert.doesNotMatch(
    script,
    /\$\{\{/,
    "the baseline script must read its inputs from env, so both runner profiles stay executable",
  );

  const hosted = recordRunnerBaseline(script, "github-hosted");
  assert.equal(
    hosted.githubEnv.get("RAYON_NUM_THREADS"),
    "1",
    "GitHub-hosted runners must export a single Rayon worker to every later step",
  );
  assert.equal(
    hosted.githubEnv.get("GOMAXPROCS"),
    "1",
    "GitHub-hosted runners must export Corsa's Go runtime cap to every later step",
  );
  assert.equal(
    hosted.githubEnv.get("VIZE_CHECK_FIXTURES_BUDGET_CPU_FLOOR"),
    "12",
    "GitHub-hosted runners must export the temporary cycle-budget floor",
  );
  assert.equal(
    hosted.githubEnv.get("VIZE_TIER_L_BUDGET_SCALE"),
    "2",
    "GitHub-hosted runners must export the temporary Tier-L budget scale",
  );
  assert.equal(
    hosted.githubEnv.get("VIZE_PERF_BUDGET_SCALE"),
    "2",
    "GitHub-hosted runners must export the temporary LSP performance budget scale",
  );
  assert.equal(hosted.baseline.get("runner_environment"), "github-hosted");
  assert.equal(hosted.baseline.get("rayon_num_threads"), "1");
  assert.equal(hosted.baseline.get("gomaxprocs"), "1");
  assert.equal(hosted.baseline.get("budget_cpu_floor"), "12");
  assert.equal(hosted.baseline.get("tier_l_budget_scale"), "2");
  assert.equal(hosted.baseline.get("perf_budget_scale"), "2");
  // The settle step compares live pressure against this number, so it has to be
  // an integer and it has to be the same count the artifact reports.
  assert.match(hosted.githubEnv.get("VIZE_RUNNER_BASELINE_THREADS") ?? "", /^[0-9]+$/);
  assert.equal(
    hosted.githubEnv.get("VIZE_RUNNER_BASELINE_THREADS"),
    hosted.baseline.get("threads_total"),
    "the exported thread baseline must match the count recorded in the artifact",
  );

  const selfHosted = recordRunnerBaseline(script, "self-hosted");
  assert.equal(
    selfHosted.githubEnv.get("RAYON_NUM_THREADS"),
    "4",
    "self-hosted runners must keep the established Rayon cap",
  );
  assert.equal(
    selfHosted.githubEnv.has("GOMAXPROCS"),
    false,
    "the hosted-runner Go runtime cap must not leak onto self-hosted runners",
  );
  assert.equal(
    selfHosted.githubEnv.has("VIZE_CHECK_FIXTURES_BUDGET_CPU_FLOOR"),
    false,
    "the hosted-runner cycle-budget floor must not leak onto self-hosted runners",
  );
  assert.equal(
    selfHosted.githubEnv.has("VIZE_TIER_L_BUDGET_SCALE"),
    false,
    "the hosted-runner Tier-L budget scale must not leak onto self-hosted runners",
  );
  assert.equal(
    selfHosted.githubEnv.has("VIZE_PERF_BUDGET_SCALE"),
    false,
    "the hosted-runner LSP budget scale must not leak onto self-hosted runners",
  );
  assert.equal(selfHosted.baseline.get("runner_environment"), "self-hosted");
  assert.equal(selfHosted.baseline.get("rayon_num_threads"), "4");
  assert.equal(selfHosted.baseline.get("gomaxprocs"), "unset");
  assert.equal(selfHosted.baseline.get("budget_cpu_floor"), "unset");
  assert.equal(selfHosted.baseline.get("tier_l_budget_scale"), "unset");
  assert.equal(selfHosted.baseline.get("perf_budget_scale"), "unset");

  // The artifact is the only evidence left when a later step is killed by the
  // spawn exhaustion of #4126, so every process-budget fact must be emitted.
  for (const fact of [
    "cpus",
    "ulimit_u",
    "ulimit_Hu",
    "cgroup",
    "pids_current",
    "pids_max",
    "threads_total",
  ]) {
    assert.ok(hosted.baseline.has(fact), `the runner baseline must record ${fact}`);
  }
});
