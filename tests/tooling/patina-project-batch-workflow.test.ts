import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

void test("type-aware batch qualification stays optional and reuses exact optimized binaries", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "benchmark.yml"));
  assert.deepEqual(workflow.on.workflow_dispatch.inputs.patina_project_batch, {
    description: "Measure the frozen 1210-file type-aware project batch and exact RPC counts",
    required: false,
    default: false,
    type: "boolean",
  });
  const steps = workflow.jobs["pr-benchmark"].steps;
  const measure = steps.find(
    (step: { name?: string }) =>
      step.name === "Qualify type-aware project batching on existing optimized binaries",
  );
  assert.equal(
    measure.if,
    "github.event_name == 'workflow_dispatch' && inputs.patina_project_batch && steps.base-integrity.outputs.skip != 'true' && steps.validate-base.outcome == 'success' && steps.build-base.outcome != 'failure'",
  );
  assert.equal(
    measure.run,
    [
      "node --test head/tools/benchmarks/scripts/patina-project-batch.test.mjs",
      "node head/tools/benchmarks/scripts/patina-project-batch.mjs \\",
      '  "$GITHUB_WORKSPACE/base/target/ci-opt/vize" \\',
      '  "$GITHUB_WORKSPACE/head/target/ci-opt/vize" \\',
      '  "$BENCHMARK_BASE_SHA" "$BENCHMARK_HEAD_SHA" \\',
      '  "$GITHUB_WORKSPACE/patina-project-batch"',
      "",
    ].join("\n"),
  );
  const retain = steps.find(
    (step: { name?: string }) =>
      step.name === "Retain type-aware batch evidence including failed observations",
  );
  assert.equal(
    retain.if,
    "always() && github.event_name == 'workflow_dispatch' && inputs.patina_project_batch",
  );
  assert.equal(retain.with["if-no-files-found"], "error");
  assert.deepEqual(retain.with.path.trim().split("\n"), [
    "patina-project-batch/*.json",
    "patina-project-batch/*.jsonl",
    "patina-project-batch/*.stdout",
    "patina-project-batch/*.stderr",
    "patina-project-batch/*.rss.txt",
    "patina-project-batch/summary.md",
    "patina-project-batch/dependency-install.log",
  ]);
});

void test("benchmark conflict guard rejects real markers without mistaking long license headings", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "benchmark.yml"));
  const source = workflow.jobs["pr-benchmark"].steps.find(
    (step: { name?: string }) => step.name === "Check base checkout",
  ).run;
  const expression = source.match(/git grep -n -E '([^']+)'/u)?.[1];
  assert.equal(expression, "^(<<<<<<<|=======$|>>>>>>>)");
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "benchmark-conflict-guard-"));
  try {
    for (const [text, expected] of [
      ["Mozilla Public License Version 2.0\n=================================\n", 1],
      ["License\n==================================\n", 1],
      ["<<<<<<< HEAD\nbefore\n=======\nafter\n>>>>>>> topic\n", 0],
      ["<<<<<<< HEAD\r\nbefore\r\n=======\r\nafter\r\n>>>>>>> topic\r\n", 0],
      ["<<<<<<< HEAD\nunclosed\n", 0],
      ["=======\n", 0],
      ["unopened\n>>>>>>> topic\n", 0],
      [`${"<".repeat(32)} HEAD\nbefore\n${"=".repeat(32)}\nafter\n${">".repeat(32)} topic\n`, 0],
    ] as const) {
      fs.writeFileSync(path.join(root, "input"), text);
      const result = spawnSync(
        "git",
        ["grep", "--no-index", "-n", "-E", expression, "--", "input"],
        { cwd: root, encoding: "utf8" },
      );
      assert.equal(result.error, undefined);
      assert.equal(result.status, expected, `${JSON.stringify(text)}: ${result.stderr}`);
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
