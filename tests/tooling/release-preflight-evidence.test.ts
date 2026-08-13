import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";

import {
  assertRealProjectMatrixReleaseArtifacts,
  assertRequiredWorkflowJobs,
  requiredRealProjectMatrixShardCount,
  requiredReleaseWorkflowEvidence,
  requiredReleaseWorkflows,
  selectRequiredWorkflowRuns,
} from "../../tools/github/release-preflight-evidence.mjs";
import { readRepoFile } from "./support/github-workflows.ts";
import {
  releaseSha,
  successfulReleaseJob,
  successfulReleaseRun,
} from "./support/release-preflight.ts";

test("release evidence paths identify the declared workflow names", () => {
  for (const [workflowName, evidence] of requiredReleaseWorkflowEvidence) {
    const workflow = readRepoFile(...evidence.path.split("/"));
    assert.match(workflow, new RegExp(`^name: ${workflowName}$`, "m"));
  }
});

test("required workflow selection fails closed for missing, stale, red, or wrong-origin gates", () => {
  const greenRuns = requiredReleaseWorkflows.map((name, index) =>
    successfulReleaseRun(name, index + 1),
  );
  assert.deepEqual(
    [...selectRequiredWorkflowRuns(greenRuns, releaseSha).keys()],
    requiredReleaseWorkflows,
  );

  assert.throws(
    () =>
      selectRequiredWorkflowRuns(
        greenRuns.filter((run) => run.path !== ".github/workflows/fuzz.yml"),
        releaseSha,
      ),
    /Fuzz: missing/,
  );
  assert.throws(
    () =>
      selectRequiredWorkflowRuns(
        greenRuns.map((run) =>
          run.path === ".github/workflows/miri.yml" ? { ...run, head_sha: "b".repeat(40) } : run,
        ),
        releaseSha,
      ),
    /Miri: missing/,
  );
  assert.throws(
    () =>
      selectRequiredWorkflowRuns(
        greenRuns.map((run) =>
          run.path === ".github/workflows/check.yml" ? { ...run, conclusion: "failure" } : run,
        ),
        releaseSha,
      ),
    /Check: completed\/failure/,
  );
  assert.throws(
    () =>
      selectRequiredWorkflowRuns(
        greenRuns.map((run) =>
          run.path === ".github/workflows/check.yml"
            ? { ...run, head_branch: "release-candidate" }
            : run,
        ),
        releaseSha,
      ),
    /Check: missing push run/,
  );
});

test("newest matching run wins across cancellation, reruns, and concurrent runs", () => {
  const greenRuns = requiredReleaseWorkflows
    .filter((name) => name !== "App E2E")
    .map((name, index) => successfulReleaseRun(name, index + 1));
  const olderSuccess = {
    ...successfulReleaseRun("App E2E", 50),
    run_started_at: "2026-07-12T00:50:00Z",
  };
  const newerCancellation = {
    ...successfulReleaseRun("App E2E", 51),
    run_started_at: "2026-07-12T00:51:00Z",
    conclusion: "cancelled",
  };
  assert.throws(
    () => selectRequiredWorkflowRuns([...greenRuns, olderSuccess, newerCancellation], releaseSha),
    /App E2E: completed\/cancelled/,
  );

  const rerunSuccess = {
    ...olderSuccess,
    run_attempt: 2,
    run_started_at: "2026-07-12T01:00:00Z",
  };
  assert.doesNotThrow(() =>
    selectRequiredWorkflowRuns([...greenRuns, rerunSuccess, newerCancellation], releaseSha),
  );

  const supersededPending = {
    ...olderSuccess,
    status: "queued",
    conclusion: null,
    run_started_at: "2026-07-12T00:40:00Z",
  };
  assert.doesNotThrow(() =>
    selectRequiredWorkflowRuns(
      [...greenRuns, supersededPending, successfulReleaseRun("App E2E", 52)],
      releaseSha,
    ),
  );
});

test("matrix-sensitive release gates require every successful job", () => {
  assert.doesNotThrow(() =>
    assertRequiredWorkflowJobs("Check", [successfulReleaseJob("test-scripts")]),
  );
  assert.throws(() => assertRequiredWorkflowJobs("Check", []), /test-scripts/);

  const appJobs = [successfulReleaseJob("app-e2e")];
  assert.doesNotThrow(() => assertRequiredWorkflowJobs("App E2E", appJobs));
  assert.throws(() => assertRequiredWorkflowJobs("App E2E", []), /app-e2e/);
  assert.throws(() => assertRequiredWorkflowJobs("App E2E", [...appJobs, ...appJobs]), /found 2/);

  const targets = [
    "linux-x64-gnu",
    "linux-arm64-gnu",
    "darwin-x64",
    "darwin-arm64",
    "win32-x64-msvc",
    "win32-arm64-msvc",
  ];
  const nativeJobs = [
    ...targets.map((target) => successfulReleaseJob(`Native host smoke (${target})`)),
    ...targets.flatMap((target) =>
      ["22", "24"].map((node) =>
        successfulReleaseJob(`Fresh install smoke (${target}, Node ${node})`),
      ),
    ),
  ];
  assert.doesNotThrow(() => assertRequiredWorkflowJobs("Native Smoke", nativeJobs));
  assert.throws(
    () => assertRequiredWorkflowJobs("Native Smoke", nativeJobs.slice(1)),
    /Native host smoke \(linux-x64-gnu\)/,
  );

  const realProjectJobs = Array.from({ length: 11 }, (_, shard) =>
    successfulReleaseJob(`real projects (${shard}/11)`),
  );
  assert.doesNotThrow(() => assertRequiredWorkflowJobs("Real Project Matrix", realProjectJobs));
  assert.throws(
    () => assertRequiredWorkflowJobs("Real Project Matrix", realProjectJobs.slice(1)),
    /real projects \(0\/11\)/,
  );
  assert.throws(
    () =>
      assertRequiredWorkflowJobs("Real Project Matrix", [
        ...realProjectJobs,
        successfulReleaseJob("real projects (0/11)"),
      ]),
    /real projects \(0\/11\).*found 2/,
  );
  assert.throws(
    () =>
      assertRequiredWorkflowJobs("Real Project Matrix", [
        { ...realProjectJobs[0], conclusion: "failure" },
        ...realProjectJobs.slice(1),
      ]),
    /real projects \(0\/11\) is completed\/failure/,
  );
});

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
