import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import {
  aggregateCheckNeedsResults,
  aggregateNeedsResults,
} from "../../tools/support/compat/github/require-needs-success.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";

type Workflow = {
  on: Record<string, unknown>;
  concurrency: { group: string; "cancel-in-progress": boolean };
  permissions: Record<string, string>;
  jobs: Record<
    string,
    {
      if?: string;
      uses?: string;
      needs?: string[];
      steps?: {
        name?: string;
        if?: string;
        run?: string;
        uses?: string;
        with?: Record<string, unknown>;
        "continue-on-error"?: boolean;
      }[];
    }
  >;
};
const check = parse(readRepoFile(".github/workflows/check.yml")) as Workflow;
const source = parse(readRepoFile(".github/workflows/pr-source-checks.yml")) as Workflow;
const topPrior = [
  "iteration-plan",
  "fmt-rust",
  "check-js",
  "security-audit",
  "node-engine-compat",
  "check-vize-apps",
  "pr-source-checks",
  "instruction-counts",
  "level-dependency-direction",
];
const prior = [
  "pr-source-plan",
  "pr-rust-source",
  "pr-rust-products",
  "pr-js-packages",
  "pr-native-runtime",
  "pr-playground-test",
];
const needs = (qualification = "success") => ({
  ...Object.fromEntries(prior.map((job) => [job, { result: "success" }])),
  "nuxt-module-qualification": { result: qualification },
});

test("retiring the WIT lane preserves the required source report and existing Check controls", () => {
  assert.equal(Object.hasOwn(check.jobs, "wit-contracts"), false);
  assert.deepEqual(check.jobs["pr-source-checks"], {
    name: "PR source checks",
    permissions: { contents: "read", actions: "read", checks: "read", "pull-requests": "read" },
    if: "${{ github.event_name == 'pull_request' || github.event_name == 'merge_group' }}",
    uses: "./.github/workflows/pr-source-checks.yml",
  });
  assert.deepEqual(source.on, {
    workflow_call: {
      outputs: {
        integration: { value: "${{ jobs.release-integration-catalog.outputs.integration }}" },
        qualification: { value: "${{ jobs.release-integration-catalog.outputs.qualification }}" },
        "product-qualification": {
          value: "${{ jobs.release-integration-catalog.outputs.product-qualification }}",
        },
      },
    },
  });
  assert.equal(Object.hasOwn(source.jobs, "wit-contracts"), false);
  assert.deepEqual(source.jobs["source-report"].needs, [
    ...prior,
    "nuxt-module-qualification",
    "canonical-corpus",
    "release-integration-catalog",
    "ci-selection-tests",
  ]);
  assert.equal(
    source.jobs["source-report"].if,
    "${{ always() && !contains(needs.*.result, 'cancelled') }}",
  );
  assert.equal(
    source.jobs["source-report"].steps?.at(-1)?.run,
    "node tools/support/compat/github/canonical-corpus-selection.mjs --check",
  );
  assert.deepEqual(check.jobs["test-report"].needs, topPrior);
  assert.equal(
    check.jobs["test-report"].if,
    "${{ always() && (github.event_name == 'pull_request' || github.event_name == 'merge_group') }}",
  );
  const reportSteps = check.jobs["test-report"].steps ?? [];
  const aggregate = reportSteps.find((step) => step.name === "Require every needed job to succeed");
  const select = reportSteps.find(
    (step) => step.name === "Re-select the current integration before the required final gate",
  );
  const final = reportSteps.find(
    (step) =>
      step.name === "Freshly verify the same exact qualification at the required final gate",
  );
  const custody = reportSteps.find(
    (step) => step.name === "Retain the required final qualification observations",
  );
  assert.ok(aggregate && select && final && custody);
  assert.equal(
    aggregate.run,
    "node tools/support/compat/github/require-needs-success.ts --check-report",
  );
  assert.ok(reportSteps.indexOf(aggregate) < reportSteps.indexOf(select));
  assert.ok(reportSteps.indexOf(select) < reportSteps.indexOf(final));
  assert.equal(select.if, "${{ github.event_name == 'merge_group' }}");
  assert.equal(select.run, "node tools/support/compat/github/release-integration-event.ts");
  assert.equal(final.if, "${{ github.event_name == 'merge_group' }}");
  assert.ok(
    final.run?.includes(
      'integration-release.ts --verify < "$directory/qualification.json" > "$directory/final-qualification.json"',
    ),
  );
  for (const step of [aggregate, select, final]) assert.equal(step["continue-on-error"], undefined);
  assert.equal(reportSteps.at(-2), final);
  assert.equal(reportSteps.at(-1), custody);
  assert.equal(custody.if, "${{ always() && github.event_name == 'merge_group' }}");
  assert.equal(custody.uses, "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a");
  assert.equal(custody.with?.path, "${{ runner.temp }}/vize-release-integration-final/");
  assert.equal(custody.with?.["if-no-files-found"], "error");
  assert.deepEqual(check.permissions, { contents: "read" });
  assert.deepEqual(source.permissions, {
    contents: "read",
    "pull-requests": "read",
    actions: "read",
  });
  assert.equal(check.concurrency["cancel-in-progress"], true);
  assert.equal(
    check.concurrency.group,
    "check-v3-${{ github.workflow }}-${{ (github.event_name == 'workflow_dispatch' || github.event_name == 'schedule') && format('full-{0}', github.sha) || (github.event.pull_request.number || github.ref) }}",
  );
});

test("failed, skipped and absent nested qualification results poison the existing required report", () => {
  const outer = (sourceResult: string) =>
    aggregateNeedsResults({
      ...Object.fromEntries(topPrior.map((job) => [job, { result: "success" }])),
      "pr-source-checks": { result: sourceResult },
    });
  for (const result of ["failure", "cancelled", "skipped"]) {
    const inner = aggregateCheckNeedsResults(needs(result), "merge_group");
    assert.equal(inner.exitCode, 1);
    assert.equal(outer(inner.exitCode === 0 ? "success" : "failure").exitCode, 1);
  }
  for (const rows of [{}, { ...needs(), "nuxt-module-qualification": {} }]) {
    assert.throws(() => aggregateCheckNeedsResults(rows, "merge_group"));
    assert.equal(outer("failure").exitCode, 1);
  }
  assert.equal(outer("skipped").exitCode, 1);
  for (const [event, wit] of [
    ["pull_request", "skipped"],
    ["merge_group", "success"],
  ]) {
    const inner = aggregateCheckNeedsResults(needs(wit), event);
    assert.equal(inner.exitCode, 0);
    assert.equal(outer(inner.exitCode === 0 ? "success" : "failure").exitCode, 0);
  }
});

test("only the queue-only row may skip in the explicit supported nonqueue modes", () => {
  for (const event of ["pull_request", "push", "schedule", "workflow_dispatch"]) {
    assert.equal(aggregateCheckNeedsResults(needs("skipped"), event).exitCode, 0);
    for (const job of prior) {
      for (const result of ["skipped", "failure", "cancelled"]) {
        const rows = { ...needs("skipped"), [job]: { result } };
        assert.equal(aggregateCheckNeedsResults(rows, event).exitCode, 1);
      }
    }
    for (const result of ["failure", "cancelled"]) {
      assert.equal(aggregateCheckNeedsResults(needs(result), event).exitCode, 1);
    }
  }
  assert.equal(aggregateNeedsResults(needs("skipped")).exitCode, 1);
});

test("queue refusals, absent results and unknown events fail closed", () => {
  assert.equal(aggregateCheckNeedsResults(needs(), "merge_group").exitCode, 0);
  for (const result of ["skipped", "failure", "cancelled"]) {
    assert.equal(aggregateCheckNeedsResults(needs(result), "merge_group").exitCode, 1);
  }
  for (const event of ["merge_group", "pull_request", "push", "schedule", "workflow_dispatch"]) {
    assert.throws(() => aggregateCheckNeedsResults({}, event), /missing nuxt-module-qualification/);
    assert.throws(
      () => aggregateCheckNeedsResults({ ...needs(), "nuxt-module-qualification": {} }, event),
      /nuxt-module-qualification reported no result/,
    );
  }
  for (const event of [undefined, "", "workflow_call", "pull_request_target", "unknown"]) {
    assert.throws(
      () => aggregateCheckNeedsResults(needs("skipped"), event),
      /Unsupported Check event/,
    );
  }
});

test("the actual Check CLI fails queue omissions while the shared strict mode stays exact", () => {
  const script = "tools/support/compat/github/require-needs-success.ts";
  const run = (args: string[], event: string, rows: Record<string, { result: string }>) =>
    spawnSync(process.execPath, [script, ...args], {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, GITHUB_EVENT_NAME: event, NEEDS_JSON: JSON.stringify(rows) },
    });
  assert.equal(run(["--check"], "pull_request", needs("skipped")).status, 0);
  assert.equal(run(["--check"], "workflow_dispatch", needs("skipped")).status, 0);
  assert.equal(run(["--check"], "merge_group", needs()).status, 0);
  assert.equal(run(["--check"], "merge_group", needs("skipped")).status, 1);
  assert.equal(run(["--check"], "unknown", needs("skipped")).status, 1);
  assert.equal(run([], "pull_request", { old: { result: "success" } }).status, 0);
  assert.equal(run([], "pull_request", { old: { result: "skipped" } }).status, 1);
  assert.equal(run(["--unknown"], "pull_request", needs()).status, 1);
});

test("retired experimental dialect runtimes and bootstrap have no active filesystem surface", () => {
  for (const relative of [
    "davinci/vize_extension_contract",
    "davinci/vize_extension_host",
    "davinci/vize_guest",
    "davinci/vize_dialect_moonbit",
    "davinci/vize_l0/src/extension",
    "davinci/vize_l1/src/lang.rs",
    "davinci/vize_l2/src/lang/moonbit",
    "npm/extension-sdk",
    "examples/volt-target",
    ".github/workflows/davinci-contracts.yml",
    ".github/workflows/level-moonbit.yml",
    ".github/actions/setup-moonbit",
    ".moonbit-version",
    "tools/nix/moonbit.nix",
  ]) {
    assert.equal(fs.existsSync(path.join(root, relative)), false, relative);
  }
});
