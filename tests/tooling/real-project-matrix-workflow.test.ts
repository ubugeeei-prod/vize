import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

type WorkflowStep = {
  env?: Record<string, string>;
  if?: string;
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
};

type WorkflowJob = {
  env?: Record<string, string>;
  "runs-on"?: string;
  steps?: WorkflowStep[];
  strategy?: { "fail-fast"?: boolean; matrix?: { shard?: number[] } };
  "timeout-minutes"?: number;
};

test("real-project workflow schedules every balanced fixture shard", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "real-project-matrix.yml")) as {
    concurrency?: { "cancel-in-progress"?: boolean; group?: string };
    jobs?: Record<string, WorkflowJob>;
    on?: { schedule?: Array<{ cron?: string }>; workflow_dispatch?: unknown };
    permissions?: Record<string, string>;
  };
  const job = workflow.jobs?.["real-project-matrix"];

  assert.ok(job);
  assert.deepEqual(workflow.permissions, { contents: "read" });
  assert.equal(workflow.on?.schedule?.[0]?.cron, "37 5 * * 0");
  assert.ok("workflow_dispatch" in (workflow.on ?? {}));
  assert.deepEqual(workflow.concurrency, {
    group: "real-project-matrix-${{ github.ref }}",
    "cancel-in-progress": true,
  });
  assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(job["timeout-minutes"], 120);
  assert.equal(job.strategy?.["fail-fast"], false);
  assert.deepEqual(job.strategy?.matrix?.shard, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
  assert.deepEqual(job.env, {
    FIXTURE_SHARD_COUNT: "11",
    FIXTURE_SHARD_INDEX: "${{ matrix.shard }}",
    FIXTURE_REPORT_DIR: "real-project-results/shard-${{ matrix.shard }}",
  });
});

test("real-project workflow hydrates only its shard and runs every core tool", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "real-project-matrix.yml")) as {
    jobs?: Record<string, WorkflowJob>;
  };
  const steps = workflow.jobs?.["real-project-matrix"]?.steps ?? [];
  const nodeSetupIndex = steps.findIndex((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"));
  const shims = steps.find((step) => step.name === "Enable package manager shims");
  assert.ok(shims, "Missing 'Enable package manager shims' step");
  const shimsIndex = steps.indexOf(shims);
  const hydrationIndex = steps.findIndex(
    (step) => step.name === "Select and hydrate fixture shard",
  );
  const hydration = steps.find((step) => step.name === "Select and hydrate fixture shard");
  const dependency = steps.find(
    (step) => step.name === "Install pinned typecheck baseline dependencies",
  );
  assert.ok(dependency, "Missing 'Install pinned typecheck baseline dependencies' step");
  const dependencyIndex = steps.indexOf(dependency);
  const run = steps.find((step) => step.name === "Exercise real projects with every core tool");
  const runIndex = steps.indexOf(run!);
  const glyphProperties = steps.find(
    (step) => step.name === "Check glyph formatter corpus properties",
  );
  const glyphPropertiesIndex = steps.indexOf(glyphProperties!);
  const divergence = steps.find((step) => step.name === "Record typechecker baseline divergence");
  const divergenceIndex = steps.indexOf(divergence!);
  const summary = steps.find((step) => step.name === "Publish shard summary");
  const summaryIndex = steps.indexOf(summary!);
  const upload = steps.find((step) => step.name === "Upload shard report");

  assert.notEqual(nodeSetupIndex, -1);
  assert.notEqual(hydrationIndex, -1);
  assert.ok(
    nodeSetupIndex < shimsIndex && shimsIndex < hydrationIndex,
    "Node 24 and package manager shims must be active before loading matrix scripts",
  );
  assert.deepEqual(steps[nodeSetupIndex].with, {
    "node-version-file": ".node-version",
    cache: true,
    "run-install": false,
  });
  assert.equal(shims.run, "corepack enable");
  assert.match(hydration?.run ?? "", /--list-fixture-paths/);
  assert.match(hydration?.run ?? "", /--shard-index "\$FIXTURE_SHARD_INDEX"/);
  assert.match(hydration?.run ?? "", /--shard-count "\$FIXTURE_SHARD_COUNT"/);
  assert.match(hydration?.run ?? "", /mkdir -p "\$FIXTURE_REPORT_DIR"/);
  assert.match(hydration?.run ?? "", /selected-fixtures\.txt/);
  assert.match(hydration?.run ?? "", /git submodule update --init --depth 1/);
  assert.doesNotMatch(hydration?.run ?? "", /--recursive/);
  assert.match(hydration?.run ?? "", /"\$\{fixture_paths\[@\]\}"/);
  assert.ok(hydrationIndex < dependencyIndex && dependencyIndex < runIndex);
  assert.match(dependency.run ?? "", /tools\/fixtures\/typecheck-dependency-prepare\.mjs/);
  assert.match(dependency.run ?? "", /--output-dir "\$FIXTURE_REPORT_DIR"/);
  assert.match(dependency.run ?? "", /--shard-index "\$FIXTURE_SHARD_INDEX"/);
  assert.match(dependency.run ?? "", /--shard-count "\$FIXTURE_SHARD_COUNT"/);
  assert.match(dependency.run ?? "", /--timeout-ms 600000/);
  assert.match(run?.run ?? "", /tools\/fixtures\/tool-matrix-report\.mjs/);
  assert.match(run?.run ?? "", /--vize-bin target\/ci\/vize/);
  assert.match(run?.run ?? "", /--timeout-ms 600000/);
  assert.match(run?.run ?? "", /--output-dir "\$FIXTURE_REPORT_DIR"/);
  assert.ok(runIndex < glyphPropertiesIndex && glyphPropertiesIndex < divergenceIndex);
  assert.equal(glyphProperties?.env?.VIZE_TEST_BIN, "target/ci/vize");
  assert.match(glyphProperties?.run ?? "", /--test-concurrency=1/);
  for (const property of ["idempotence", "parse-preservation", "lint-agreement"]) {
    assert.match(
      glyphProperties?.run ?? "",
      new RegExp(`tests/tooling/glyph-corpus-${property}\\.test\\.ts`),
    );
  }
  assert.ok(runIndex < divergenceIndex && divergenceIndex < summaryIndex);
  assert.match(divergence?.run ?? "", /tools\/fixtures\/typecheck-divergence-report\.mjs/);
  assert.match(divergence?.run ?? "", /--report-dir "\$FIXTURE_REPORT_DIR"/);
  assert.match(divergence?.run ?? "", /--shard-index "\$FIXTURE_SHARD_INDEX"/);
  assert.match(divergence?.run ?? "", /--shard-count "\$FIXTURE_SHARD_COUNT"/);
  assert.match(divergence?.run ?? "", /--vue-tsc-bin tests\/node_modules\/\.bin\/vue-tsc/);
  assert.equal(summary?.if, "${{ always() }}");
  assert.match(summary?.run ?? "", /summary\.md/);
  assert.match(summary?.run ?? "", /\*-typecheck-divergence\.md/);
  assert.match(summary?.run ?? "", /divergence_reports\[@\]/);
  assert.equal(upload?.if, "${{ always() }}");
  assert.match(upload?.uses ?? "", /^actions\/upload-artifact@[0-9a-f]{40}$/);
  assert.deepEqual(upload?.with, {
    name: "real-project-matrix-${{ matrix.shard }}",
    path: "${{ env.FIXTURE_REPORT_DIR }}",
    "if-no-files-found": "error",
    "retention-days": 30,
  });
});
