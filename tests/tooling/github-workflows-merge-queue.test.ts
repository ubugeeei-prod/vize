import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { aggregateNeedsResults } from "../../tools/support/compat/github/require-needs-success.mjs";
import { readRepoFile, root } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  if?: string;
  run?: string;
  uses?: string;
  env?: Record<string, string>;
  with?: Record<string, string>;
  "continue-on-error"?: boolean;
};
type Job = { if?: string; needs?: string[] | string; steps?: Step[]; uses?: string };
const source = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as {
  jobs: Record<string, Job>;
};
const check = parse(readRepoFile(".github", "workflows", "check.yml")) as {
  jobs: Record<string, Job>;
};
const recipe = parse(
  readRepoFile(".github", "actions", "test-rust-workspace-differential", "action.yml"),
) as {
  inputs: Record<string, { default: string }>;
  runs: { using: string; steps: Step[] };
};
const actionPath = "./.github/actions/test-rust-workspace-differential";
const lanes = [
  "pr-source-plan",
  "pr-rust-source",
  "pr-js-packages",
  "pr-tooling-scripts",
  "pr-playground-test",
];
const tailCommands = [
  "cargo test -p vize_l1_to_l2 --features davinci-differential --test davinci_lowering_corpus -- --nocapture",
  "cargo test -p vize_l1_to_l2 --features davinci-differential --test davinci_dom_corpus -- --nocapture",
  "cargo test -p vize_l1_to_l2 --features davinci-differential --test davinci_remarks_corpus -- --nocapture",
  "cargo test -p vize_l1_to_l2 --features davinci-differential --test davinci_pug_corpus -- --nocapture",
  "cargo test -p vize_atelier_sfc --features davinci-dom-differential --test davinci_production_reach -- --nocapture",
  'VIZE_DAVINCI_DIFFERENTIAL_CORPUS="$PWD" cargo test -p vize_atelier_ssr --features davinci-differential --test davinci_ssr_corpus -- --nocapture',
  "cargo test -p vize_atelier_jsx --features davinci-differential --lib vdom::l2_differential::l2_vdom_admitted_cases_match_relief_codegen -- --exact",
  "cargo test -p vize --features legacy --test check_nuxt_tsconfig_isolation_cli",
  "cargo test -p vize_atelier_core --features legacy --test davinci_l2_transform_vue2",
  "cargo test -p vize_vitrine --no-default-features --features wasm",
  'VIZE_DAVINCI_DIFFERENTIAL_CORPUS="$PWD" cargo test -p vize_patina --features davinci-differential --test davinci_markup_differential -- --nocapture',
];
const commands = (job: Job) => (job.steps ?? []).map((step) => step.run ?? "").join("\n");

test("queue scope reaches the planner and every full source lane remains required", () => {
  assert.equal(
    source.jobs["pr-source-plan"].steps?.find((step) => step.name === "Plan source checks")?.run,
    'node tools/support/compat/github/plan-source-checks.mjs "$BASE_SHA" "$GITHUB_SHA" "$GITHUB_EVENT_NAME"',
  );
  assert.deepEqual(source.jobs["source-report"].needs, lanes);
  assert.equal(source.jobs["source-report"].if, "${{ always() }}");
  assert.match(commands(source.jobs["source-report"]), /require-needs-success\.mjs/);
  assert.ok(check.jobs["test-report"].needs?.includes("pr-source-checks"));
  for (const required of [
    "fmt-rust",
    "check-js",
    "security-audit",
    "node-engine-compat",
    "check-vize-apps",
  ]) {
    assert.ok(check.jobs["test-report"].needs?.includes(required), `${required} remains mandatory`);
  }
  assert.equal(check.jobs["pr-source-checks"].uses, "./.github/workflows/pr-source-checks.yml");
  assert.match(commands(source.jobs["pr-js-packages"]), /vp run --workspace-root test:js/);
  assert.match(commands(source.jobs["pr-tooling-scripts"]), /vp run --workspace-root test:scripts/);
  assert.match(commands(source.jobs["pr-tooling-scripts"]), /cargo build --profile ci -p vize/);
  assert.match(commands(source.jobs["pr-playground-test"]), /test:browser/);
  assert.match(commands(source.jobs["pr-playground-test"]), /test:vrt/);
  const vrtFailure = source.jobs["pr-playground-test"].steps?.find(
    (step) => step.name === "Fail if VRT failed",
  );
  assert.match(vrtFailure?.if ?? "", /steps\.vrt\.outcome == 'failure'/);
  assert.equal(vrtFailure?.run, "exit 1");
});

test("queue Rust retains prerequisites and executes the shared feature tail after the workspace", () => {
  const steps = source.jobs["pr-rust-source"].steps ?? [];
  const pkl = steps.findIndex((step) => step.name === "Install Pkl CLI");
  const workspace = steps.findIndex((step) =>
    /cargo test --workspace(?:;|$)/m.test(step.run ?? ""),
  );
  const tail = steps.findIndex((step) => step.uses === actionPath);
  const coverage = steps.findIndex((step) => step.name === "Check fixture coverage");
  assert.ok(pkl >= 0 && pkl < workspace && workspace < tail && tail < coverage);
  assert.equal(steps[workspace].env?.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(
    steps[tail].if,
    "${{ github.event_name == 'merge_group' && needs.pr-source-plan.outputs.rust == 'true' }}",
  );
  assert.equal(steps[tail].with?.["workspace-already-tested"], "true");
  assert.notEqual(steps[tail]["continue-on-error"], true);
  const manual = check.jobs["clippy-and-test"].steps?.find((step) => step.name === "Test");
  assert.equal(manual?.uses, actionPath);
  assert.equal(manual?.with?.["workspace-already-tested"], undefined);
  assert.equal(recipe.inputs["workspace-already-tested"].default, "false");
  assert.equal(recipe.runs.using, "composite");
  assert.equal(recipe.runs.steps[0].if, "${{ inputs.workspace-already-tested != 'true' }}");
  assert.equal(recipe.runs.steps[0].run, "cargo test --workspace");
  assert.deepEqual(recipe.runs.steps[1].run?.trim().split("\n"), tailCommands);
  for (const step of recipe.runs.steps) {
    assert.equal(step.env?.VIZE_TEST_REQUIRE_TSGO, "1");
    assert.notEqual(step["continue-on-error"], true);
  }
});

test("both queue reports reject failed, cancelled, skipped and missing actual dependencies", () => {
  for (const job of [source.jobs["source-report"], check.jobs["test-report"]]) {
    const dependencies = job.needs;
    assert.ok(Array.isArray(dependencies) && dependencies.length > 0);
    const needs = () =>
      Object.fromEntries(dependencies.map((lane) => [lane, { result: "success" }]));
    assert.equal(aggregateNeedsResults(needs()).exitCode, 0);
    for (const lane of dependencies) {
      for (const result of ["failure", "cancelled", "skipped", "unknown"]) {
        assert.equal(aggregateNeedsResults({ ...needs(), [lane]: { result } }).exitCode, 1);
      }
      assert.throws(() => aggregateNeedsResults({ ...needs(), [lane]: {} }), /reported no result/);
    }
  }
});

test("the shared bash recipe stops at a failed feature command (simulated cargo)", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-queue-recipe-"));
  try {
    const log = join(cwd, "commands.log");
    writeFileSync(
      join(cwd, "cargo"),
      '#!/bin/sh\nprintf "%s|%s|%s\\n" "$*" "$VIZE_TEST_REQUIRE_TSGO" "${VIZE_DAVINCI_DIFFERENTIAL_CORPUS-}" >> "$VIZE_GATE_TEST_LOG"\ncase "$*" in *"$VIZE_GATE_TEST_FAIL"*) exit 42;; esac\n',
      { mode: 0o755 },
    );
    for (const [failure, count, status] of [
      ["davinci_remarks_corpus", 3, 42],
      ["no-such-command", 11, 0],
    ] as const) {
      writeFileSync(log, "");
      const run = spawnSync(
        "/bin/bash",
        ["--noprofile", "--norc", "-eo", "pipefail", "-c", recipe.runs.steps[1].run!],
        {
          cwd: root,
          encoding: "utf8",
          env: {
            PATH: cwd,
            VIZE_TEST_REQUIRE_TSGO: "1",
            VIZE_GATE_TEST_LOG: log,
            VIZE_GATE_TEST_FAIL: failure,
          },
        },
      );
      assert.equal(run.status, status, run.stderr);
      const lines = readFileSync(log, "utf8").trim().split("\n");
      assert.equal(lines.length, count);
      assert.ok(lines.every((line) => line.includes("|1|")));
      if (status === 0) {
        assert.ok(lines[5].endsWith(`|${root}`));
        assert.ok(lines[10].endsWith(`|${root}`));
      }
    }
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});
