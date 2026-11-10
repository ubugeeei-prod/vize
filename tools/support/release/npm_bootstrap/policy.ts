import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

type Json = Record<string, unknown>;

const repository = "ubugeeei-prod/vize";
const workflowPath = ".github/workflows/release.yml";
const artifactName = "npm-packages";
const strictVersion =
  /^(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-(?:(?:0|[1-9]\d*)|(?:\d*[A-Za-z-][0-9A-Za-z-]*))(?:\.(?:(?:0|[1-9]\d*)|(?:\d*[A-Za-z-][0-9A-Za-z-]*)))*)?$/;
const fullSha = /^[0-9a-f]{40}$/;

/** Deliberate first-publish exceptions. New packages require a reviewed policy change. */
export const bootstrapPackages = new Map([
  ["npm/framework/nuxt-lint-config", "@vizejs/nuxt-lint-config"],
  ["npm/plugin-sdk", "@vizejs/plugin-sdk"],
] as const);

function requireValue(value: unknown, message: string): asserts value {
  if (!value) throw new Error(message);
}

function object(value: unknown, message: string): Json {
  requireValue(value !== null && typeof value === "object" && !Array.isArray(value), message);
  return value as Json;
}

function sha(value: unknown, message: string): string {
  requireValue(typeof value === "string" && fullSha.test(value), message);
  return value;
}

function positiveId(value: unknown, message: string): string {
  requireValue(typeof value === "number" && Number.isSafeInteger(value) && value > 0, message);
  return value.toString();
}

export interface BootstrapRequest {
  packageName: string;
  packagePath: string;
  releaseRunId: string;
  tagName: string;
  version: string;
}

export function validateRequest(input: {
  packagePath: string;
  releaseRunId: string;
  tagName: string;
  version: string;
  workflowRef: string;
}): BootstrapRequest {
  requireValue(input.workflowRef === "refs/heads/main", "bootstrap must run from main");
  requireValue(strictVersion.test(input.version), "strict package version required");
  requireValue(input.tagName === `v${input.version}`, "tag must exactly match package version");
  requireValue(
    /^[1-9]\d*$/.test(input.releaseRunId) && Number.isSafeInteger(Number(input.releaseRunId)),
    "positive safe Release run ID required",
  );
  const packageName = bootstrapPackages.get(
    input.packagePath as typeof bootstrapPackages extends Map<infer K, string> ? K : never,
  );
  requireValue(packageName !== undefined, "package path is not approved for npm bootstrap");
  return { ...input, packageName };
}

export interface ReleaseRun {
  conclusion?: unknown;
  event?: unknown;
  head_branch?: unknown;
  head_repository?: unknown;
  head_sha?: unknown;
  id?: unknown;
  name?: unknown;
  path?: unknown;
  status?: unknown;
}

export function validateCandidate(
  request: BootstrapRequest,
  candidate: { sha: string; parents: readonly string[] },
  run: ReleaseRun,
): { candidateSha: string; parentSha: string } {
  sha(candidate.sha, "candidate tag commit SHA required");
  requireValue(candidate.parents.length === 1, "candidate tag must have exactly one parent");
  const parentSha = sha(candidate.parents[0], "candidate parent SHA required");
  const headRepository = object(run.head_repository, "Release head repository required");
  const expected: Json = {
    conclusion: "failure",
    event: "workflow_dispatch",
    head_branch: "main",
    head_repository: repository,
    head_sha: parentSha,
    id: request.releaseRunId,
    name: "Release",
    path: workflowPath,
    status: "completed",
  };
  const actual: Json = {
    conclusion: run.conclusion,
    event: run.event,
    head_branch: run.head_branch,
    head_repository: headRepository.full_name,
    head_sha: run.head_sha,
    id: positiveId(run.id, "Release run numeric ID required"),
    name: run.name,
    path: run.path,
    status: run.status,
  };
  const mismatches = Object.keys(expected).filter((key) => actual[key] !== expected[key]);
  requireValue(mismatches.length === 0, `Release run identity differs: ${mismatches.join(", ")}`);
  return { candidateSha: candidate.sha, parentSha };
}

interface Job {
  conclusion?: unknown;
  name?: unknown;
  status?: unknown;
}

export function validateReleaseJobs(jobs: readonly Job[]): void {
  const required = new Map([
    ["Prepare the release candidate", "success"],
    ["Build npm packages", "success"],
    ["Smoke package tarballs", "success"],
    ["Tag the release", "success"],
    ["Admit the immutable release publication", "success"],
    ["Publish npm packages", "failure"],
  ]);
  for (const [name, conclusion] of required) {
    const matches = jobs.filter((job) => job.name === name);
    requireValue(matches.length === 1, `Release run must contain exactly one ${name} job`);
    requireValue(
      matches[0]?.status === "completed" && matches[0]?.conclusion === conclusion,
      `${name} must be completed/${conclusion}`,
    );
  }
}

interface Artifact {
  digest?: unknown;
  expired?: unknown;
  id?: unknown;
  name?: unknown;
  workflow_run?: unknown;
}

export interface ReleaseArtifact {
  digest: string;
  id: string;
}

export function validateArtifactInventory(
  artifacts: readonly Artifact[],
  request: BootstrapRequest,
  parentSha: string,
): ReleaseArtifact {
  const matches = artifacts.filter((artifact) => artifact.name === artifactName);
  requireValue(matches.length === 1, "Release run must have exactly one npm-packages artifact");
  const artifact = matches[0]!;
  requireValue(artifact.expired === false, "npm-packages artifact must not be expired");
  const id = positiveId(artifact.id, "npm-packages artifact ID required");
  requireValue(
    typeof artifact.digest === "string" && /^sha256:[0-9a-f]{64}$/.test(artifact.digest),
    "npm-packages artifact SHA-256 digest required",
  );
  const source = object(artifact.workflow_run, "npm-packages workflow identity required");
  requireValue(
    positiveId(source.id, "npm-packages workflow run ID required") === request.releaseRunId &&
      source.head_branch === "main" &&
      source.head_sha === parentSha,
    "npm-packages artifact is not bound to the exact Release run",
  );
  return { digest: artifact.digest, id };
}

function parseManifest(source: string): Json {
  try {
    return object(JSON.parse(source) as unknown, "package manifest object required");
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(`invalid package manifest: ${detail}`, { cause: error });
  }
}

function workspaceVersion(cargoToml: string): string {
  const lines = cargoToml.split(/\r?\n/);
  const start = lines.findIndex((line) => line.trim() === "[workspace.package]");
  requireValue(start >= 0, "workspace package section required");
  const versions: string[] = [];
  for (const line of lines.slice(start + 1)) {
    if (line.trim().startsWith("[")) break;
    const match = /^version\s*=\s*"([^"]+)"\s*$/.exec(line.trim());
    if (match !== null) versions.push(match[1]!);
  }
  requireValue(versions.length === 1, "one workspace package version required");
  const version = versions[0]!;
  requireValue(strictVersion.test(version), "strict workspace package version required");
  return version;
}

export function validatePackageManifest(
  request: BootstrapRequest,
  packageManifest: string,
  cargoToml?: string,
  prepared = false,
): Json {
  const manifest = parseManifest(packageManifest);
  requireValue(manifest.name === request.packageName, "package manifest name differs");
  requireValue(manifest.version === request.version, "package manifest version differs");
  const publishConfig = object(manifest.publishConfig, "public publishConfig required");
  requireValue(publishConfig.access === "public", "package must publish with public access");
  if (cargoToml !== undefined) {
    requireValue(workspaceVersion(cargoToml) === request.version, "workspace version differs");
  }
  if (prepared) {
    requireValue(
      !/"(?:workspace|catalog):/.test(JSON.stringify(manifest)),
      "prepared package manifest contains unresolved protocols",
    );
  }
  return manifest;
}

export function validateRegistryStatus(packageName: string, status: number): void {
  if (status === 404) return;
  if (status === 200) {
    throw new Error(`${packageName} already exists on npm; first-publish bootstrap refused`);
  }
  throw new Error(`npm registry returned HTTP ${status} for ${packageName}`);
}

function runGit(args: readonly string[]): string {
  const result = spawnSync("git", args, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    timeout: 30_000,
  });
  if (result.error !== undefined) throw result.error;
  requireValue(result.status === 0, `git ${args.join(" ")} failed: ${result.stderr.trim()}`);
  return result.stdout.trim();
}

async function github(pathname: string, token: string): Promise<Json> {
  const response = await fetch(`https://api.github.com/repos/${repository}/${pathname}`, {
    headers: {
      accept: "application/vnd.github+json",
      authorization: `Bearer ${token}`,
      "user-agent": "vize-npm-bootstrap",
      "x-github-api-version": "2022-11-28",
    },
    signal: AbortSignal.timeout(30_000),
  });
  requireValue(response.ok, `GitHub GET ${pathname} returned HTTP ${response.status}`);
  return object(await response.json(), `GitHub GET ${pathname} returned a non-object`);
}

async function githubCollection(
  pathname: string,
  collection: string,
  token: string,
): Promise<Json[]> {
  const values: Json[] = [];
  let expected: number | null = null;
  for (let page = 1; ; page += 1) {
    const separator = pathname.includes("?") ? "&" : "?";
    const response = await github(`${pathname}${separator}per_page=100&page=${page}`, token);
    const rows = response[collection];
    requireValue(Array.isArray(rows), `GitHub ${collection} collection required`);
    if (expected === null) {
      requireValue(
        Number.isSafeInteger(response.total_count) && Number(response.total_count) >= 0,
        `GitHub ${collection} total_count required`,
      );
      expected = Number(response.total_count);
    }
    values.push(...rows.map((value) => object(value, `GitHub ${collection} row required`)));
    if (rows.length < 100) break;
  }
  requireValue(values.length === expected, `GitHub ${collection} pagination differs`);
  return values;
}

async function assertUnpublished(packageName: string): Promise<void> {
  const response = await fetch(`https://registry.npmjs.org/${encodeURIComponent(packageName)}`, {
    headers: {
      accept: "application/vnd.npm.install-v1+json",
      "user-agent": "vize-npm-bootstrap",
    },
    signal: AbortSignal.timeout(30_000),
  });
  validateRegistryStatus(packageName, response.status);
}

function requestFromEnvironment(env: NodeJS.ProcessEnv): BootstrapRequest {
  requireValue(env.GITHUB_REPOSITORY === repository, `repository must be ${repository}`);
  return validateRequest({
    packagePath: env.BOOTSTRAP_PACKAGE_PATH ?? "",
    releaseRunId: env.RELEASE_RUN_ID ?? "",
    tagName: env.RELEASE_TAG_NAME ?? "",
    version: env.RELEASE_VERSION ?? "",
    workflowRef: env.GITHUB_REF ?? "",
  });
}

function appendOutputs(outputs: Readonly<Record<string, string>>): void {
  const destination = process.env.GITHUB_OUTPUT;
  requireValue(destination !== undefined && destination !== "", "GITHUB_OUTPUT required");
  fs.appendFileSync(
    destination,
    Object.entries(outputs)
      .map(([name, value]) => `${name}=${value}\n`)
      .join(""),
  );
}

async function preflight(env: NodeJS.ProcessEnv): Promise<void> {
  const request = requestFromEnvironment(env);
  const token = env.GITHUB_TOKEN ?? "";
  requireValue(token !== "", "GitHub token required for read-only release evidence");
  runGit([
    "fetch",
    "--quiet",
    "--no-tags",
    "origin",
    `+refs/tags/${request.tagName}:refs/tags/${request.tagName}`,
  ]);
  const candidateSha = runGit(["rev-parse", `refs/tags/${request.tagName}^{commit}`]);
  const commit = runGit(["rev-list", "--parents", "-n", "1", candidateSha]).split(" ");
  requireValue(commit[0] === candidateSha, "candidate commit identity differs");
  const run = await github(`actions/runs/${request.releaseRunId}`, token);
  const candidate = validateCandidate(
    request,
    { sha: candidateSha, parents: commit.slice(1) },
    run,
  );
  const cargoToml = runGit(["show", `${candidateSha}:Cargo.toml`]);
  const manifest = runGit(["show", `${candidateSha}:${request.packagePath}/package.json`]);
  validatePackageManifest(request, manifest, cargoToml);
  const jobs = await githubCollection(
    `actions/runs/${request.releaseRunId}/jobs?filter=latest`,
    "jobs",
    token,
  );
  validateReleaseJobs(jobs);
  const artifacts = await githubCollection(
    `actions/runs/${request.releaseRunId}/artifacts`,
    "artifacts",
    token,
  );
  const artifact = validateArtifactInventory(artifacts, request, candidate.parentSha);
  await assertUnpublished(request.packageName);

  const stageRoot = env.BOOTSTRAP_STAGE_ROOT ?? "";
  requireValue(stageRoot !== "", "BOOTSTRAP_STAGE_ROOT required");
  fs.mkdirSync(stageRoot);
  fs.writeFileSync(
    path.join(stageRoot, "pnpm-workspace.yaml"),
    `${runGit(["show", `${candidateSha}:pnpm-workspace.yaml`])}\n`,
    { flag: "wx" },
  );
  appendOutputs({
    artifact_digest: artifact.digest,
    artifact_id: artifact.id,
    candidate_sha: candidate.candidateSha,
    package_name: request.packageName,
    package_relative_path: request.packagePath.slice("npm/".length),
    parent_sha: candidate.parentSha,
  });
}

function packageRoot(env: NodeJS.ProcessEnv, request: BootstrapRequest): string {
  const stageRoot = env.BOOTSTRAP_STAGE_ROOT ?? "";
  requireValue(stageRoot !== "", "BOOTSTRAP_STAGE_ROOT required");
  const npmRoot = path.resolve(stageRoot, "npm");
  const packageRoot = path.resolve(npmRoot, request.packagePath.slice("npm/".length));
  requireValue(
    packageRoot.startsWith(`${npmRoot}${path.sep}`),
    "package path escapes artifact root",
  );
  return packageRoot;
}

function validateDownloaded(env: NodeJS.ProcessEnv, prepared: boolean): void {
  const request = requestFromEnvironment(env);
  const root = packageRoot(env, request);
  const manifestPath = path.join(root, "package.json");
  const stat = fs.lstatSync(manifestPath);
  requireValue(
    stat.isFile() && !stat.isSymbolicLink(),
    "artifact package.json must be a regular file",
  );
  const manifest = fs.readFileSync(manifestPath, "utf8");
  validatePackageManifest(request, manifest, undefined, prepared);
  if (!prepared) {
    appendOutputs({
      original_manifest_sha512: createHash("sha512").update(manifest).digest("hex"),
      package_root: root,
    });
  }
}

async function main(): Promise<void> {
  const [command, extra] = process.argv.slice(2);
  requireValue(extra === undefined, "one npm bootstrap policy command required");
  if (command === "preflight") return preflight(process.env);
  if (command === "artifact") return validateDownloaded(process.env, false);
  if (command === "prepared") return validateDownloaded(process.env, true);
  if (command === "registry") {
    const request = requestFromEnvironment(process.env);
    return assertUnpublished(request.packageName);
  }
  throw new Error("usage: policy.ts <preflight|artifact|prepared|registry>");
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  });
}
