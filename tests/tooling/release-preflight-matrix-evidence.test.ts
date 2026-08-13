import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";

import {
  assertRealProjectMatrixReleaseArtifacts,
  requiredRealProjectMatrixShardCount,
} from "../../tools/github/release-preflight-matrix-evidence.mjs";
import { releaseSha, successfulReleaseRun } from "./support/release-preflight.ts";

test("release preflight validates every real-project shard artifact", async () => {
  const run = successfulReleaseRun("Real Project Matrix", 500);
  const artifacts = realProjectArtifacts(run);

  await assert.doesNotReject(() =>
    assertRealProjectMatrixReleaseArtifacts({
      run,
      artifacts,
      readArtifactEntries: async (artifact) => shardEntries(Number(artifact.name.split("-").pop())),
    }),
  );
});

test("release preflight rejects missing or foreign real-project shard artifacts", async () => {
  const run = successfulReleaseRun("Real Project Matrix", 501);
  await assert.rejects(
    assertRealProjectMatrixReleaseArtifacts({
      run,
      artifacts: realProjectArtifacts(run).filter(
        (artifact) => artifact.name !== "real-project-matrix-7",
      ),
      readArtifactEntries: async (artifact) => shardEntries(Number(artifact.name.split("-").pop())),
    }),
    /real-project-matrix-7 artifact; found 0/,
  );
  await assert.rejects(
    assertRealProjectMatrixReleaseArtifacts({
      run,
      artifacts: realProjectArtifacts(run).map((artifact) =>
        artifact.name === "real-project-matrix-0"
          ? { ...artifact, workflow_run: { ...artifact.workflow_run, head_sha: "b".repeat(40) } }
          : artifact,
      ),
      readArtifactEntries: async () => shardEntries(0),
    }),
    /not bound to run/,
  );
  await assert.rejects(
    assertRealProjectMatrixReleaseArtifacts({
      run,
      artifacts: realProjectArtifacts(run).map((artifact) =>
        artifact.name === "real-project-matrix-0"
          ? { ...artifact, workflow_run: undefined }
          : artifact,
      ),
      readArtifactEntries: async () => shardEntries(0),
    }),
    /not bound to run/,
  );
});

test("release preflight rejects record-only and non-zero typecheck parity artifacts", async () => {
  for (const [label, mutate, message] of [
    [
      "record-only",
      (entries: Record<string, string>) =>
        mutateDivergence(entries, (artifact) => (artifact.enforcement.budgetMode = "record-only")),
      /must not be record-only/,
    ],
    [
      "breached budget",
      (entries: Record<string, string>) =>
        mutateDivergence(entries, (artifact) => {
          artifact.budget.passed = false;
          artifact.budget.verdict = "breached";
        }),
      /budget is breached/,
    ],
    [
      "false positive",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.divergence.summary.falsePositiveCount = 1),
        ),
      /zero unexplained false positives and false negatives/,
    ],
    [
      "false negative",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.divergence.summary.falseNegativeCount = 1),
        ),
      /zero unexplained false positives and false negatives/,
    ],
  ] as const) {
    const run = successfulReleaseRun("Real Project Matrix", 510);
    await assert.rejects(
      assertRealProjectMatrixReleaseArtifacts({
        run,
        artifacts: realProjectArtifacts(run),
        readArtifactEntries: async (artifact) => {
          const entries = shardEntries(Number(artifact.name.split("-").pop()));
          if (artifact.name === "real-project-matrix-0") mutate(entries);
          return entries;
        },
      }),
      message,
      label,
    );
  }
});

test("release preflight requires same-corpus coverage, mutation oracle, and preparation linkage", async () => {
  for (const [label, mutate, message] of [
    [
      "coverage mismatch",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.baseline.coverage.baselineVueFilesSha256 = "0".repeat(64)),
        ),
      /same non-empty authored Vue corpus/,
    ],
    [
      "empty shared coverage",
      (entries: Record<string, string>) =>
        mutateDivergence(entries, (artifact) => {
          artifact.baseline.coverage.sharedVueFileCount = 0;
          artifact.baseline.coverage.vizeVueFileCount = 0;
          artifact.baseline.coverage.baselineVueFileCount = 0;
        }),
      /same non-empty authored Vue corpus/,
    ],
    [
      "missing mutation",
      (entries: Record<string, string>) =>
        mutateDivergence(entries, (artifact) => delete artifact.mutationOracle),
      /seeded mutation oracle/,
    ],
    [
      "forged broken mutation state",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.mutationOracle.states[1].sharedCount = 0),
        ),
      /seeded mutation oracle/,
    ],
    [
      "unrestored repaired mutation state",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.mutationOracle.states[2].sourceSha256 = "7".repeat(64)),
        ),
      /seeded mutation oracle/,
    ],
    [
      "failed surface verdict",
      (entries: Record<string, string>) => {
        entries["surface-verdict.json"] = json({ status: "failure" });
      },
      /surface verdict is failure/,
    ],
    [
      "wrong shard count",
      (entries: Record<string, string>) => {
        const summary = JSON.parse(entries["summary.json"]);
        summary.command.shardCount = requiredRealProjectMatrixShardCount - 1;
        entries["summary.json"] = json(summary);
      },
      /not exact release evidence/,
    ],
    [
      "missing dependency artifact",
      (entries: Record<string, string>) => delete entries["fixture-typecheck-dependencies.json"],
      /typecheck dependency artifact/,
    ],
    [
      "unlinked dependency artifact",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.preparation.payloadSha256 = "0".repeat(64)),
        ),
      /missing dependency preparation linkage/,
    ],
  ] as const) {
    const run = successfulReleaseRun("Real Project Matrix", 520);
    await assert.rejects(
      assertRealProjectMatrixReleaseArtifacts({
        run,
        artifacts: realProjectArtifacts(run),
        readArtifactEntries: async (artifact) => {
          const entries = shardEntries(Number(artifact.name.split("-").pop()));
          if (artifact.name === "real-project-matrix-0") mutate(entries);
          return entries;
        },
      }),
      message,
      label,
    );
  }
});

function realProjectArtifacts(run: ReturnType<typeof successfulReleaseRun>) {
  return Array.from({ length: requiredRealProjectMatrixShardCount }, (_, shard) => ({
    id: 1_000 + shard,
    name: `real-project-matrix-${shard}`,
    expired: false,
    archive_download_url: `https://example.test/artifacts/${shard}.zip`,
    workflow_run: {
      id: run.id,
      head_branch: run.head_branch,
      head_sha: run.head_sha,
    },
  }));
}

function shardEntries(shard: number): Record<string, string> {
  const dependency = {
    schema: "vize.fixtureTypecheckDependencyInstall",
    version: 2,
    project: "fixture",
    revision: "b".repeat(40),
    evidence: { commitSha: releaseSha },
    packageManager: { name: "pnpm", version: "10.0.0" },
    lockfile: { path: "pnpm-lock.yaml", sizeBytes: 18, sha256: "1".repeat(64) },
    install: {
      command: ["pnpm", "install", "--frozen-lockfile", "--ignore-scripts", "--prefer-offline"],
      durationMs: 1,
      exitCode: 0,
      stdoutSha256: "2".repeat(64),
      stderrSha256: "3".repeat(64),
    },
    baselinePrepare: null,
  };
  const dependencyText = json(dependency);
  return {
    "selected-fixtures.txt": "tests/_fixtures/_git/fixture\n",
    "summary.json": json({
      schema: "vize.fixtureToolMatrixReport",
      version: 3,
      evidence: { commitSha: releaseSha },
      command: { shardIndex: shard, shardCount: requiredRealProjectMatrixShardCount },
    }),
    "surface-verdict.json": json({ status: "success" }),
    "fixture-typecheck-dependencies.json": dependencyText,
    "fixture-typecheck-divergence.json": json({
      schema: "vize.fixtureTypecheckDivergenceRun",
      version: 4,
      project: "fixture",
      revision: "b".repeat(40),
      evidence: { commitSha: releaseSha },
      enforcement: { budgetMode: "enforce" },
      preparation: {
        schema: "vize.fixtureTypecheckPreparationEvidence",
        version: 1,
        payloadSha256: sha256(dependencyText),
      },
      baseline: {
        coverage: {
          verdict: "usable",
          vizeVueFileCount: 1,
          baselineVueFileCount: 1,
          sharedVueFileCount: 1,
          vizeVueFilesSha256: "4".repeat(64),
          baselineVueFilesSha256: "4".repeat(64),
          missingVueFiles: [],
          unexpectedVueFiles: [],
        },
      },
      mutationOracle: {
        schema: "vize.fixtureTypecheckSeededMutationOracle",
        version: 1,
        verdict: "passed",
        passed: true,
        file: "src/App.vue",
        span: { line: 3, column: 1 },
        states: [
          mutationState("clean", "5".repeat(64), 0, 0, 0),
          mutationState("broken", "6".repeat(64), 1, 0, 0),
          mutationState("repaired", "5".repeat(64), 0, 0, 0),
        ],
      },
      budget: { passed: true, verdict: "passed" },
      divergence: {
        summary: {
          falsePositiveCount: 0,
          falseNegativeCount: 0,
        },
      },
    }),
  };
}

function mutationState(
  name: string,
  sourceSha256: string,
  sharedCount: number,
  falsePositiveCount: number,
  falseNegativeCount: number,
) {
  return {
    name,
    sourceSha256,
    vizeDiagnosticCount: sharedCount,
    baselineDiagnosticCount: sharedCount,
    sharedCount,
    falsePositiveCount,
    falseNegativeCount,
  };
}

function mutateDivergence(entries: Record<string, string>, mutate: (artifact: any) => void) {
  const artifact = JSON.parse(entries["fixture-typecheck-divergence.json"]);
  mutate(artifact);
  entries["fixture-typecheck-divergence.json"] = json(artifact);
}

function json(value: unknown) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function sha256(value: string) {
  return createHash("sha256").update(value).digest("hex");
}
