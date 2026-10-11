import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, string>;
  env?: Record<string, string>;
  "continue-on-error"?: boolean;
};
type Workflow = {
  jobs: Record<string, { steps?: Step[]; needs?: string | string[]; uses?: string }>;
};
const callers = [
  { file: "pr-source-checks.yml", job: "pr-tooling-scripts" },
  { file: "check.yml", job: "test-scripts" },
];

for (const { file, job } of callers) {
  test(`${file} ${job} requires a genuine current-source CLI receipt before full tooling tests`, () => {
    const workflow = parse(readRepoFile(".github", "workflows", file)) as Workflow;
    const steps = workflow.jobs[job].steps ?? [];
    const shared = file === "pr-source-checks.yml";
    const build = steps.findIndex((step) =>
      shared
        ? step.uses === "./.github/actions/restore-source-cli"
        : /cargo build --profile ci -p vize/.test(step.run ?? ""),
    );
    const producer = shared
      ? ((parse(readRepoFile(".github", "workflows", "pr-source-cli.yml")) as Workflow).jobs.cli
          .steps ?? [])
      : steps;
    const originalBuild = producer.find((step) =>
      /cargo build --profile ci -p vize/.test(step.run ?? ""),
    );
    assert.ok(originalBuild);
    const tests = steps.findIndex((step) =>
      /(?:^| )vp run --workspace-root test:scripts(?::planned)?$/.test(step.run ?? ""),
    );
    assert.ok(build >= 0 && tests > build);
    assert.match(
      originalBuild.run ?? "",
      /cargo build --profile ci -p vize && vp exec node tests\/differential\/build-receipt\.ts/,
    );
    assert.match(
      originalBuild.run ?? "",
      /build-receipt\.ts && cat target\/ci\/vize\.differential-build\.json/,
    );
    assert.notEqual(steps[build]["continue-on-error"], true);
    assert.notEqual(originalBuild["continue-on-error"], true);
    assert.notEqual(steps[tests]["continue-on-error"], true);
    if (file === "check.yml") {
      assert.equal(steps[tests].run, "vp run --workspace-root test:scripts");
      for (const step of steps.slice(0, tests)) {
        assert.equal(step.env?.VIZE_LSP_BIN, undefined);
        assert.equal(step.env?.VIZE_LSP_REQUIRE_SOURCE_BUILD, undefined);
      }
    }
    assert.equal(steps[tests].env?.VIZE_LSP_BIN, "${{ github.workspace }}/target/ci/vize");
    assert.equal(steps[tests].env?.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
    if (file === "pr-source-checks.yml") {
      assert.equal(workflow.jobs["pr-source-cli"].uses, "./.github/workflows/pr-source-cli.yml");
      assert.deepEqual(workflow.jobs[job].needs, ["pr-source-plan", "pr-source-cli"]);
      assert.equal(steps[build].with?.sha256, "${{ needs.pr-source-cli.outputs.sha256 }}");
      const restore = parse(
        readRepoFile(".github", "actions", "restore-source-cli", "action.yml"),
      ) as { runs: { steps: Step[] } };
      const command = restore.runs.steps.at(-1)?.run ?? "";
      assert.match(command, /cat target\/ci\/vize\.differential-build\.json/);
      assert.doesNotMatch(command, /build-receipt\.ts/);
      const selected = steps.findIndex((step) =>
        /vp run --workspace-root test:scripts:pr/.test(step.run ?? ""),
      );
      assert.ok(selected > build);
      assert.notEqual(steps[selected]["continue-on-error"], true);
    }
  });
}
