import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("Tool Benchmark ignores only known non-runtime pull request paths", () => {
  const workflow = parse(
    fs.readFileSync(path.join(root, ".github", "workflows", "tool-benchmark.yml"), "utf8"),
  ) as Record<string, unknown>;
  const events = workflow.on as { pull_request?: { "paths-ignore"?: string[] } };
  const jobs = workflow.jobs as Record<
    string,
    {
      if?: string;
      needs?: string;
      outputs?: Record<string, string>;
      steps?: Array<Record<string, unknown>>;
    }
  >;
  const impact = jobs["tool-benchmark-impact"];
  assert.ok(impact, "missing tool-benchmark-impact job");
  const changeDetection = impact.steps?.find((step) => step.id === "changes");
  assert.ok(changeDetection, "missing changes step");
  assert.match(String(changeDetection.uses), /^dorny\/paths-filter@/);
  const filterInput = changeDetection.with as {
    filters: string;
    "predicate-quantifier"?: string;
  };
  const filters = parse(String(filterInput.filters)) as {
    runtime?: string[];
  };

  assert.equal(events.pull_request?.["paths-ignore"], undefined);
  assert.equal(filterInput["predicate-quantifier"], "every");
  const expectedRuntimePatterns = [
    "**",
    "!**/*.md",
    "!docs/**",
    "!.changeset/**",
    "!.github/ISSUE_TEMPLATE/**",
    "!.github/PULL_REQUEST_TEMPLATE.md",
    "!.github/actions/app-readiness/**",
    "!.github/workflows/check-bench.yml",
    "!.github/workflows/check.yml",
    "!.github/workflows/criterion-bench.yml",
    "!.github/workflows/build-docs.yml",
    "!.github/workflows/deploy-docs.yml",
    "!.github/workflows/e2e.yml",
    "!.github/workflows/fuzz.yml",
    "!.github/workflows/miri.yml",
    "!.github/workflows/native-smoke.yml",
    "!.github/workflows/pkg-pr-new.yml",
    "!.github/workflows/release*.yml",
    "!.github/workflows/title-policy.yml",
    "!tools/benchmarks/scripts/check-gate.mjs",
    "!tools/benchmarks/scripts/check-gate-env.mjs",
    "!tools/benchmarks/scripts/check-gate-plants.mjs",
    "!tools/benchmarks/scripts/check-gate-report.mjs",
    "!editors/**",
    "!tests/**",
    "!tools/commands/ci/github/**",
    "!tools/moon/cmd/github/**",
    "!tools/moon/cmd/publish*/**",
    "!tools/moon/cmd/release/**",
    "!tools/commands/ci/source-coverage.rs",
    "!tools/commands/ci/source-file-lengths.rs",
    "!tools/commands/ci/check-warning-budget.rs",
    "!tools/commands/release/npm/tag.rs",
  ];
  assert.deepEqual([...(filters.runtime ?? [])].sort(), expectedRuntimePatterns.toSorted());
  assert.match(impact.if ?? "", /github\.event_name == 'pull_request'/);
  assert.equal(impact.outputs?.runtime, "${{ steps.changes.outputs.runtime }}");
  assert.equal(jobs["tool-benchmark"].needs, "tool-benchmark-impact");
  const benchmarkIf = jobs["tool-benchmark"].if ?? "";
  assert.match(benchmarkIf, /always\(\).* !cancelled\(\)/);
  assert.match(benchmarkIf, /github\.event_name != 'pull_request'/);
  assert.match(benchmarkIf, /outputs\.runtime == 'true'/);
});
