import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import vm from "node:vm";
import { parse } from "yaml";

import { repoRoot } from "../_helpers/moonbit.ts";
import {
  realProjectArtifacts,
  shardEntries,
  typecheckRegistry,
} from "../_helpers/release-preflight-matrix-evidence-fixture.ts";
import { createReleasePreflightVerifyOnlyFixture } from "../support/release-preflight-runner-fixture.ts";
import { releaseSha, successfulReleaseRun } from "../support/release-preflight.ts";
import {
  createReleaseGateDispatchPlans,
  releaseGateRunQualifiers,
} from "../../../tools/support/compat/github/release-preflight-bootstrap.mjs";
import {
  requiredReleaseWorkflows,
  selectRequiredWorkflowRuns,
} from "../../../tools/support/compat/github/release-preflight-evidence.mjs";
import { assertRealProjectMatrixReleaseArtifacts } from "../../../tools/support/compat/github/release-preflight-matrix-evidence.mjs";

type SurfaceReport = {
  version: number;
  sourceCommit: string;
  shardIndex: string;
  status: string;
  failedSurfaceNames: string[];
  surfaces: Array<{ name: string; outcome: string; rawOutcome: string; budgetMode: string }>;
};
const byCodeUnit = (left: string, right: string) => (left < right ? -1 : left > right ? 1 : 0);

const modeOutcomes = {
  core_tools_mode: ["CORE_TOOLS_MODE", "VIZE_CORE_TOOLS_OUTCOME"],
  typecheck_dependencies_mode: [
    "TYPECHECK_DEPENDENCIES_MODE",
    "VIZE_TYPECHECK_DEPENDENCIES_OUTCOME",
  ],
  lint_divergence_mode: ["LINT_DIVERGENCE_MODE", "VIZE_LINT_DIVERGENCE_OUTCOME"],
  lsp_mode: ["LSP_MODE", "VIZE_LSP_OUTCOME"],
  typecheck_divergence_mode: ["TYPECHECK_DIVERGENCE_MODE", "VIZE_TYPECHECK_DIVERGENCE_OUTCOME"],
} as const;
const modeInputs = [...Object.keys(modeOutcomes), "davinci_dom_corpus_mode"];
const workflow = parse(
  fs.readFileSync(path.join(repoRoot, ".github/workflows/real-project-matrix.yml"), "utf8"),
);
const plans = createReleaseGateDispatchPlans({
  ref: "release/v0.446.0",
  headSha: releaseSha,
  baseSha: "b".repeat(40),
});
const matrixPlan = plans.find(
  (plan: { workflowName: string }) => plan.workflowName === "Real Project Matrix",
);
assert.ok(matrixPlan);

function primaryTitle(inputs: Record<string, string>) {
  return workflow["run-name"].replace(
    /\$\{\{([\s\S]*?)\}\}/g,
    (_match: string, expression: string) =>
      vm.runInNewContext(expression, { inputs, github: { sha: releaseSha } }, { timeout: 100 }),
  );
}

test("primary Matrix enum, actual run-name expression and strict selector bind all six modes", () => {
  assert.deepEqual(Object.keys(matrixPlan.inputs).sort(byCodeUnit), modeInputs.sort(byCodeUnit));
  for (const name of modeInputs) {
    const input = workflow.on.workflow_dispatch.inputs[name];
    assert.equal(input.default, "enforce");
    assert.ok(input.options.includes(matrixPlan.inputs[name]));
    assert.equal(matrixPlan.inputs[name], "enforce");
    for (const mode of input.options.filter((value: string) => value !== "enforce")) {
      const title = primaryTitle({ ...matrixPlan.inputs, [name]: mode });
      assert.equal(title, `Real Project Matrix observational @ ${releaseSha}`, `${name}=${mode}`);
      const runs = plans.map(
        (plan: { workflowName: string; expectedRunName: string }, index: number) => ({
          ...successfulReleaseRun(plan.workflowName, index + 1),
          display_title: plan.workflowName === "Real Project Matrix" ? title : plan.expectedRunName,
          head_branch: "release/v0.446.0",
          event: "workflow_dispatch",
        }),
      );
      assert.throws(
        () =>
          selectRequiredWorkflowRuns(
            runs,
            releaseSha,
            requiredReleaseWorkflows,
            releaseGateRunQualifiers(plans),
          ),
        /Real Project Matrix: missing/,
      );
    }
  }
  assert.equal(primaryTitle(matrixPlan.inputs), matrixPlan.expectedRunName);
  assert.equal(
    primaryTitle({}),
    matrixPlan.expectedRunName,
    "default/scheduled modes are enforced",
  );
  assert.deepEqual(
    plans.map((plan: { workflowName: string }) => plan.workflowName).sort(byCodeUnit),
    [...requiredReleaseWorkflows].sort(byCodeUnit),
  );
});

function actualSurfaceReport(overrides: NodeJS.ProcessEnv = {}) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-enforced-surface-"));
  try {
    const output = path.join(directory, "surface-verdict.json");
    const environment: NodeJS.ProcessEnv = {
      GITHUB_SHA: releaseSha,
      FIXTURE_SHARD_INDEX: "0",
      VIZE_WAIVER_AUDIT_OUTCOME: "success",
      VIZE_SYNTAX_HIGHLIGHTER_OUTCOME: "success",
      VIZE_GLYPH_OUTCOME: "success",
    };
    for (const [mode, outcome] of Object.values(modeOutcomes)) {
      environment[mode] = "enforce";
      environment[outcome] = "success";
    }
    const result = spawnSync(
      "rust-script",
      [
        "tools/commands/fixtures/real-project-surface-verdict.rs",
        "--from-workflow-env",
        "--output",
        output,
      ],
      {
        cwd: repoRoot,
        encoding: "utf8",
        env: { ...process.env, ...environment, ...overrides },
      },
    );
    assert.ifError(result.error);
    assert.equal(result.status, 0, result.stderr);
    return JSON.parse(fs.readFileSync(output, "utf8")) as SurfaceReport;
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

function actualPreflight(mutate?: (entries: Record<string, string>) => void) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-enforced-matrix-"));
  try {
    const fixture = createReleasePreflightVerifyOnlyFixture(directory, {
      mutateShardEntries(shard, entries) {
        if (shard === 0) mutate?.(entries);
      },
    });
    const result = spawnSync(
      "rust-script",
      ["tools/commands/ci/github/release-preflight.rs", "--verify-only"],
      {
        cwd: repoRoot,
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${fixture.binDir}${path.delimiter}${process.env.PATH ?? ""}`,
          ...fixture.env,
        },
      },
    );
    assert.ifError(result.error);
    return result;
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

test("actual Rust producer and full 22-archive runtime accept genuine complete enforcement", () => {
  const report = actualSurfaceReport();
  assert.equal(report.version, 2);
  assert.ok(
    report.surfaces.every(
      (surface: { budgetMode: string; rawOutcome: string }) =>
        surface.budgetMode === "enforce" && surface.rawOutcome === "success",
    ),
  );
  const result = actualPreflight((entries) => {
    entries["surface-verdict.json"] = JSON.stringify(report);
  });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /Release preflight passed/);
});

test("actual Rust release runtime refuses record-only and skipped producer successes", () => {
  for (const [mode, outcome] of Object.values(modeOutcomes)) {
    for (const rawOutcome of ["success", "failure"]) {
      const report = actualSurfaceReport({ [mode]: "record-only", [outcome]: rawOutcome });
      assert.equal(
        report.status,
        "success",
        "observational policy stays distinct from release eligibility",
      );
      assert.ok(
        report.surfaces.some(
          (surface: { budgetMode: string; rawOutcome: string }) =>
            surface.budgetMode === "record-only" && surface.rawOutcome === rawOutcome,
        ),
      );
      const result = actualPreflight((entries) => {
        entries["surface-verdict.json"] = JSON.stringify(report);
      });
      assert.equal(result.status, 1, `${mode}: ${result.stderr}`);
      assert.match(result.stderr, /complete enforced surface set/);
    }
  }
  const report = actualSurfaceReport({
    TYPECHECK_DIVERGENCE_MODE: "skip",
    VIZE_TYPECHECK_DIVERGENCE_OUTCOME: "skipped",
  });
  assert.equal(report.status, "success");
  const result = actualPreflight((entries) => {
    entries["surface-verdict.json"] = JSON.stringify(report);
  });
  assert.equal(result.status, 1, result.stderr);
  assert.match(result.stderr, /complete enforced surface set/);
});

test("compatibility archive validator also refuses stale, foreign and incomplete strict reports", async () => {
  const run = successfulReleaseRun("Real Project Matrix", 600);
  for (const mutate of [
    (report: SurfaceReport) => {
      report.version = 1;
    },
    (report: SurfaceReport) => {
      report.sourceCommit = "b".repeat(40);
    },
    (report: SurfaceReport) => {
      report.shardIndex = "9";
    },
    (report: SurfaceReport) => {
      report.surfaces.pop();
    },
    (report: SurfaceReport) => {
      report.surfaces[0] = report.surfaces[1];
    },
    (report: SurfaceReport) => {
      report.failedSurfaceNames = ["core-tools"];
    },
    (report: SurfaceReport) => {
      report.surfaces[1].budgetMode = "record-only";
    },
    (report: SurfaceReport) => {
      report.surfaces[7].rawOutcome = "skipped";
    },
  ]) {
    await assert.rejects(
      () =>
        assertRealProjectMatrixReleaseArtifacts({
          run,
          artifacts: realProjectArtifacts(run),
          registry: typecheckRegistry(),
          readArtifactEntries: async (artifact: { name: string }) => {
            const shard = Number(artifact.name.split("-").pop());
            const entries = shardEntries(shard);
            if (shard === 0) {
              const report = JSON.parse(entries["surface-verdict.json"]);
              mutate(report);
              entries["surface-verdict.json"] = JSON.stringify(report);
            }
            return entries;
          },
        }),
      /surface verdict/,
    );
  }
});

test("actual Rust release runtime refuses omitted typecheck coverage and retained nonzero divergences", () => {
  for (const mutate of [
    (entries: Record<string, string>) => {
      for (const name of Object.keys(entries))
        if (name.endsWith("-typecheck-divergence.json")) delete entries[name];
    },
    (entries: Record<string, string>) => {
      const report = JSON.parse(entries["lint-divergence-summary.json"]);
      report.budget.status = "failure";
      report.budget.passed = false;
      report.budget.failedCount = 1;
      entries["lint-divergence-summary.json"] = JSON.stringify(report);
    },
    (entries: Record<string, string>) => {
      const name = Object.keys(entries).find((name) => name.endsWith("-typecheck-divergence.json"));
      assert.ok(name);
      const report = JSON.parse(entries[name]);
      report.divergence.summary.falseNegativeCount = 1;
      entries[name] = JSON.stringify(report);
    },
  ]) {
    const result = actualPreflight(mutate);
    assert.equal(result.status, 1, result.stderr);
    assert.match(
      result.stderr,
      /typecheck dependency artifact count|zero unexplained false positives and false negatives|lint divergence budget/,
    );
    assert.doesNotMatch(result.stdout, /Release preflight passed/);
  }
});
