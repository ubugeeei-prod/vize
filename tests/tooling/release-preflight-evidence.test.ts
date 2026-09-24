import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import {
  assertRequiredWorkflowJobs,
  requiredReleaseWorkflowEvidence,
  requiredReleaseWorkflows,
  requiredWorkflowJobNames,
  selectRequiredWorkflowRuns,
  summarizeRequiredWorkflowJobFailures,
} from "../../tools/support/compat/github/release-preflight-evidence.mjs";
import { requiredRealProjectMatrixShardCount } from "../../tools/support/compat/github/release-preflight-matrix-evidence.mjs";
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
          run.path === ".github/workflows/check.yml" ? { ...run, event: "push" } : run,
        ),
        releaseSha,
      ),
    /Check: missing workflow_dispatch run/,
  );
  for (const path of [".github/workflows/miri.yml", ".github/workflows/build-docs.yml"]) {
    assert.throws(
      () =>
        selectRequiredWorkflowRuns(
          greenRuns.map((run) => (run.path === path ? { ...run, event: "push" } : run)),
          releaseSha,
        ),
      /missing workflow_dispatch run/,
      path,
    );
  }
});

test("release evidence ignores a failed optional Benchmark run", () => {
  const runs = requiredReleaseWorkflows.map((name, index) => successfulReleaseRun(name, index + 1));
  runs.push({ ...successfulReleaseRun("Benchmark", 99), conclusion: "failure" });
  assert.deepEqual(
    [...selectRequiredWorkflowRuns(runs, releaseSha).keys()],
    requiredReleaseWorkflows,
  );
});

test("release Check job evidence covers the exact SemVer matrix in Rust and JS", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "check.yml")) as {
    jobs?: {
      "semver-checks"?: {
        name?: string;
        strategy?: { matrix?: { crate?: string[] } };
      };
    };
  };
  const semverJob = workflow.jobs?.["semver-checks"];
  assert.equal(semverJob?.name, "cargo-semver-checks (${{ matrix.crate }})");
  const crates = semverJob?.strategy?.matrix?.crate;
  assert.ok(crates);
  assert.equal(crates.length, 12);
  assert.deepEqual(requiredWorkflowJobNames("Check"), [
    "test-scripts",
    ...crates.map((name) => `cargo-semver-checks (${name})`),
  ]);

  const rust = readRepoFile("tools", "commands", "ci", "github", "release-preflight.rs");
  const rustCrateBlock = rust.match(
    /const REQUIRED_SEMVER_CRATES: &\[&str\] = &\[([\s\S]*?)\];/,
  )?.[1];
  assert.ok(rustCrateBlock);
  const rustCrates = [...rustCrateBlock.matchAll(/"([^"]+)"/g)].map((match) => match[1]);
  assert.deepEqual(rustCrates, crates);
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
  const checkJobs = requiredWorkflowJobNames("Check").map(successfulReleaseJob);
  assert.doesNotThrow(() => assertRequiredWorkflowJobs("Check", checkJobs));
  assert.throws(() => assertRequiredWorkflowJobs("Check", []), /test-scripts/);
  const semverName = "cargo-semver-checks (vize_armature)";
  const withoutSemver = checkJobs.filter((job) => job.name !== semverName);
  assert.throws(
    () => assertRequiredWorkflowJobs("Check", withoutSemver),
    /cargo-semver-checks \(vize_armature\) job; found 0/,
  );
  assert.equal(
    summarizeRequiredWorkflowJobFailures("Check", withoutSemver),
    `required jobs: ${semverName}=missing`,
  );
  for (const conclusion of ["skipped", "failure"]) {
    const jobs = checkJobs.map((job) => (job.name === semverName ? { ...job, conclusion } : job));
    assert.throws(
      () => assertRequiredWorkflowJobs("Check", jobs),
      new RegExp(`cargo-semver-checks \\(vize_armature\\) is completed/${conclusion}`),
    );
  }

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

test("required workflow selection can report failed release jobs inline", () => {
  const runs = requiredReleaseWorkflows.map((name, index) => successfulReleaseRun(name, index + 1));
  const matrix = runs.find((run) => run.name === "Real Project Matrix");
  assert.ok(matrix);
  matrix.conclusion = "cancelled";

  const matrixJobs = Array.from({ length: requiredRealProjectMatrixShardCount }, (_, shard) =>
    successfulReleaseJob(`real projects (${shard}/${requiredRealProjectMatrixShardCount})`),
  );
  matrixJobs[0] = { ...matrixJobs[0], conclusion: "cancelled" };
  matrixJobs[12] = { ...matrixJobs[12], conclusion: "failure" };

  const summary = summarizeRequiredWorkflowJobFailures("Real Project Matrix", matrixJobs);
  assert.equal(
    summary,
    `required jobs: real projects (0/${requiredRealProjectMatrixShardCount})=completed/cancelled, real projects (12/${requiredRealProjectMatrixShardCount})=completed/failure`,
  );
  assert.throws(
    () =>
      selectRequiredWorkflowRuns(
        runs,
        releaseSha,
        requiredReleaseWorkflows,
        new Map(),
        new Map(),
        new Map([["Real Project Matrix", summary]]),
      ),
    new RegExp(
      `Real Project Matrix: completed\\/cancelled[\\s\\S]*required jobs: real projects \\(0\\/${requiredRealProjectMatrixShardCount}\\)=completed\\/cancelled, real projects \\(12\\/${requiredRealProjectMatrixShardCount}\\)=completed\\/failure`,
    ),
  );
});
