import assert from "node:assert/strict";
import { test } from "node:test";

import {
  assertRequiredWorkflowJobs,
  requiredReleaseWorkflowEvidence,
  requiredReleaseWorkflows,
  selectRequiredWorkflowRuns,
} from "../../legacy-tools/github/release-preflight-evidence.mjs";
import { requiredRealProjectMatrixShardCount } from "../../legacy-tools/github/release-preflight-matrix-evidence.mjs";
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
    .filter((name) => name !== "Docs build")
    .map((name, index) => successfulReleaseRun(name, index + 1));
  const olderSuccess = {
    ...successfulReleaseRun("Docs build", 50),
    run_started_at: "2026-07-12T00:50:00Z",
  };
  const newerCancellation = {
    ...successfulReleaseRun("Docs build", 51),
    run_started_at: "2026-07-12T00:51:00Z",
    conclusion: "cancelled",
  };
  assert.throws(
    () => selectRequiredWorkflowRuns([...greenRuns, olderSuccess, newerCancellation], releaseSha),
    /Docs build: completed\/cancelled/,
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
      [...greenRuns, supersededPending, successfulReleaseRun("Docs build", 52)],
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

  const realProjectJobs = Array.from({ length: requiredRealProjectMatrixShardCount }, (_, shard) =>
    successfulReleaseJob(`real projects (${shard}/${requiredRealProjectMatrixShardCount})`),
  );
  assert.doesNotThrow(() => assertRequiredWorkflowJobs("Real Project Matrix", realProjectJobs));
  assert.throws(
    () => assertRequiredWorkflowJobs("Real Project Matrix", realProjectJobs.slice(1)),
    new RegExp(`real projects \\(0\\/${requiredRealProjectMatrixShardCount}\\)`),
  );
  assert.throws(
    () =>
      assertRequiredWorkflowJobs("Real Project Matrix", [
        ...realProjectJobs,
        successfulReleaseJob(`real projects (0/${requiredRealProjectMatrixShardCount})`),
      ]),
    new RegExp(`real projects \\(0\\/${requiredRealProjectMatrixShardCount}\\).*found 2`),
  );
  assert.throws(
    () =>
      assertRequiredWorkflowJobs("Real Project Matrix", [
        { ...realProjectJobs[0], conclusion: "failure" },
        ...realProjectJobs.slice(1),
      ]),
    new RegExp(
      `real projects \\(0\\/${requiredRealProjectMatrixShardCount}\\) is completed\\/failure`,
    ),
  );
});
