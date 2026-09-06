import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

import { testAndBenchmarkTasks } from "../../tools/config/vite-plus/tasks/test-benchmark.ts";
import {
  budgetRegistryPath,
  loadLspIncrementalBudget,
} from "../performance/support/incremental-metrics.ts";
import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

const scorecardPath = "tests/_fixtures/maestro-vue-language-tools-scorecard.json";
const scorecardDocPath = "docs/release/maestro-vue-language-tools-scorecard.md";

type Evidence = {
  file: string;
  contains: string[];
};

type Oracle = {
  id: string;
  summary: string;
  evidence: Evidence[];
};

type FeatureRow = {
  dimension: string;
  lspMethods: string[];
  mustInclude: Oracle[];
  mustExclude: Oracle[];
};

type EditorRow = {
  editor: string;
  coverage: string;
  ciJob: string;
  task: string;
  mustInclude: string[];
  mustExclude: string[];
  evidence: Evidence[];
};

type LatencyBudgetRow = {
  fixtureId: string;
  fixtureName: string;
  suite: string;
  budgetSource: string;
  ciJob: string;
  ciStep: string;
  completionLane: string;
  hoverLane: string;
  diagnosticsToStableLanes: string[];
};

type Scorecard = {
  schemaVersion: number;
  trackingIssue: number;
  baseline: {
    name: string;
    server: string;
    versionEvidence: Evidence[];
  };
  featureMatrix: FeatureRow[];
  editorBreadth: EditorRow[];
  latencyBudgets: LatencyBudgetRow[];
};

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readScorecard(): Scorecard {
  return JSON.parse(fs.readFileSync(path.join(repoRoot, scorecardPath), "utf8")) as Scorecard;
}

function taskCommand(name: string): string {
  const entry = testAndBenchmarkTasks[name] as { command?: string } | undefined;
  assert.ok(entry?.command, `missing task ${name}`);
  return entry.command;
}

// Evidence anchors name behavior (test titles, identifiers, diagnostic payloads),
// so whitespace is incidental: collapsing runs of whitespace on both sides keeps
// the release gate passing across reformats and line rewraps.
function collapseWhitespace(value: string): string {
  return value.replace(/\s+/g, " ");
}

function assertEvidence(evidence: Evidence[]): void {
  assert.ok(evidence.length > 0, "each oracle must point at executable evidence");
  for (const item of evidence) {
    const absolute = path.join(repoRoot, item.file);
    assert.ok(fs.existsSync(absolute), `missing evidence file ${item.file}`);
    const content = collapseWhitespace(fs.readFileSync(absolute, "utf8"));
    for (const required of item.contains) {
      assert.ok(
        content.includes(collapseWhitespace(required)),
        `${item.file} must contain ${JSON.stringify(required)}`,
      );
    }
  }
}

test("Maestro scorecard fixture covers every Vue Language Server parity dimension", () => {
  const scorecard = readScorecard();
  assert.equal(scorecard.schemaVersion, 1);
  assert.equal(scorecard.trackingIssue, 3224);
  assert.equal(scorecard.baseline.name, "vuejs/language-tools");
  assert.equal(scorecard.baseline.server, "Vue Language Server");
  assertEvidence(scorecard.baseline.versionEvidence);

  const requiredDimensions = [
    "diagnostics",
    "completion",
    "signature-help",
    "hover",
    "definition",
    "references",
    "rename",
    "code-actions",
    "semantic-tokens",
    "inlay-hints",
    "document-features",
    "file-rename",
    "workspace-symbols",
  ];
  assert.deepEqual(
    scorecard.featureMatrix.map((row) => row.dimension),
    requiredDimensions,
  );

  for (const row of scorecard.featureMatrix) {
    assert.ok(row.lspMethods.length > 0, `${row.dimension} must name LSP methods`);
    assert.ok(row.mustInclude.length > 0, `${row.dimension} needs positive oracles`);
    assert.ok(row.mustExclude.length > 0, `${row.dimension} needs negative oracles`);
    for (const oracle of [...row.mustInclude, ...row.mustExclude]) {
      assert.match(oracle.id, /^[a-z0-9-]+$/);
      assert.ok(oracle.summary.length > 20, `${row.dimension}.${oracle.id} is too vague`);
      assertEvidence(oracle.evidence);
    }
  }
});

test("Maestro scorecard gates editor breadth through CI-backed artifacts", () => {
  const scorecard = readScorecard();
  const expectedEditors = ["VS Code", "Zed", "Neovim", "Helix", "Vim", "Emacs"];
  assert.deepEqual(
    scorecard.editorBreadth.map((row) => row.editor),
    expectedEditors,
  );

  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const hostAction = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");
  const hostJob = workflowJobBody(workflow, "editor-host-smoke");
  assert.match(hostJob, /uses: \.\/\.github\/actions\/vscode-host-smoke/);
  const jobs = (parse(workflow) as { jobs: Record<string, { needs?: string[] | string }> }).jobs;
  const reportNeeds = jobs["test-report"]?.needs ?? [];
  assert.ok(
    (Array.isArray(reportNeeds) ? reportNeeds : [reportNeeds]).includes("editor-host-smoke"),
    "test-report must aggregate the editor-host-smoke gate",
  );

  for (const row of scorecard.editorBreadth) {
    assert.equal(row.ciJob, "editor-host-smoke");
    assert.ok(row.mustInclude.length > 0, `${row.editor} must state covered behavior`);
    assert.ok(row.mustExclude.length > 0, `${row.editor} must state forbidden overclaim/leakage`);
    assertEvidence(row.evidence);
    assert.ok(hostAction.includes(row.task), `${row.editor} task is not wired in host CI`);
    assert.ok(taskCommand(row.task).length > 0, `${row.editor} task command must be registered`);
  }

  assert.equal(
    scorecard.editorBreadth.filter((row) => row.coverage.includes("real-server")).length,
    5,
    "five editor integrations have real-server evidence; Emacs is explicitly packaged Eglot evidence",
  );
  assert.equal(
    scorecard.editorBreadth.find((row) => row.editor === "Emacs")?.coverage,
    "packaged-eglot-spec",
  );
});

test("Maestro scorecard names enforced Misskey and Vue Vben Admin latency budgets", () => {
  const scorecard = readScorecard();
  assert.deepEqual(
    scorecard.latencyBudgets.map((row) => row.fixtureId),
    ["misskey", "vue-vben-admin"],
  );

  const workflow = readRepoFile(".github", "actions", "check-vue-parity", "action.yml");
  for (const row of scorecard.latencyBudgets) {
    assert.equal(row.budgetSource, budgetRegistryPath);
    assert.equal(row.ciJob, "vue-parity");
    assert.ok(workflow.includes(row.ciStep), `${row.suite} must be run by vue-parity CI`);
    assert.ok(workflow.includes("test:performance:lsp-incremental"));

    const { fixtureId, budget } = loadLspIncrementalBudget(row.suite);
    assert.equal(fixtureId, row.fixtureId);
    for (const lane of [row.completionLane, row.hoverLane, ...row.diagnosticsToStableLanes]) {
      const budgetMs = budget.laneBudgetsMs[lane];
      assert.ok(
        Number.isSafeInteger(budgetMs) && budgetMs > 0,
        `${row.suite}.${lane} must have a positive enforced latency budget`,
      );
      assert.ok(
        budgetMs <= budget.laneHardTimeoutMs,
        `${row.suite}.${lane} must fit under the hard timeout`,
      );
    }
  }
});

test("Maestro scorecard documentation mirrors fixture dimensions and evidence lanes", () => {
  const scorecard = readScorecard();
  const doc = collapseWhitespace(
    readRepoFile("docs", "release", "maestro-vue-language-tools-scorecard.md"),
  );

  assert.ok(doc.includes(scorecardPath), "scorecard docs must name the fixture source of truth");
  assert.ok(
    doc.includes("tests/tooling/lsp-vue-language-tools-scorecard.test.ts"),
    "scorecard docs must name the executable metadata guard",
  );

  for (const row of scorecard.featureMatrix) {
    const label = row.dimension.replaceAll("-", " ");
    assert.ok(doc.includes(label), `${scorecardDocPath} is missing dimension ${row.dimension}`);
  }

  for (const row of scorecard.editorBreadth) {
    assert.ok(doc.includes(row.editor), `${scorecardDocPath} is missing editor ${row.editor}`);
    assert.ok(doc.includes(row.task), `${scorecardDocPath} is missing task ${row.task}`);
  }

  for (const row of scorecard.latencyBudgets) {
    assert.ok(
      doc.includes(row.fixtureName),
      `${scorecardDocPath} is missing fixture ${row.fixtureName}`,
    );
    assert.ok(doc.includes(row.suite), `${scorecardDocPath} is missing suite ${row.suite}`);
  }
});
