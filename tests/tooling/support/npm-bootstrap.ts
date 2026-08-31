import {
  requiredFailedReleaseJobs,
  requiredSkippedReleaseJobs,
  requiredSuccessfulReleaseJobs,
  validateBootstrapRequest,
} from "../../../legacy-tools/github/npm-bootstrap-contract.mjs";

export const tagName = "v1.2.3";
export const tagSha = "a".repeat(40);
export const mainSha = "b".repeat(40);
export const releaseRunId = "123456789";
export const repository = "ubugeeei-prod/vize";
export const packagePath = "npm/framework/nuxt-lint-config";
export const packageName = "@vizejs/nuxt-lint-config";
export const artifactName = "release-package-nuxt-lint-config";
export const cargoToml = '[workspace.package]\nversion = "1.2.3"\n';
export const packageManifest = JSON.stringify({
  name: packageName,
  version: "1.2.3",
  publishConfig: { access: "public" },
});

export function request(overrides: Record<string, string> = {}) {
  return validateBootstrapRequest({
    tagName,
    packagePath,
    releaseRunId,
    workflowRef: "refs/heads/main",
    workflowSha: tagSha,
    ...overrides,
  });
}

export function releaseRun(overrides: Record<string, unknown> = {}) {
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

export function releaseJobs() {
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

export function releaseArtifact(overrides: Record<string, unknown> = {}) {
  return {
    name: artifactName,
    expired: false,
    workflow_run: { id: Number(releaseRunId), head_branch: tagName, head_sha: tagSha },
    ...overrides,
  };
}
