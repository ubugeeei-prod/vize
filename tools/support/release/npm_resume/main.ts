import { appendFileSync } from "node:fs";
import { collectStdout, println, run, runMain, writeStderr } from "../command_io.ts";
import { cliArgs, nonEmptyEnv, readFileToString } from "../command_runtime.ts";
import { npmRegistryPackagePath } from "../publish_npm_package/registry.ts";
import {
  backfillTag,
  higherStableVersions,
  validateResumePolicy,
  type ReleaseRunFacts,
} from "./policy.ts";

interface GitHubRun {
  event?: unknown;
  status?: unknown;
  conclusion?: unknown;
  head_sha?: unknown;
  head_branch?: unknown;
  path?: unknown;
  repository?: { full_name?: unknown };
}

interface GitHubArtifacts {
  artifacts?: Array<{ name?: unknown; expired?: unknown }>;
}

interface PackageManifest {
  name?: unknown;
  napi?: { targets?: unknown };
  optionalDependencies?: unknown;
}

function fail(message: string): number {
  writeStderr(`${message}\n`);
  return 1;
}

function parseManifest(path: string): PackageManifest | null {
  const content = readFileToString(path);
  if (content === null) return null;
  try {
    return JSON.parse(content) as PackageManifest;
  } catch {
    return null;
  }
}

function stringArray(value: unknown): string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string") ? value : [];
}

function releasePackageNames(): string[] | null {
  const native = parseManifest("npm/native/package.json");
  if (native === null || typeof native.name !== "string") return null;
  const optional = native.optionalDependencies;
  if (optional === null || typeof optional !== "object" || Array.isArray(optional)) return null;
  const dirs = [
    "npm/fresco-native",
    "npm/wasm",
    ...(nonEmptyEnv("RELEASE_JS_PACKAGES") ?? "").split(" "),
  ].filter((value) => value !== "");
  const names = [native.name, ...Object.keys(optional)];
  for (const dir of dirs) {
    const manifest = parseManifest(`${dir}/package.json`);
    if (manifest === null || typeof manifest.name !== "string") return null;
    if (!names.includes(manifest.name)) names.push(manifest.name);
  }
  return names;
}

function requiredArtifacts(): string[] | null {
  const native = parseManifest("npm/native/package.json");
  const fresco = parseManifest("npm/fresco-native/package.json");
  const nativeTargets = stringArray(native?.napi?.targets);
  const frescoTargets = stringArray(fresco?.napi?.targets);
  if (nativeTargets.length === 0 || frescoTargets.length === 0) return null;
  return [
    "npm-packages",
    "npm-package-wasm",
    ...nativeTargets.map((target) => `bindings-${target}`),
    ...frescoTargets.map((target) => `fresco-bindings-${target}`),
  ];
}

async function githubJson<T>(path: string): Promise<T | null> {
  const api = nonEmptyEnv("GITHUB_API_URL") ?? "https://api.github.com";
  const token = nonEmptyEnv("GITHUB_TOKEN");
  try {
    const response = await fetch(`${api}${path}`, {
      headers: {
        accept: "application/vnd.github+json",
        ...(token === undefined ? {} : { authorization: `Bearer ${token}` }),
        "x-github-api-version": "2022-11-28",
      },
    });
    return response.ok ? ((await response.json()) as T) : null;
  } catch {
    return null;
  }
}

function remoteReleaseVersions(output: string): string[] {
  return output
    .split("\n")
    .map((line) => /refs\/tags\/v(\d+\.\d+\.\d+)$/.exec(line)?.[1])
    .filter((value): value is string => value !== undefined);
}

async function visiblePackage(name: string, version: string): Promise<boolean> {
  const runId = nonEmptyEnv("GITHUB_RUN_ID") ?? "resume";
  const url = `https://registry.npmjs.org/${npmRegistryPackagePath(name)}/${version}?vize-resume-check=${runId}`;
  try {
    const response = await fetch(url, {
      headers: { accept: "application/json", "cache-control": "no-cache" },
    });
    if (!response.ok) return false;
    const body = (await response.json()) as { name?: unknown; version?: unknown };
    return body.name === name && body.version === version;
  } catch {
    return false;
  }
}

async function npmResume(version: string, runId: string): Promise<number> {
  if (!/^\d+\.\d+\.\d+$/.test(version) || !/^[1-9]\d*$/.test(runId)) {
    return fail("Usage: npm_resume/main.ts <stable-version> <failed-release-run-id>");
  }
  if ((nonEmptyEnv("RELEASE_SOURCE_INPUT") ?? "") !== "") {
    return fail("source cannot be combined with npm resume");
  }
  const repository = nonEmptyEnv("GITHUB_REPOSITORY");
  if (repository === undefined) return fail("GITHUB_REPOSITORY is required");
  const [runData, artifactData] = await Promise.all([
    githubJson<GitHubRun>(`/repos/${repository}/actions/runs/${runId}`),
    githubJson<GitHubArtifacts>(
      `/repos/${repository}/actions/runs/${runId}/artifacts?per_page=100`,
    ),
  ]);
  if (runData === null || artifactData === null) return fail("Failed to read the release run");

  const tag = `v${version}`;
  if ((await run("git", ["fetch", "--no-tags", "--depth=2", "origin", `refs/tags/${tag}`])) !== 0)
    return fail(`Failed to fetch ${tag}`);
  const tagSha = (await collectStdout("git", ["rev-parse", "FETCH_HEAD"])).stdout.trim();
  const tagParent = (await collectStdout("git", ["rev-parse", "FETCH_HEAD^"])).stdout.trim();
  if (!/^[0-9a-f]{40}$/.test(tagSha) || !/^[0-9a-f]{40}$/.test(tagParent))
    return fail(`Failed to resolve ${tag}`);

  const tags = await collectStdout("git", ["ls-remote", "--tags", "--refs", "origin", "v*"]);
  if (tags.status !== 0) return fail("Failed to read release tags");
  const higherVersions = higherStableVersions(version, remoteReleaseVersions(tags.stdout));
  const packageNames = releasePackageNames();
  const artifacts = requiredArtifacts();
  if (packageNames === null || artifacts === null) return fail("Failed to read release manifests");

  const incompleteHigherPackages: string[] = [];
  for (const higher of higherVersions) {
    const visible = await Promise.all(packageNames.map((name) => visiblePackage(name, higher)));
    for (let index = 0; index < packageNames.length; index++) {
      if (!visible[index]) incompleteHigherPackages.push(`${packageNames[index]}@${higher}`);
    }
  }

  const runFacts: ReleaseRunFacts = {
    repository:
      typeof runData.repository?.full_name === "string" ? runData.repository.full_name : "",
    workflowPath: typeof runData.path === "string" ? runData.path : "",
    event: typeof runData.event === "string" ? runData.event : "",
    branch: typeof runData.head_branch === "string" ? runData.head_branch : "",
    status: typeof runData.status === "string" ? runData.status : "",
    conclusion: typeof runData.conclusion === "string" ? runData.conclusion : null,
    headSha: typeof runData.head_sha === "string" ? runData.head_sha : "",
  };
  const artifactNames = (artifactData.artifacts ?? [])
    .filter((artifact) => artifact.expired === false && typeof artifact.name === "string")
    .map((artifact) => artifact.name as string);
  const errors = validateResumePolicy({
    repository,
    run: runFacts,
    tagParent,
    artifactNames,
    requiredArtifacts: artifacts,
    targetVersion: version,
    higherVersions,
    incompleteHigherPackages,
  });
  if (errors.length > 0) return fail(errors.join("\n"));

  const publishTag = higherVersions.length === 0 ? "" : backfillTag(version);
  const output = nonEmptyEnv("GITHUB_OUTPUT");
  if (output === undefined) return fail("GITHUB_OUTPUT is required");
  appendFileSync(
    output,
    [
      `version=${version}`,
      `tag=${tag}`,
      `sha=${tagSha}`,
      `publish_tag=${publishTag}`,
      `allow_newer_tag=${higherVersions.length === 0 ? "" : "true"}`,
    ].join("\n") + "\n",
  );
  println(
    higherVersions.length === 0
      ? `Resuming current release ${tag} from run ${runId}`
      : `Backfilling ${tag} from run ${runId} without moving latest beyond ${higherVersions.at(-1)}`,
  );
  return 0;
}

if (import.meta.main) runMain(() => npmResume(cliArgs()[0] ?? "", cliArgs()[1] ?? ""));
