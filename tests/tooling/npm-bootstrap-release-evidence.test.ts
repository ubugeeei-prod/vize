import assert from "node:assert/strict";
import { test } from "node:test";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { repoRoot } from "./_helpers/moonbit.ts";

import {
  requiredSuccessfulReleaseJobs,
  validateReleaseArtifact,
  validateReleaseJobs,
  validateReleaseRun,
  verifyReleaseRunEvidence,
} from "../../legacy-tools/github/npm-bootstrap-contract.mjs";
import {
  artifactName,
  releaseArtifact,
  releaseJobs,
  releaseRun,
  releaseRunId,
  repository,
  tagName,
  tagSha,
} from "./support/npm-bootstrap.ts";

test("npm bootstrap accepts only the exact completed failed tag Release run", () => {
  assert.doesNotThrow(() =>
    validateReleaseRun({ run: releaseRun(), releaseRunId, repository, tagName, tagSha }),
  );
  for (const changed of [
    { id: 1 },
    { name: "Check" },
    { path: ".github/workflows/check.yml" },
    { event: "workflow_dispatch" },
    { status: "in_progress" },
    { conclusion: "success" },
    { head_branch: "main" },
    { head_sha: "c".repeat(40) },
    { head_repository: { full_name: "someone/fork" } },
  ]) {
    assert.throws(
      () =>
        validateReleaseRun({
          run: releaseRun(changed),
          releaseRunId,
          repository,
          tagName,
          tagSha,
        }),
      /does not match the failed exact-tag release contract/,
    );
  }
});

test("npm bootstrap requires exact unique successful gates and the failed target publish job", () => {
  assert.doesNotThrow(() => validateReleaseJobs(releaseJobs()));
  const missing = releaseJobs().slice(1);
  assert.throws(() => validateReleaseJobs(missing), /exactly one Build release npm packages/);
  const duplicate = [...releaseJobs(), releaseJobs()[0]];
  assert.throws(() => validateReleaseJobs(duplicate), /job names must be unique/);
  const failedGate = releaseJobs();
  failedGate[0] = { ...failedGate[0], conclusion: "failure" };
  assert.throws(() => validateReleaseJobs(failedGate), /completed\/success/);
  const successfulPublish = releaseJobs();
  successfulPublish[requiredSuccessfulReleaseJobs.length].conclusion = "success";
  assert.throws(() => validateReleaseJobs(successfulPublish), /completed\/failure/);
});

test("npm bootstrap rejects every unexpected non-terminal or non-success Release job", () => {
  const nonTerminal = releaseJobs();
  nonTerminal.push({ name: "Some new job", status: "in_progress", conclusion: null });
  assert.throws(() => validateReleaseJobs(nonTerminal), /Every Release job must be terminal/);

  for (const conclusion of [
    "failure",
    "cancelled",
    "timed_out",
    "action_required",
    "stale",
    "startup_failure",
    "neutral",
    "skipped",
  ]) {
    const jobs = releaseJobs();
    jobs.push({ name: `Unexpected ${conclusion}`, status: "completed", conclusion });
    assert.throws(() => validateReleaseJobs(jobs), /Unexpected Release job conclusion/, conclusion);
  }

  const wrongSkipped = releaseJobs();
  wrongSkipped.at(-1)!.conclusion = "success";
  assert.throws(() => validateReleaseJobs(wrongSkipped), /completed\/skipped/);
});

test("npm bootstrap requires one unexpired artifact bound to the Release run", () => {
  const validate = (artifacts: Array<Record<string, unknown>>) =>
    validateReleaseArtifact({ artifacts, artifactName, releaseRunId, tagName, tagSha });
  assert.doesNotThrow(() => validate([releaseArtifact()]));
  assert.throws(() => validate([]), /exactly one/);
  assert.throws(() => validate([releaseArtifact(), releaseArtifact()]), /found 2/);
  assert.throws(() => validate([releaseArtifact({ expired: true })]), /has expired/);
  assert.throws(
    () =>
      validate([
        releaseArtifact({
          workflow_run: { id: 1, head_branch: tagName, head_sha: tagSha },
        }),
      ]),
    /not bound/,
  );
});

test("npm bootstrap verifies run, jobs, and artifact through the GitHub API", async () => {
  const requested: string[] = [];
  const fetchImpl = async (input: string | URL | Request) => {
    const url = new URL(input instanceof Request ? input.url : input);
    requested.push(url.pathname);
    let payload;
    if (url.pathname.endsWith(`/actions/runs/${releaseRunId}`)) {
      payload = releaseRun();
    } else if (url.pathname.endsWith(`/actions/runs/${releaseRunId}/jobs`)) {
      payload = { jobs: releaseJobs() };
    } else if (url.pathname.endsWith(`/actions/runs/${releaseRunId}/artifacts`)) {
      payload = { artifacts: [releaseArtifact()] };
    } else {
      return new Response("not found", { status: 404 });
    }
    return new Response(JSON.stringify(payload), {
      status: 200,
      headers: { "content-type": "application/json" },
    });
  };

  await verifyReleaseRunEvidence({
    apiUrl: "https://api.github.test",
    repository,
    token: "token",
    releaseRunId,
    tagName,
    tagSha,
    artifactName,
    fetchImpl,
  });
  assert.deepEqual(requested, [
    `/repos/${repository}/actions/runs/${releaseRunId}`,
    `/repos/${repository}/actions/runs/${releaseRunId}/jobs`,
    `/repos/${repository}/actions/runs/${releaseRunId}/artifacts`,
  ]);
});

test("npm bootstrap recovers a promoted PR run only after all candidate gates passed", () => {
  const run = releaseRun({
    event: "workflow_dispatch",
    head_branch: `release/${tagName}`,
    display_title: `Release ${tagName} PR #42 @ ${tagSha}`,
  });
  assert.doesNotThrow(() => validateReleaseRun({ run, releaseRunId, repository, tagName, tagSha }));
  assert.throws(() =>
    validateReleaseRun({
      run: { ...run, display_title: "Release crate handoff" },
      releaseRunId,
      repository,
      tagName,
      tagSha,
    }),
  );
  const jobs = releaseJobs().filter(
    (job) => job.name !== "release-preflight / Verify release safety contract",
  );
  const gates = [
    "Authorize release candidate",
    "Release candidate ready",
    "candidate-preflight / Verify release safety contract",
    "candidate-preflight / Validate crates.io publish plan",
    "release-preflight / Wait for validated PR and tag promotion",
  ];
  jobs.push(...gates.map((name) => ({ name, status: "completed", conclusion: "success" })));
  jobs.push({
    name: "Release crates.io handoff crates",
    status: "completed",
    conclusion: "skipped",
  });
  assert.doesNotThrow(() => validateReleaseJobs(jobs));
  for (const name of gates) {
    assert.throws(
      () => validateReleaseJobs(jobs.filter((job) => job.name !== name)),
      /exactly one/,
    );
  }
  assert.doesNotThrow(() =>
    validateReleaseArtifact({
      artifacts: [
        releaseArtifact({
          workflow_run: {
            id: Number(releaseRunId),
            head_branch: `release/${tagName}`,
            head_sha: tagSha,
          },
        }),
      ],
      artifactName,
      releaseRunId,
      tagName,
      tagSha,
    }),
  );
});

test("Rust recovery accepts the exact promoted candidate and rejects stale identity", () => {
  const command = path.join(repoRoot, "tools/commands/ci/github/npm-bootstrap-preflight.rs");
  const run = releaseRun({
    event: "workflow_dispatch",
    head_branch: `release/${tagName}`,
    display_title: `Release ${tagName} PR #42 @ ${tagSha}`,
  });
  const invoke = (candidate: typeof run) =>
    spawnSync(
      "rust-script",
      [
        command,
        "__contract",
        "release-run",
        JSON.stringify({ run: candidate, releaseRunId, repository, tagName, tagSha }),
      ],
      { encoding: "utf8" },
    );
  const valid = invoke(run);
  assert.equal(valid.status, 0, `${valid.error ?? ""}\n${valid.stderr}`);
  for (const changed of [
    { head_sha: "d".repeat(40) },
    { head_branch: "main" },
    { display_title: "manual handoff" },
  ]) {
    assert.notEqual(invoke({ ...run, ...changed }).status, 0);
  }
});
