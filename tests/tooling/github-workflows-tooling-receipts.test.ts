import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

type Step = { name?: string; run?: string; "continue-on-error"?: boolean };
type Workflow = { jobs: Record<string, { steps?: Step[] }> };
const callers = [
  { file: "pr-source-checks.yml", job: "pr-tooling-scripts" },
  { file: "check.yml", job: "test-scripts" },
];

for (const { file, job } of callers) {
  test(`${file} ${job} creates its own CLI receipt before full tooling tests`, () => {
    const workflow = parse(readRepoFile(".github", "workflows", file)) as Workflow;
    const steps = workflow.jobs[job].steps ?? [];
    const build = steps.findIndex((step) =>
      /cargo build --profile ci -p vize/.test(step.run ?? ""),
    );
    const tests = steps.findIndex((step) =>
      /vp run --workspace-root test:scripts/.test(step.run ?? ""),
    );
    assert.ok(build >= 0 && tests > build);
    assert.match(
      steps[build].run ?? "",
      /cargo build --profile ci -p vize && vp exec node tests\/differential\/build-receipt\.mjs/,
    );
    assert.notEqual(steps[build]["continue-on-error"], true);
    assert.notEqual(steps[tests]["continue-on-error"], true);
    if (file === "pr-source-checks.yml") {
      const selected = steps.findIndex((step) =>
        /vp run --workspace-root test:scripts:pr/.test(step.run ?? ""),
      );
      assert.ok(selected > build);
      assert.notEqual(steps[selected]["continue-on-error"], true);
    }
  });
}
