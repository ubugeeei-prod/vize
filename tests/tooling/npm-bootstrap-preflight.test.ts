import assert from "node:assert/strict";
import { test } from "node:test";

import {
  assertPackageIsUnpublished,
  bootstrapArtifacts,
  bootstrapPackages,
  requiredFailedReleaseJobs,
  requiredSkippedReleaseJobs,
  requiredSuccessfulReleaseJobs,
  validateBootstrapManifest,
  validateBootstrapRequest,
  validateDownloadedArtifact,
  validateRegistryResponse,
  validateReleaseArtifact,
  validateReleaseCommit,
  validateReleaseJobs,
  validateReleaseRun,
  verifyReleaseRunEvidence,
} from "../../tools/github/npm-bootstrap-preflight.mjs";

const tagName = "v1.2.3";
const tagSha = "a".repeat(40);
const mainSha = "b".repeat(40);
const releaseRunId = "123456789";
const repository = "ubugeeei-prod/vize";
const packagePath = "npm/framework/nuxt-lint-config";
const packageName = "@vizejs/nuxt-lint-config";
const artifactName = "release-package-nuxt-lint-config";
const cargoToml = '[workspace.package]\nversion = "1.2.3"\n';
const packageManifest = JSON.stringify({
  name: packageName,
  version: "1.2.3",
  publishConfig: { access: "public" },
});

function request(overrides: Record<string, string> = {}) {
  return validateBootstrapRequest({
    tagName,
    packagePath,
    releaseRunId,
    workflowRef: "refs/heads/main",
    workflowSha: tagSha,
    ...overrides,
  });
}

function releaseRun(overrides: Record<string, unknown> = {}) {
  return {
    id: Number(releaseRunId),
    name: "Release",
    path: ".github/workflows/release.yml",
    event: "push",
    status: "completed",
    conclusion: "failure",
    head_branch: tagName,
    head_sha: tagSha,
    head_repository: { full_name: repository },
    ...overrides,
  };
}

function releaseJobs() {
  return [
    ...requiredSuccessfulReleaseJobs.map((name) => ({
      name,
      status: "completed",
      conclusion: "success",
    })),
    ...requiredFailedReleaseJobs.map((name) => ({
      name,
      status: "completed",
      conclusion: "failure",
    })),
    ...requiredSkippedReleaseJobs.map((name) => ({
      name,
      status: "completed",
      conclusion: "skipped",
    })),
  ];
}

function releaseArtifact(overrides: Record<string, unknown> = {}) {
  return {
    name: artifactName,
    expired: false,
    workflow_run: { id: Number(releaseRunId), head_branch: tagName, head_sha: tagSha },
    ...overrides,
  };
}

test("npm bootstrap allowlist binds one package path to one Release artifact", () => {
  assert.deepEqual([...bootstrapPackages], [[packagePath, packageName]]);
  assert.deepEqual([...bootstrapArtifacts], [[packagePath, artifactName]]);
  assert.deepEqual(request(), {
    artifactName,
    packageName,
    packagePath,
    releaseRunId,
    tagName,
    workflowSha: tagSha,
  });
});

test("npm bootstrap rejects non-main dispatches and every non-allowlisted path", () => {
  assert.throws(() => request({ workflowRef: "refs/heads/release" }), /dispatched from main/);
  for (const rejectedPath of [
    "npm/framework/nuxt",
    "npm/framework/nuxt-lint-config/..",
    "npm/framework/nuxt-lint-config\n--tag latest",
    "",
  ]) {
    assert.throws(() => request({ packagePath: rejectedPath }), /not approved/, rejectedPath);
  }
});

test("npm bootstrap validates tag, run ID, and dispatch SHA before using them", () => {
  for (const rejectedTag of [
    "1.2.3",
    "v01.2.3",
    "v1.2.3-alpha..1",
    "v1.2.3:refs/heads/main",
    "v1.2.*",
    "",
  ]) {
    assert.throws(() => request({ tagName: rejectedTag }), /strict v-prefixed SemVer/, rejectedTag);
  }
  for (const rejectedRunId of ["0", "-1", "12x", "9007199254740992", ""]) {
    assert.throws(() => request({ releaseRunId: rejectedRunId }), /positive safe integer/);
  }
  assert.throws(() => request({ workflowSha: "main" }), /GITHUB_SHA must be a full commit SHA/);
  assert.doesNotThrow(() => request({ tagName: "v1.2.3-rc.1" }));
});

test("npm bootstrap binds tag, workspace, and public package metadata to one version", () => {
  assert.equal(
    validateBootstrapManifest({
      tagName,
      tagSha,
      packagePath,
      packageName,
      cargoToml,
      packageManifest,
    }),
    "1.2.3",
  );
  assert.throws(
    () =>
      validateBootstrapManifest({
        tagName: "v1.2.4",
        tagSha,
        packagePath,
        packageName,
        cargoToml,
        packageManifest,
      }),
    /does not match workspace version/,
  );
  assert.throws(
    () =>
      validateBootstrapManifest({
        tagName,
        tagSha,
        packagePath,
        packageName,
        cargoToml,
        packageManifest: JSON.stringify({
          name: "@vizejs/wrong",
          version: "1.2.3",
          publishConfig: { access: "public" },
        }),
      }),
    /expected @vizejs\/nuxt-lint-config/,
  );
  assert.throws(
    () =>
      validateBootstrapManifest({
        tagName,
        tagSha,
        packagePath,
        packageName,
        cargoToml,
        packageManifest: JSON.stringify({ name: packageName, version: "1.2.3" }),
      }),
    /publishConfig\.access as public/,
  );
});

test("npm bootstrap requires the tag to equal dispatch SHA on main first-parent", () => {
  assert.doesNotThrow(() =>
    validateReleaseCommit({ tagSha, workflowSha: tagSha, mainSha, isOnFirstParent: true }),
  );
  assert.throws(
    () =>
      validateReleaseCommit({
        tagSha,
        workflowSha: "c".repeat(40),
        mainSha,
        isOnFirstParent: true,
      }),
    /exactly match repository dispatch SHA/,
  );
  assert.throws(
    () => validateReleaseCommit({ tagSha, workflowSha: tagSha, mainSha, isOnFirstParent: false }),
    /not on the first-parent history/,
  );
});

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

test("npm bootstrap binds the downloaded package manifest to preflight outputs", () => {
  assert.doesNotThrow(() =>
    validateDownloadedArtifact({
      packageManifest,
      expectedName: packageName,
      expectedVersion: "1.2.3",
    }),
  );
  assert.throws(
    () =>
      validateDownloadedArtifact({
        packageManifest,
        expectedName: packageName,
        expectedVersion: "1.2.4",
      }),
    /expected @vizejs\/nuxt-lint-config@1\.2\.4/,
  );
  assert.throws(
    () =>
      validateDownloadedArtifact({
        packageManifest: "{",
        expectedName: packageName,
        expectedVersion: "1.2.3",
      }),
    /invalid package\.json/,
  );
});

test("npm bootstrap proceeds only on an authoritative registry 404", async () => {
  assert.doesNotThrow(() => validateRegistryResponse(packageName, 404));
  assert.throws(() => validateRegistryResponse(packageName, 200), /already exists on npm/);
  assert.throws(() => validateRegistryResponse(packageName, 500), /returned HTTP 500/);

  let requestedUrl = "";
  await assertPackageIsUnpublished(packageName, async (url) => {
    requestedUrl = String(url);
    return new Response(null, { status: 404 });
  });
  assert.equal(requestedUrl, "https://registry.npmjs.org/%40vizejs%2Fnuxt-lint-config");
});
