import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import vm from "node:vm";
import { parse } from "yaml";

import { repoRoot } from "../_helpers/tool-commands.ts";
import {
  realProjectArtifacts,
  shardEntries,
  typecheckRegistry,
} from "../_helpers/release-preflight-matrix-evidence-fixture.ts";
import { runVerifyOnlyPreflight as actualPreflight } from "../support/release-preflight-verify-only.ts";
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
// Tool surfaces and the Davinci corpus gate a release. Lint and LSP parity are
// recorded, and typecheck divergence is skipped, until per-project budgets gate
// the three parity surfaces again.
const releaseModes = {
  core_tools_mode: "enforce",
  typecheck_dependencies_mode: "enforce",
  lint_divergence_mode: "record-only",
  lsp_mode: "record-only",
  typecheck_divergence_mode: "skip",
  davinci_dom_corpus_mode: "enforce",
} as const;
const enforcedModes = ["core_tools_mode", "typecheck_dependencies_mode"] as const;
const recordedModes = ["lint_divergence_mode", "lsp_mode"] as const;
const modeInputs = Object.keys(releaseModes);
const workflow = parse(
  fs.readFileSync(path.join(repoRoot, ".github/workflows/real-project-matrix.yml"), "utf8"),
);
const plans = createReleaseGateDispatchPlans({
  ref: "release/v0.446.0",
  headSha: releaseSha,
  baseSha: "b".repeat(40),
});
assert.equal(
  plans.some((plan: { workflowName: string }) => plan.workflowName === "Real Project Matrix"),
  false,
);

function primaryTitle(inputs: Record<string, string>) {
  return workflow["run-name"].replace(
    /\$\{\{([\s\S]*?)\}\}/g,
    (_match: string, expression: string) =>
      vm.runInNewContext(expression, { inputs, github: { sha: releaseSha } }, { timeout: 100 }),
  );
}

function assertSelectorIgnoresMatrixTitle(title: string) {
  const runs = [
    ...plans.map((plan: { workflowName: string; expectedRunName: string }, index: number) => ({
      ...successfulReleaseRun(plan.workflowName, index + 1),
      display_title: plan.expectedRunName,
      head_branch: "release/v0.446.0",
      event: "workflow_dispatch",
    })),
    {
      ...successfulReleaseRun("Real Project Matrix", 99),
      display_title: title,
      head_branch: "release/v0.446.0",
      event: "workflow_dispatch",
    },
  ];
  assert.doesNotThrow(() =>
    selectRequiredWorkflowRuns(
      runs,
      releaseSha,
      requiredReleaseWorkflows,
      releaseGateRunQualifiers(plans),
    ),
  );
}

test("matrix run names stay descriptive and no longer gate the release", () => {
  assert.equal(primaryTitle(releaseModes), `Real Project Matrix release-gated @ ${releaseSha}`);
  for (const name of modeInputs) {
    const input = workflow.on.workflow_dispatch.inputs[name];
    assert.equal(input.default, "enforce");
    assert.ok(input.options.includes(releaseModes[name]));
    for (const mode of input.options.filter((value: string) => value !== releaseModes[name])) {
      const title = primaryTitle({ ...releaseModes, [name]: mode });
      assert.equal(title, `Real Project Matrix observational @ ${releaseSha}`, `${name}=${mode}`);
      assertSelectorIgnoresMatrixTitle(title);
    }
  }
  const enforcedTitle = `Real Project Matrix enforced @ ${releaseSha}`;
  assert.equal(primaryTitle({}), enforcedTitle, "default/scheduled modes are enforced");
  assert.equal(
    primaryTitle(Object.fromEntries(modeInputs.map((name) => [name, "enforce"]))),
    enforcedTitle,
  );
  assertSelectorIgnoresMatrixTitle(enforcedTitle);
  assert.deepEqual(
    plans.map((plan: { workflowName: string }) => plan.workflowName).sort(byCodeUnit),
    [...requiredReleaseWorkflows].sort(byCodeUnit),
  );
});

function actualSurfaceReport(overrides: NodeJS.ProcessEnv = {}) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-gated-surface-"));
  try {
    const output = path.join(directory, "surface-verdict.json");
    const environment: NodeJS.ProcessEnv = {
      GITHUB_SHA: releaseSha,
      FIXTURE_SHARD_INDEX: "0",
      VIZE_WAIVER_AUDIT_OUTCOME: "success",
      VIZE_SYNTAX_HIGHLIGHTER_OUTCOME: "success",
      VIZE_GLYPH_OUTCOME: "success",
    };
    for (const [name, [mode, outcome]] of Object.entries(modeOutcomes)) {
      environment[mode] = releaseModes[name as keyof typeof modeOutcomes];
      environment[outcome] = environment[mode] === "skip" ? "skipped" : "success";
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

function preflightWithReport(report: SurfaceReport) {
  return actualPreflight((entries) => {
    entries["surface-verdict.json"] = JSON.stringify(report);
  });
}

test("actual Rust producer and full 22-archive runtime accept the exact release-gated dispatch", () => {
  const report = actualSurfaceReport();
  assert.equal(report.version, 2);
  assert.deepEqual(
    report.surfaces.map((surface) => [surface.name, surface.budgetMode, surface.rawOutcome]),
    [
      ["waiver-audit", "enforce", "success"],
      ["typecheck-dependencies", "enforce", "success"],
      ["core-tools", "enforce", "success"],
      ["lsp", "record-only", "success"],
      ["lint-divergence", "record-only", "success"],
      ["syntax-highlighter", "enforce", "success"],
      ["glyph", "enforce", "success"],
      ["typecheck-divergence", "skip", "skipped"],
    ],
  );
  const result = preflightWithReport(report);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /Release preflight passed/);
});

test("recorded parity surfaces retain a raw failure without refusing the release", () => {
  for (const name of recordedModes) {
    const [, outcome] = modeOutcomes[name];
    const report = actualSurfaceReport({ [outcome]: "failure" });
    assert.equal(report.status, "success");
    assert.deepEqual(
      report.surfaces
        .filter((surface) => surface.rawOutcome === "failure")
        .map((surface) => [surface.budgetMode, surface.outcome]),
      [["record-only", "success"]],
      name,
    );
    const result = preflightWithReport(report);
    assert.equal(result.status, 0, `${name}: ${result.stderr}`);
    assert.match(result.stdout, /Release preflight passed/);
  }
});

test("release preflight does not refuse observational matrix tool surfaces", () => {
  for (const name of enforcedModes) {
    const [mode, outcome] = modeOutcomes[name];
    for (const rawOutcome of ["success", "failure"]) {
      const report = actualSurfaceReport({ [mode]: "record-only", [outcome]: rawOutcome });
      assert.equal(report.status, "success");
      const result = preflightWithReport(report);
      assert.equal(result.status, 0, `${name}: ${result.stderr}`);
      assert.match(result.stdout, /Release preflight passed/);
    }
  }
});

test("release preflight does not refuse matrix parity surfaces", () => {
  for (const name of recordedModes) {
    const [mode] = modeOutcomes[name];
    const result = preflightWithReport(actualSurfaceReport({ [mode]: "enforce" }));
    assert.equal(result.status, 0, `${name}: ${result.stderr}`);
    assert.match(result.stdout, /Release preflight passed/);
  }
  for (const mode of ["enforce", "record-only"]) {
    const result = preflightWithReport(
      actualSurfaceReport({
        TYPECHECK_DIVERGENCE_MODE: mode,
        VIZE_TYPECHECK_DIVERGENCE_OUTCOME: "success",
      }),
    );
    assert.equal(result.status, 0, `typecheck ${mode}: ${result.stderr}`);
    assert.match(result.stdout, /Release preflight passed/);
  }
});

test("compatibility archive validator also refuses stale, foreign and incomplete release-gated reports", async () => {
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
      report.surfaces[2].rawOutcome = "failure";
    },
    (report: SurfaceReport) => {
      report.surfaces[3].rawOutcome = "cancelled";
    },
    (report: SurfaceReport) => {
      report.surfaces[4].budgetMode = "enforce";
    },
    (report: SurfaceReport) => {
      report.surfaces[7].budgetMode = "record-only";
    },
    (report: SurfaceReport) => {
      report.surfaces[7].rawOutcome = "success";
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
