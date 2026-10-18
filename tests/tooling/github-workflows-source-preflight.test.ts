import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { checkWorkflowJobSources, readRepoFile, root } from "./support/github-workflows.ts";

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

test("a failed source plan blocks every source build and cannot pass the source report", () => {
  const plan = source.jobs["pr-source-plan"];
  const steps = plan.steps ?? [];
  const runtime = steps.findIndex((step) => step.uses?.startsWith("actions/setup-node@") === true);
  const base = steps.findIndex((step) => step.name === "Resolve and fetch comparison base");
  const output = steps.findIndex((step) => step.name === "Plan source checks");
  assert.ok(runtime >= 0 && runtime < base && base < output);
  for (const step of [steps[base], steps[output]]) {
    assert.equal(step.if, undefined, "comparison and selection cannot skip");
    assert.equal(step["continue-on-error"], undefined, "plan failures are required");
  }
  for (const lane of ["pr-rust-source", "pr-js-packages", "pr-playground-test"]) {
    assert.deepEqual(source.jobs[lane].needs, "pr-source-plan", lane);
    assert.equal(source.jobs[lane].if, plan.if, `${lane} retains implicit success() gating`);
  }
  assert.equal(source.jobs["pr-native-runtime"].needs, "pr-source-plan");
  assert.equal(
    source.jobs["pr-native-runtime"].if,
    undefined,
    "native producer retains implicit success()",
  );
  const report = source.jobs["source-report"];
  const dependencies = typeof report.needs === "string" ? [report.needs] : (report.needs ?? []);
  assert.equal(report.if, "${{ always() && !contains(needs.*.result, 'cancelled') }}");
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

test("full checks retain required inventory preflights and exhaustive tooling behind shared callers", () => {
  const check = parse(checkWorkflowJobSources()) as {
    jobs: Record<string, Job & { uses?: string }>;
  };
  const steps = check.jobs["check-js"].steps ?? [];
  const inventory = steps.findIndex(
    (step) => step.uses === "./.github/actions/check-level-inventories",
  );
  const fullCheck = steps.findIndex((step) => step.name === "Check JS/TS");
  assert.ok(inventory >= 0 && inventory < fullCheck);
  assert.equal(
    steps[inventory].if,
    "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
  );
  assert.equal(steps[inventory]["continue-on-error"], undefined);
  const action = parse(readRepoFile(".github", "actions", "check-level-inventories", "action.yml"));
  assert.equal(action.runs.steps[0].run, "node tools/support/compat/davinci/source-inventories.ts");
  assert.equal(action.runs.steps[0]["continue-on-error"], undefined);

  assert.equal(check.jobs["full-source-cli"].uses, "./.github/workflows/full-source-cli.yml");
  assert.equal(check.jobs["tooling-plan"].uses, "./.github/workflows/full-tooling-plan.yml");
  assert.deepEqual(check.jobs["test-scripts"].needs, ["full-source-cli", "tooling-plan"]);
  assert.ok(
    check.jobs["test-scripts"].steps?.some(
      (step) => step.uses === "./.github/actions/test-full-tooling",
    ),
  );
  const worker = parse(readRepoFile(".github", "actions", "test-full-tooling", "action.yml"));
  const select = worker.runs.steps.findIndex(
    (step: Step) => step.name === "Select the exhaustive full tooling inventory",
  );
  const execute = worker.runs.steps.findIndex((step: Step) => step.name === "Test release scripts");
  assert.ok(select >= 0 && select < execute);
  assert.match(worker.runs.steps[select].run, /plan-tooling-tests\.mjs 0{40}.*--tier merge/);
  assert.equal(worker.runs.steps[execute].run, "vp run --workspace-root test:scripts:planned");
  assert.equal(worker.runs.steps[execute].env.VIZE_TOOLING_TEST_TIER, "merge");
  assert.equal(worker.runs.steps[execute]["continue-on-error"], undefined);
  const temporary = mkdtempSync(join(tmpdir(), "full-tooling-inventory-"));
  try {
    const output = join(temporary, "plan.json");
    const result = spawnSync(
      process.execPath,
      [
        "tools/support/compat/github/plan-tooling-tests.mjs",
        "0".repeat(40),
        "1".repeat(40),
        "--tier",
        "merge",
        "--output",
        output,
      ],
      {
        cwd: root,
        encoding: "utf8",
        env: { ...process.env, GITHUB_OUTPUT: "", GITHUB_STEP_SUMMARY: "" },
      },
    );
    assert.equal(result.status, 0, result.stderr);
    const plan: unknown = JSON.parse(readFileSync(output, "utf8"));
    assert.ok(plan !== null && typeof plan === "object" && "tests" in plan);
    assert.ok(Array.isArray(plan.tests));
    const selected: unknown[] = plan.tests;
    assert.ok(selected.every((file): file is string => typeof file === "string"));
    const inventory = ["tests/tooling", "tests/tooling/davinci", "tests/tooling/release"]
      .flatMap((directory) =>
        readdirSync(join(root, directory))
          .filter((name) => /\.test\.(?:ts|mjs)$/.test(name))
          .map((name) => `${directory}/${name}`),
      )
      .sort();
    assert.deepEqual(selected, inventory);
    for (const file of ["davinci-module-layout.test.ts", "retired-tooling-languages.test.ts"]) {
      assert.ok(selected.includes(`tests/tooling/${file}`), `${file} must remain in full health`);
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
  assert.deepEqual(check.jobs["test-scripts-gate"].needs, [
    "full-source-cli",
    "tooling-plan",
    "test-scripts",
  ]);
  assert.match(check.jobs["test-scripts-gate"].if ?? "", /always\(\)/);
  assert.ok(
    check.jobs["test-scripts-gate"].steps?.some(
      (step) => step.uses === "./.github/actions/report-full-js-qualification",
    ),
  );
});
