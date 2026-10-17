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
} from "../../../tools/support/compat/github/release-preflight-evidence.mjs";
import { requiredRealProjectMatrixShardCount } from "../../../tools/support/compat/github/release-preflight-matrix-evidence.mjs";
import { readRepoFile } from "../support/github-workflows.ts";
import { matrixSourceFixture } from "../support/real-project-matrix-source.ts";
import {
  releaseSha,
  successfulReleaseJob,
  successfulReleaseRun,
} from "../support/release-preflight.ts";

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

  assert.deepEqual(requiredReleaseWorkflows, ["Check"]);
  assert.throws(() => selectRequiredWorkflowRuns([], releaseSha), /Check: missing/);
  for (const patch of [
    { head_sha: "b".repeat(40) },
    { path: ".github/workflows/foreign.yml" },
    { event: "pull_request" },
    { conclusion: "failure" },
    { conclusion: "skipped" },
    { conclusion: "cancelled" },
    { status: "in_progress", conclusion: null },
  ]) {
    assert.throws(
      () => selectRequiredWorkflowRuns([{ ...greenRuns[0], ...patch }], releaseSha),
      /Check:/,
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
  const fullJobs = [
    "test-scripts",
    "test-js-packages",
    "build-js-packages",
    "full-js-report",
    "Full Rust workspace and doctests",
    "clippy-and-test",
    "full-rust-report",
  ];
  assert.deepEqual(
    requiredWorkflowJobNames("Check", { event: "workflow_dispatch", head_branch: "main" }),
    fullJobs,
  );
  assert.deepEqual(
    requiredWorkflowJobNames("Check", { event: "workflow_dispatch", head_branch: "v0.458.0" }),
    [
      "test-scripts",
      "test-js-packages",
      "build-js-packages",
      "full-js-report",
      "Full Rust workspace and doctests",
      "clippy-and-test",
      "full-rust-report",
      ...crates.map((name) => `cargo-semver-checks (${name})`),
    ],
  );

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
    .filter((name) => name !== "Check")
    .map((name, index) => successfulReleaseRun(name, index + 1));
  const olderSuccess = {
    ...successfulReleaseRun("Check", 50),
    run_started_at: "2026-07-12T00:50:00Z",
  };
  const newerCancellation = {
    ...successfulReleaseRun("Check", 51),
    run_started_at: "2026-07-12T00:51:00Z",
    conclusion: "cancelled",
  };
  assert.throws(
    () => selectRequiredWorkflowRuns([...greenRuns, olderSuccess, newerCancellation], releaseSha),
    /cancelled/,
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
      [...greenRuns, supersededPending, successfulReleaseRun("Check", 52)],
      releaseSha,
    ),
  );
});

test("release full JS jobs reject missing, duplicate, unfinished and unsuccessful qualification", () => {
  const jobs = requiredWorkflowJobNames("Check").map(successfulReleaseJob);
  for (const name of requiredWorkflowJobNames("Check")) {
    const original = jobs.find((job) => job.name === name);
    assert.ok(original);
    for (const variant of [
      jobs.filter((job) => job.name !== name),
      [...jobs, original],
      ...["skipped", "failure", "cancelled"].map((conclusion) =>
        jobs.map((job) => (job.name === name ? { ...job, conclusion } : job)),
      ),
      jobs.map((job) => (job.name === name ? { ...job, status: "in_progress" } : job)),
    ]) {
      assert.throws(() => assertRequiredWorkflowJobs("Check", variant), new RegExp(name));
      assert.ok(summarizeRequiredWorkflowJobFailures("Check", variant)?.includes(name));
    }
  }
});

test("matrix-sensitive release gates require every successful job", () => {
  const releaseRun = successfulReleaseRun("Check", 1);
  const checkJobs = requiredWorkflowJobNames("Check", releaseRun).map(successfulReleaseJob);
  assert.doesNotThrow(() => assertRequiredWorkflowJobs("Check", checkJobs, releaseRun));
  assert.throws(() => assertRequiredWorkflowJobs("Check", []), /test-scripts/);
  const semverName = "cargo-semver-checks (vize_armature)";
  const withoutSemver = checkJobs.filter((job) => job.name !== semverName);
  assert.throws(
    () => assertRequiredWorkflowJobs("Check", withoutSemver, releaseRun),
    /cargo-semver-checks \(vize_armature\) job; found 0/,
  );
  assert.equal(
    summarizeRequiredWorkflowJobFailures("Check", withoutSemver, releaseRun),
    `required jobs: ${semverName}=missing`,
  );
  for (const conclusion of ["skipped", "failure"]) {
    const jobs = checkJobs.map((job) => (job.name === semverName ? { ...job, conclusion } : job));
    assert.throws(
      () => assertRequiredWorkflowJobs("Check", jobs, releaseRun),
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

  const source = matrixSourceFixture();
  const assertMatrixJobs = (jobs: ReturnType<typeof successfulReleaseJob>[]) =>
    assertRequiredWorkflowJobs(
      "Real Project Matrix",
      jobs,
      { head_sha: source.legacySha },
      source.repoRoot,
    );
  const realProjectJobs = Array.from({ length: requiredRealProjectMatrixShardCount }, (_, shard) =>
    successfulReleaseJob(`real projects (${shard}/${requiredRealProjectMatrixShardCount})`),
  );
  assert.doesNotThrow(() => assertMatrixJobs(realProjectJobs));
  assert.throws(
    () => assertMatrixJobs(realProjectJobs.slice(1)),
    new RegExp(`real projects \\(0\\/${requiredRealProjectMatrixShardCount}\\)`),
  );
  assert.throws(
    () =>
      assertMatrixJobs([
        ...realProjectJobs,
        successfulReleaseJob(`real projects (0/${requiredRealProjectMatrixShardCount})`),
      ]),
    new RegExp(`real projects \\(0\\/${requiredRealProjectMatrixShardCount}\\).*found 2`),
  );
  assert.throws(
    () =>
      assertMatrixJobs([
        { ...realProjectJobs[0], conclusion: "failure" },
        ...realProjectJobs.slice(1),
      ]),
    new RegExp(
      `real projects \\(0\\/${requiredRealProjectMatrixShardCount}\\) is completed\\/failure`,
    ),
  );
});

test("a cancelled Check blocks publication", () => {
  assert.throws(() =>
    selectRequiredWorkflowRuns(
      [{ ...successfulReleaseRun("Check", 1), conclusion: "cancelled" }],
      releaseSha,
    ),
  );
});
