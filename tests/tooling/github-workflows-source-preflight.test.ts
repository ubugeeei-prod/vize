import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile, root } from "./support/github-workflows.ts";

interface Step {
  readonly name?: string;
  readonly uses?: string;
  readonly run?: string;
  readonly if?: string;
  readonly "continue-on-error"?: boolean;
}
interface Job {
  readonly if?: string;
  readonly needs?: readonly string[] | string;
  readonly steps?: readonly Step[];
}
const source = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as {
  jobs: Record<string, Job>;
};

function rejectFailedPlan(
  event: string,
  failed: Readonly<Record<string, Readonly<{ result: string }>>>,
): void {
  const run = spawnSync(
    process.execPath,
    ["tools/support/compat/github/canonical-corpus-selection.mjs", "--check"],
    {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, NEEDS_JSON: JSON.stringify(failed), GITHUB_EVENT_NAME: event },
    },
  );
  assert.equal(run.status, 1, run.stderr);
  assert.match(run.stderr, /corpus plan did not succeed/u);
}

test("failed source preflights block every source build and cannot pass the source report", () => {
  const plan = source.jobs["pr-source-plan"];
  const steps = plan.steps ?? [];
  const runtime = steps.findIndex(
    (step) => step.uses?.startsWith("voidzero-dev/setup-vp@") === true,
  );
  const base = steps.findIndex((step) => step.name === "Resolve and fetch comparison base");
  const inventory = steps.findIndex(
    (step) => step.uses === "./.github/actions/check-level-inventories",
  );
  const layout = steps.findIndex(
    (step) => step.run === "node --test tests/tooling/davinci-module-layout.test.ts",
  );
  const output = steps.findIndex((step) => step.name === "Plan source checks");
  assert.ok(runtime >= 0 && runtime < base && base < inventory && inventory < layout);
  assert.ok(layout < output, "preflights must finish before source selection and Rust setup");
  for (const step of [steps[inventory], steps[layout]]) {
    assert.equal(step.if, undefined, "source preflights cannot skip");
    assert.equal(step["continue-on-error"], undefined, "source preflight failures are required");
  }
  for (const lane of [
    "pr-rust-source",
    "pr-js-packages",
    "pr-tooling-scripts",
    "pr-playground-test",
  ]) {
    assert.deepEqual(
      source.jobs[lane].needs,
      lane === "pr-tooling-scripts" ? ["pr-source-plan", "pr-source-cli"] : "pr-source-plan",
      lane,
    );
    assert.equal(source.jobs[lane].if, plan.if, `${lane} retains implicit success() gating`);
  }
  assert.equal(source.jobs["pr-source-cli"].needs, "pr-source-plan");
  assert.equal(source.jobs["pr-source-cli"].if, undefined, "producer retains implicit success()");
  const report = source.jobs["source-report"];
  const dependencies = typeof report.needs === "string" ? [report.needs] : (report.needs ?? []);
  assert.equal(report.if, "${{ always() }}");
  assert.ok(dependencies.includes("pr-source-plan"));
  assert.equal(
    report.steps?.at(-1)?.run,
    "node tools/support/compat/github/canonical-corpus-selection.mjs --check",
  );
  const failed = Object.fromEntries(
    dependencies.map((lane) => [
      lane,
      { result: lane === "pr-source-plan" ? "failure" : "skipped" },
    ]),
  );
  for (const event of ["pull_request", "merge_group"]) {
    rejectFailedPlan(event, failed);
  }
});
