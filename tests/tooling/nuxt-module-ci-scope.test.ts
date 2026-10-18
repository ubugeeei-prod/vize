import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";
import {
  aggregateCheckNeedsResults,
  aggregateNeedsResults,
} from "../../tools/support/compat/github/require-needs-success.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  if?: string;
  run?: string;
  uses?: string;
  "working-directory"?: string;
  "continue-on-error"?: boolean;
  with?: { path?: string };
};
const workflow = parse(readRepoFile(".github", "workflows", "nuxt3-module-build.yml")) as {
  on: Record<string, { paths?: string[] }>;
  jobs: Record<string, { if?: string; "continue-on-error"?: boolean; steps: Step[] }>;
};
const job = workflow.jobs["nuxt3-module-build"];
const steps = job.steps;

function runsOn(step: Step, event: string) {
  if (!step.if || step.if === "always()") return true;
  const condition = step.if.match(/^\$\{\{ github\.event_name (==|!=) 'pull_request' \}\}$/);
  assert.ok(condition, `unrecognized Nuxt event condition: ${step.if}`);
  return condition[1] === "==" ? event === "pull_request" : event !== "pull_request";
}

const campaigns = (event: string) =>
  steps
    .filter((step) => runsOn(step, event))
    .flatMap((step) => step.run?.match(/compat\/nuxt\/([\w-]+-build\.mjs)/)?.[1] ?? []);

test("every affected PR retains source-built Nuxt 3 SSR, hydration and navigation", () => {
  assert.ok(workflow.on.pull_request.paths?.includes("crates/**"));
  assert.ok(workflow.on.pull_request.paths?.includes("davinci/**"));
  assert.ok(workflow.on.pull_request.paths?.includes("npm/framework/nuxt/**"));
  assert.equal(job.if, undefined);
  assert.notEqual(job["continue-on-error"], true);
  assert.deepEqual(campaigns("pull_request"), ["nuxt3-module-build.mjs"]);
  const native = steps.findIndex(
    (step) => step.name === "Build and capture the actual source native compiler",
  );
  const core = steps.findIndex((step) => step.run?.includes("nuxt3-module-build.mjs"));
  assert.ok(native >= 0 && core > native);
  assert.match(steps[native].run ?? "", /--no-cache --filter '\.\/npm\/native' build:debug/);
  for (const step of [steps[native], steps[core]]) {
    assert.equal(step.if, undefined);
    assert.notEqual(step["continue-on-error"], true);
  }
  const upload = steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.equal(upload?.if, "always()");
  assert.ok(upload?.with?.path?.includes("${{ runner.temp }}/nuxt3-module-proof"));
});

test("merge, main and manual runs retain every extended campaign and source receipt", () => {
  assert.ok(Object.hasOwn(workflow.on, "workflow_call"));
  assert.equal(Object.hasOwn(workflow.on, "merge_group"), false, "one required reusable execution");
  const source = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml"));
  const check = parse(readRepoFile(".github", "workflows", "check.yml"));
  assert.deepEqual(source.jobs["nuxt-module-qualification"], {
    name: "Nuxt module qualification",
    needs: "release-integration-catalog",
    if: "${{ github.event_name == 'merge_group' && needs.release-integration-catalog.outputs.product-qualification == '' }}",
    uses: "./.github/workflows/nuxt3-module-build.yml",
  });
  const condition = source.jobs["nuxt-module-qualification"].if
    .slice(3, -2)
    .replaceAll("needs.release-integration-catalog.outputs.product-qualification", "product");
  for (const event of [
    "pull_request",
    "merge_group",
    "push",
    "workflow_dispatch",
    "schedule",
    "unknown",
  ])
    for (const product of ["", "untrusted-nonempty", "null"])
      assert.equal(
        runInNewContext(condition, { github: { event_name: event }, product }),
        event === "merge_group" && product === "",
        `${event}: ${product}`,
      );
  assert.ok(source.jobs["source-report"].needs.includes("nuxt-module-qualification"));
  assert.ok(check.jobs["test-report"].needs.includes("pr-source-checks"));
  for (const event of ["merge_group", "push", "workflow_dispatch"]) {
    if (event !== "merge_group") assert.ok(Object.hasOwn(workflow.on, event));
    assert.deepEqual(campaigns(event), [
      "nuxt3-module-build.mjs",
      "prefetch-manifest-build.mjs",
      "lint-config-build.mjs",
      "compiler-defaults-build.mjs",
      "relative-glob-build.mjs",
      "javascript-workspace-build.mjs",
    ]);
    for (const step of steps.filter((item) => item.run?.includes("compat/nuxt/"))) {
      assert.notEqual(step["continue-on-error"], true);
      if (
        step.run?.match(
          /(?:lint-config|compiler-defaults|relative-glob|javascript-workspace)-build/,
        )
      ) {
        assert.ok(
          step.run.includes('"$RUNNER_TEMP/nuxt3-module-proof/source-native/custody.json"'),
        );
      }
    }
  }
});

test("failed, cancelled, missing and skipped Nuxt queue results cannot pass the required report", () => {
  const needs = (result: string) => ({
    "nuxt-module-qualification": { result },
  });
  for (const result of ["success", "failure", "cancelled", "skipped", "unknown"]) {
    const expected = result === "success" ? 0 : 1;
    const source = aggregateCheckNeedsResults(needs(result), "merge_group");
    assert.equal(source.exitCode, expected, result);
    assert.equal(
      aggregateNeedsResults({
        "pr-source-checks": { result: source.exitCode === 0 ? "success" : "failure" },
      }).exitCode,
      expected,
    );
  }
  assert.throws(
    () => aggregateCheckNeedsResults({}, "merge_group"),
    /missing nuxt-module-qualification/,
  );
  assert.throws(
    () =>
      aggregateCheckNeedsResults(
        { ...needs("success"), "nuxt-module-qualification": {} },
        "merge_group",
      ),
    /nuxt-module-qualification reported no result/,
  );
  assert.equal(aggregateCheckNeedsResults(needs("skipped"), "pull_request").exitCode, 0);
  for (const result of ["failure", "cancelled"])
    assert.equal(aggregateCheckNeedsResults(needs(result), "pull_request").exitCode, 1);

  for (const [rows, expected] of [
    [needs("success"), 0],
    [needs("failure"), 1],
    [needs("cancelled"), 1],
    [needs("skipped"), 1],
    [{}, 1],
  ] as const) {
    const run = spawnSync(
      process.execPath,
      ["tools/support/compat/github/require-needs-success.ts", "--check"],
      {
        cwd: root,
        encoding: "utf8",
        env: { ...process.env, GITHUB_EVENT_NAME: "merge_group", NEEDS_JSON: JSON.stringify(rows) },
      },
    );
    assert.equal(run.error, undefined);
    assert.equal(run.status, expected, run.stdout + run.stderr);
  }
});

test("PR browser setup avoids the extended CLI and fixture prerequisites", () => {
  const browser = steps.filter(
    (step) => runsOn(step, "pull_request") && step.run?.includes("playwright install"),
  );
  assert.equal(browser.length, 1);
  assert.equal(browser[0]["working-directory"], "tests");
  assert.equal(browser[0].run, "vize-ci-apt-retry vp exec playwright install --with-deps chromium");
  assert.notEqual(browser[0]["continue-on-error"], true);
  const prerequisites = steps.filter(
    (step) =>
      step.run?.includes("cargo build --profile ci -p vize") ||
      step.run?.includes("tests/differential/build-receipt.ts") ||
      [
        "tests/_fixtures/_projects/nuxt-critical-css-build",
        "tools/support/compat/nuxt/fixtures/vite-relative-glob",
      ].includes(step["working-directory"] ?? ""),
  );
  assert.equal(prerequisites.length, 4);
  for (const step of prerequisites) {
    assert.equal(runsOn(step, "pull_request"), false, step.name);
    for (const event of ["merge_group", "push", "workflow_dispatch"]) {
      assert.equal(runsOn(step, event), true, step.name);
    }
  }
});
