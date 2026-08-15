import assert from "node:assert/strict";
import { test } from "node:test";

import {
  assertRealProjectMatrixReleaseArtifacts,
  requireRealProjectMatrixRun,
} from "../../tools/github/release-preflight-matrix-evidence.mjs";
import {
  mutateDivergence,
  realProjectArtifacts,
  shardEntries,
} from "./_helpers/release-preflight-matrix-evidence-fixture.ts";
import { successfulReleaseRun } from "./support/release-preflight.ts";

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

test("release preflight requires exact lint divergence evidence", async () => {
  for (const [label, mutate, message] of [
    [
      "missing",
      (entries: Record<string, string>) => delete entries["lint-divergence-summary.json"],
      /lint-divergence-summary\.json/,
    ],
    [
      "foreign commit",
      (entries: Record<string, string>) => {
        const summary = JSON.parse(entries["lint-divergence-summary.json"]);
        summary.evidence.commitSha = "b".repeat(40);
        entries["lint-divergence-summary.json"] = `${JSON.stringify(summary, null, 2)}\n`;
      },
      /lint divergence summary is not exact release evidence/,
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

test("release preflight requires a selected Real Project Matrix run", () => {
  const run = successfulReleaseRun("Real Project Matrix", 530);
  assert.equal(requireRealProjectMatrixRun(new Map([["Real Project Matrix", run]])), run);
  for (const selected of [new Map(), new Map([["Benchmark", run]])] as Map<string, unknown>[]) {
    assert.throws(
      () => requireRealProjectMatrixRun(selected),
      /Real Project Matrix release evidence is required/,
    );
  }
});
