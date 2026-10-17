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

test("source planning is required before product builds and a failed plan cannot pass the report", () => {
  const plan = source.jobs["pr-source-plan"];
  const steps = plan.steps ?? [];
  const required = steps.filter((step) =>
    /(?:comparison-base\.ts|plan-pr-checks\.ts|canonical-corpus-selection\.mjs)/.test(
      step.run ?? "",
    ),
  );
  assert.equal(required.length, 3);
  for (const step of required) {
    assert.equal(step.if, undefined, "planning cannot skip");
    assert.notEqual(step["continue-on-error"], true, "planning failures are required");
  }
  for (const lane of [
    "pr-rust-source",
    "pr-js-packages",
    "pr-native-runtime",
    "pr-playground-test",
  ]) {
    assert.equal(source.jobs[lane].needs, "pr-source-plan", lane);
    assert.ok(
      source.jobs[lane].if === undefined || source.jobs[lane].if === plan.if,
      `${lane} retains implicit success() gating`,
    );
  }
  const report = source.jobs["source-report"];
  const dependencies = typeof report.needs === "string" ? [report.needs] : (report.needs ?? []);
  assert.equal(
    report.if,
    "${{ always() && github.event_name == 'merge_group' && !contains(needs.*.result, 'cancelled') }}",
  );
  assert.ok(dependencies.includes("pr-source-plan"));
  assert.equal(
    report.steps?.at(-1)?.run,
    "node tools/support/compat/github/canonical-corpus-selection.mjs --check",
  );
  for (const event of ["pull_request", "merge_group"]) {
    for (const result of ["failure", "cancelled", "skipped", undefined]) {
      const failed = Object.fromEntries(dependencies.map((lane) => [lane, { result: "success" }]));
      if (result) failed["pr-source-plan"] = { result };
      else delete failed["pr-source-plan"];
      rejectFailedPlan(event, failed);
    }
  }
});
