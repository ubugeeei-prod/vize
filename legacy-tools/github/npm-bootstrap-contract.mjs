import {
  assertReleaseCommitIsOnMainFirstParent,
  assertReleaseMetadata,
} from "./release-preflight-core.mjs";
import { githubApiPages, githubApiRequest } from "./release-preflight-github.mjs";

export const bootstrapPackages = new Map([
  ["npm/framework/nuxt-lint-config", "@vizejs/nuxt-lint-config"],
]);
export const bootstrapArtifacts = new Map([
  ["npm/framework/nuxt-lint-config", "release-package-nuxt-lint-config"],
]);
export const requiredSuccessfulReleaseJobs = [
  "Build release npm packages",
  "Smoke release npm package installs",
  "release-preflight / Verify release safety contract",
];
export const requiredFailedReleaseJobs = ["Release @vizejs/nuxt-lint-config to npm"];
export const requiredSkippedReleaseJobs = ["Release @vizejs/nuxt to npm", "Create GitHub Release"];

const registryOrigin = "https://registry.npmjs.org";
const releaseTagPattern =
  /^v(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-(?:(?:0|[1-9]\d*)|(?:\d*[A-Za-z-][0-9A-Za-z-]*))(?:\.(?:(?:0|[1-9]\d*)|(?:\d*[A-Za-z-][0-9A-Za-z-]*)))*)?$/;

export function validateBootstrapRequest({
  tagName,
  packagePath,
  releaseRunId,
  workflowRef,
  workflowSha,
}) {
  if (workflowRef !== "refs/heads/main") {
    throw new Error(`npm bootstrap must be dispatched from main, got ${workflowRef || "no ref"}`);
  }
  const packageName = bootstrapPackages.get(packagePath);
  const artifactName = bootstrapArtifacts.get(packagePath);
  if (packageName == null || artifactName == null) {
    throw new Error(`Package path is not approved for npm bootstrap: ${packagePath || "(empty)"}`);
  }
  if (!releaseTagPattern.test(tagName ?? "")) {
    throw new Error(`Release tag must be a strict v-prefixed SemVer, got ${tagName || "(empty)"}`);
  }
  if (!/^[1-9]\d*$/.test(releaseRunId ?? "") || !Number.isSafeInteger(Number(releaseRunId))) {
    throw new Error(
      `Release run ID must be a positive safe integer, got ${releaseRunId || "(empty)"}`,
    );
  }
  if (!/^[0-9a-f]{40}$/.test(workflowSha ?? "")) {
    throw new Error(`GITHUB_SHA must be a full commit SHA, got ${workflowSha || "(empty)"}`);
  }
  return { artifactName, packageName, packagePath, releaseRunId, tagName, workflowSha };
}

export function validateBootstrapManifest({
  tagName,
  tagSha,
  packagePath,
  packageName,
  cargoToml,
  packageManifest,
}) {
  const version = assertReleaseMetadata({
    tag: tagName,
    sha: tagSha,
    cargoToml,
    packageManifests: [{ path: `${packagePath}/package.json`, content: packageManifest }],
  });
  const packageJson = JSON.parse(packageManifest);
  if (packageJson.name !== packageName) {
    throw new Error(
      `${packagePath}/package.json names ${String(packageJson.name)}, expected ${packageName}`,
    );
  }
  if (packageJson.publishConfig?.access !== "public") {
    throw new Error(`${packageName} must declare publishConfig.access as public`);
  }
  return version;
}

export function validateReleaseCommit({ tagSha, workflowSha, mainSha, isOnFirstParent }) {
  if (
    !/^[0-9a-f]{40}$/.test(tagSha) ||
    !/^[0-9a-f]{40}$/.test(workflowSha) ||
    !/^[0-9a-f]{40}$/.test(mainSha)
  ) {
    throw new Error("The release tag and origin/main must resolve to full commit SHAs");
  }
  if (tagSha !== workflowSha) {
    throw new Error(
      `Release tag commit ${tagSha} must exactly match repository dispatch SHA ${workflowSha}`,
    );
  }
  assertReleaseCommitIsOnMainFirstParent(tagSha, mainSha, isOnFirstParent);
}

function exactJob(jobs, name) {
  const matches = jobs.filter((job) => job.name === name);
  if (matches.length !== 1) {
    throw new Error(`Release run must contain exactly one ${name} job, found ${matches.length}`);
  }
  return matches[0];
}

export function validateReleaseRun({ run, releaseRunId, repository, tagName, tagSha }) {
  const expected = {
    conclusion: "failure",
    event: "push",
    head_branch: tagName,
    head_sha: tagSha,
    id: releaseRunId,
    name: "Release",
    path: ".github/workflows/release.yml",
    repository,
    status: "completed",
  };
  const actual = {
    conclusion: run?.conclusion,
    event: run?.event,
    head_branch: run?.head_branch,
    head_sha: run?.head_sha,
    id: String(run?.id ?? ""),
    name: run?.name,
    path: run?.path,
    repository: run?.head_repository?.full_name,
    status: run?.status,
  };
  const mismatches = Object.keys(expected).filter((key) => actual[key] !== expected[key]);
  if (mismatches.length > 0) {
    throw new Error(
      `Release run ${releaseRunId} does not match the failed exact-tag release contract: ${mismatches
        .map((key) => `${key}=${String(actual[key])}`)
        .join(", ")}`,
    );
  }
}

export function validateReleaseJobs(jobs) {
  const jobNameCounts = new Map();
  for (const job of jobs) {
    jobNameCounts.set(job.name, (jobNameCounts.get(job.name) ?? 0) + 1);
    if (job.status !== "completed") {
      throw new Error(`Every Release job must be terminal; ${String(job.name)} is ${job.status}`);
    }
  }
  const duplicates = [...jobNameCounts].filter(([, count]) => count !== 1);
  if (duplicates.length > 0) {
    throw new Error(
      `Release job names must be unique: ${duplicates.map(([name, count]) => `${String(name)}=${count}`).join(", ")}`,
    );
  }

  for (const name of requiredSuccessfulReleaseJobs) {
    const job = exactJob(jobs, name);
    if (job.status !== "completed" || job.conclusion !== "success") {
      throw new Error(`${name} must be completed/success, got ${job.status}/${job.conclusion}`);
    }
  }
  for (const name of requiredFailedReleaseJobs) {
    const job = exactJob(jobs, name);
    if (job.status !== "completed" || job.conclusion !== "failure") {
      throw new Error(`${name} must be completed/failure, got ${job.status}/${job.conclusion}`);
    }
  }
  for (const name of requiredSkippedReleaseJobs) {
    const job = exactJob(jobs, name);
    if (job.status !== "completed" || job.conclusion !== "skipped") {
      throw new Error(`${name} must be completed/skipped, got ${job.status}/${job.conclusion}`);
    }
  }

  /** @type {Map<string, string>} */
  const allowedNonSuccess = new Map([
    ...requiredFailedReleaseJobs.map((name) => [name, "failure"]),
    ...requiredSkippedReleaseJobs.map((name) => [name, "skipped"]),
  ]);
  for (const job of jobs) {
    const expected = allowedNonSuccess.get(job.name) ?? "success";
    if (job.conclusion !== expected) {
      throw new Error(
        `Unexpected Release job conclusion: ${String(job.name)}=${String(job.conclusion)}, expected ${String(expected)}`,
      );
    }
  }
}

export function validateReleaseArtifact({
  artifacts,
  artifactName,
  releaseRunId,
  tagName,
  tagSha,
}) {
  const matches = artifacts.filter((artifact) => artifact.name === artifactName);
  if (matches.length !== 1) {
    throw new Error(
      `Release run must contain exactly one ${artifactName} artifact, found ${matches.length}`,
    );
  }
  const artifact = matches[0];
  if (artifact.expired === true) {
    throw new Error(`Release artifact ${artifactName} has expired`);
  }
  const source = artifact.workflow_run;
  if (
    String(source?.id ?? "") !== releaseRunId ||
    source?.head_branch !== tagName ||
    source?.head_sha !== tagSha
  ) {
    throw new Error(`Release artifact ${artifactName} is not bound to ${tagName} (${tagSha})`);
  }
}

export async function verifyReleaseRunEvidence({
  apiUrl,
  repository,
  token,
  releaseRunId,
  tagName,
  tagSha,
  artifactName,
  fetchImpl = globalThis.fetch,
}) {
  if (repository !== "ubugeeei-prod/vize" || token === "") {
    throw new Error("GITHUB_REPOSITORY must be ubugeeei-prod/vize and GITHUB_TOKEN is required");
  }
  const requestOptions = { apiUrl, repository, token, fetchImpl };
  const runResponse = await githubApiRequest({
    ...requestOptions,
    resource: `actions/runs/${releaseRunId}`,
  });
  let run;
  try {
    run = JSON.parse(runResponse.body);
  } catch {
    throw new Error(`GitHub API returned invalid JSON for Release run ${releaseRunId}`);
  }
  validateReleaseRun({ run, releaseRunId, repository, tagName, tagSha });

  const jobs = await githubApiPages({
    ...requestOptions,
    resource: `actions/runs/${releaseRunId}/jobs`,
    collection: "jobs",
  });
  validateReleaseJobs(jobs);
  const artifacts = await githubApiPages({
    ...requestOptions,
    resource: `actions/runs/${releaseRunId}/artifacts`,
    collection: "artifacts",
  });
  validateReleaseArtifact({ artifacts, artifactName, releaseRunId, tagName, tagSha });
}

export function validateRegistryResponse(packageName, status) {
  if (status === 404) return;
  if (status === 200) {
    throw new Error(`${packageName} already exists on npm; bootstrap is only for first publish`);
  }
  throw new Error(`npm registry returned HTTP ${status} while checking ${packageName}`);
}

export async function assertPackageIsUnpublished(packageName, fetchImpl = globalThis.fetch) {
  const response = await fetchImpl(`${registryOrigin}/${encodeURIComponent(packageName)}`, {
    headers: {
      accept: "application/vnd.npm.install-v1+json",
      "user-agent": "vize-npm-bootstrap (https://github.com/ubugeeei-prod/vize)",
    },
    signal: AbortSignal.timeout(30_000),
  });
  validateRegistryResponse(packageName, response.status);
}

export function validateDownloadedArtifact({ packageManifest, expectedName, expectedVersion }) {
  let packageJson;
  try {
    packageJson = JSON.parse(packageManifest);
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(`Downloaded Release artifact has an invalid package.json: ${detail}`, {
      cause: error,
    });
  }
  if (packageJson.name !== expectedName || packageJson.version !== expectedVersion) {
    throw new Error(
      `Downloaded Release artifact is ${String(packageJson.name)}@${String(packageJson.version)}, expected ${expectedName}@${expectedVersion}`,
    );
  }
}
