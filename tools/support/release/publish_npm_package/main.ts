// Publishes one npm package with rerun-safe registry visibility checks.
import { collectOutput, println, run, runMain, sleepSeconds, writeStderr } from "../command_io.ts";
import {
  cliArgs,
  dirnamePath,
  joinPath,
  moonTrim,
  nonEmptyEnv,
  parseMoonInt,
  pathExists,
  readFileToString,
  replaceEvery,
  resolvePath,
  toolBin,
} from "../command_runtime.ts";
import { isJsonObject, parseJson, stringMember, type JsonValue } from "../ordered_json.ts";
import { npmVersionIsVisible, waitForRegistryVisibility, type NpmPackageInfo } from "./registry.ts";
import { isStagedPublishError } from "./staged_error.ts";

function normalizeManifestPath(value: string): string | null {
  const manifestPath = moonTrim(value);
  if (manifestPath === "") return null;
  if (
    manifestPath.startsWith("http://") ||
    manifestPath.startsWith("https://") ||
    manifestPath.startsWith("#")
  ) {
    return null;
  }
  return manifestPath;
}

function pushUniqueString(items: string[], value: string): void {
  if (!items.includes(value)) items.push(value);
}

function collectManifestPaths(value: JsonValue, paths: string[]): void {
  if (typeof value === "string") {
    const manifestPath = normalizeManifestPath(value);
    if (manifestPath !== null) pushUniqueString(paths, manifestPath);
  } else if (Array.isArray(value)) {
    for (const item of value) collectManifestPaths(item, paths);
  } else if (isJsonObject(value)) {
    for (const child of value.members.values()) collectManifestPaths(child, paths);
  }
}

export function validatePublishEntrypoints(directory: string, packageJson: JsonValue): string[] {
  const missing: string[] = [];
  if (!isJsonObject(packageJson)) {
    missing.push("package.json root is not an object");
    return missing;
  }
  const manifestPaths: string[] = [];
  for (const field of ["main", "types", "bin", "exports"]) {
    const value = packageJson.get(field);
    if (value !== undefined) collectManifestPaths(value, manifestPaths);
  }
  for (const manifestPath of manifestPaths) {
    if (!pathExists(joinPath(directory, manifestPath))) {
      pushUniqueString(missing, manifestPath);
    }
  }
  return missing;
}

function findRepoRoot(start: string): string | null {
  const currentPath = resolvePath(start);
  if (
    pathExists(joinPath(currentPath, "pnpm-workspace.yaml")) &&
    pathExists(joinPath(currentPath, "npm"))
  ) {
    return currentPath;
  }
  const parent = dirnamePath(start);
  return parent === currentPath ? null : findRepoRoot(parent);
}

/** Reads and parses a `package.json`, returning `undefined` on failure. */
function readPackageJson(target: string): JsonValue | undefined {
  const content = readFileToString(target);
  return content === null ? undefined : parseJson(content);
}

function nonEmptyMember(value: JsonValue, key: string): string | null {
  const member = stringMember(value, key);
  return member === undefined || member === "" ? null : member;
}

/** Parses a positive integer and falls back on invalid input. */
function positiveIntOr(value: string, fallback: number): number {
  const parsed = parseMoonInt(value);
  return parsed !== null && parsed > 0 ? parsed : fallback;
}

function envPositiveInt(name: string, fallback: number): number {
  const value = nonEmptyEnv(name);
  return value === undefined ? fallback : positiveIntOr(value, fallback);
}

/** Derives the npm dist-tag for a package version. */
export function npmTag(version: string): string {
  const configured = nonEmptyEnv("NPM_TAG");
  if (configured !== undefined) return configured;
  if (version.includes("-alpha")) return "alpha";
  if (version.includes("-beta")) return "beta";
  if (version.includes("-rc")) return "rc";
  return "latest";
}

function shellQuote(value: string): string {
  return `'${replaceEvery(value, "'", `'"'"'`)}'`;
}

/** Builds the shell command line that runs a repository TypeScript command. */
export function childScriptCommand(
  nodeBin: string,
  scriptPath: string,
  args: readonly string[],
): string {
  return [shellQuote(nodeBin), shellQuote(scriptPath), ...args.map(shellQuote)].join(" ");
}

/**
 * Resolves workspace and catalog dependency protocols before publishing.
 * Release artifacts are published from downloaded directories rather than a
 * fully installed workspace, so we proactively rewrite protocol-based version
 * specs to concrete npm-compatible strings. This keeps the publish path
 * deterministic and removes a class of release-only pnpm failures.
 */
async function preparePublishManifest(directory: string): Promise<number> {
  const configuredRoot = nonEmptyEnv("VIZE_REPO_ROOT");
  const startDir = resolvePath(configuredRoot ?? process.cwd());
  const repoRoot = findRepoRoot(startDir);
  if (repoRoot === null) return 0;
  const nodeBin = toolBin("NODE_BIN", process.execPath);
  const scriptPath = joinPath(
    repoRoot,
    "tools/support/release/prepare_npm_publish_manifest/main.ts",
  );
  const exitCode = await run("sh", ["-c", childScriptCommand(nodeBin, scriptPath, [directory])], {
    cwd: repoRoot,
  });
  if (exitCode !== 0) {
    writeStderr(`Failed to prepare npm publish manifest for ${directory}\n`);
  }
  return exitCode;
}

const USAGE =
  "Usage: node tools/support/release/publish_npm_package/main.ts <dir> [--provenance] [--no-wait] [--prepared] [--ignore-failure] [--retries N] [--delay N]\n";

/**
 * Main entry point for resilient single-package npm publishing.
 * The function first decides whether publishing is necessary, then performs
 * retries around `vp pm publish`, and finally waits for registry propagation
 * unless `--no-wait` leaves that to a single post-publish verification.
 * `--ignore-failure` only affects the final hard failure after all retries and
 * visibility checks have been exhausted.
 */
export async function publishNpmPackage(args: readonly string[]): Promise<number> {
  if (args.length === 0) {
    writeStderr(USAGE);
    return 1;
  }

  const directory = resolvePath(args[0]);
  const vpBin = toolBin("VP_BIN", "vp");
  let retries = envPositiveInt("PUBLISH_RETRY_LIMIT", 3);
  let delaySeconds = envPositiveInt("PUBLISH_RETRY_DELAY", 15);
  const resolutionRetries = envPositiveInt("PUBLISH_RESOLUTION_RETRY_LIMIT", retries);
  const resolutionDelaySeconds = envPositiveInt("PUBLISH_RESOLUTION_RETRY_DELAY", delaySeconds);
  let provenance = false;
  let ignoreFailure = false;
  let waitVisible = true;
  let prepared = false;
  let index = 1;
  while (index < args.length) {
    const arg = args[index];
    if (arg === "--provenance") {
      provenance = true;
      index += 1;
      continue;
    }
    if (arg === "--no-wait") {
      waitVisible = false;
      index += 1;
      continue;
    }
    if (arg === "--prepared") {
      prepared = true;
      index += 1;
      continue;
    }
    if (arg === "--ignore-failure") {
      ignoreFailure = true;
      index += 1;
      continue;
    }
    if (arg === "--retries" && index + 1 < args.length) {
      retries = positiveIntOr(args[index + 1], retries);
      index += 2;
      continue;
    }
    if (arg === "--delay" && index + 1 < args.length) {
      delaySeconds = positiveIntOr(args[index + 1], delaySeconds);
      index += 2;
      continue;
    }
    writeStderr(`Unknown argument: ${arg}\n`);
    return 1;
  }

  const packageJsonPath = joinPath(directory, "package.json");
  // --prepared: the caller already resolved every manifest before publishing.
  const prepareExitCode = prepared ? 0 : await preparePublishManifest(directory);
  if (prepareExitCode !== 0) return prepareExitCode;
  const packageJson = readPackageJson(packageJsonPath);
  if (packageJson === undefined) {
    writeStderr(`Failed to parse ${packageJsonPath}\n`);
    return 1;
  }
  const version = nonEmptyMember(packageJson, "version");
  if (version === null) {
    writeStderr(`Failed to read version from ${packageJsonPath}\n`);
    return 1;
  }
  const name = nonEmptyMember(packageJson, "name");
  if (name === null) {
    writeStderr(`Failed to read name from ${packageJsonPath}\n`);
    return 1;
  }
  const info: NpmPackageInfo = { name, version, tag: npmTag(version) };
  const missingEntrypoints = validatePublishEntrypoints(directory, packageJson);
  if (missingEntrypoints.length > 0) {
    writeStderr(
      `Refusing to publish ${info.name}@${info.version} because package.json points at files that are missing from ${directory}:\n`,
    );
    for (const missingPath of missingEntrypoints) {
      writeStderr(`  - ${missingPath}\n`);
    }
    return 1;
  }
  const publishArgs = ["pm", "publish", "--access", "public", "--no-git-checks", "--tag", info.tag];
  if (provenance) {
    publishArgs.push("--", "--provenance");
  }

  const maxAttempts = retries > 0 ? retries : 1;
  if (await npmVersionIsVisible(info.name, info.version, 0)) {
    println(`${info.name}@${info.version} is already published. Skipping publish.`);
    return waitVisible
      ? waitForRegistryVisibility(info, resolutionRetries, resolutionDelaySeconds, {
          allowNewerTag: true,
        })
      : 0;
  }
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    const commandDisplay = publishArgs.join(" ");
    println(
      `Attempt ${attempt}/${maxAttempts}: publish ${info.name}@${info.version} with ${vpBin} ${commandDisplay}`,
    );
    const publish = await collectOutput(vpBin, publishArgs, { cwd: directory });
    const stdoutText = moonTrim(publish.stdout);
    const stderrText = moonTrim(publish.stderr);
    if (stdoutText !== "") {
      println(stdoutText);
    }
    if (stderrText !== "") {
      writeStderr(`${stderrText}\n`);
    }
    if (publish.status === 0) {
      return waitVisible
        ? waitForRegistryVisibility(info, resolutionRetries, resolutionDelaySeconds)
        : 0;
    }
    if (await npmVersionIsVisible(info.name, info.version, 0 - attempt)) {
      println(
        `${info.name}@${info.version} appears in npm despite a non-zero publish exit. Verifying registry visibility.`,
      );
      return waitForRegistryVisibility(info, resolutionRetries, resolutionDelaySeconds);
    }
    if (isStagedPublishError(`${stdoutText}\n${stderrText}`)) {
      println(
        `${info.name}@${info.version} is staged in npm but not yet visible. Waiting for registry visibility before retrying.`,
      );
      return waitForRegistryVisibility(info, resolutionRetries, resolutionDelaySeconds);
    }
    if (attempt < maxAttempts) {
      println(
        `Publish attempt failed for ${info.name}@${info.version}. Retrying in ${delaySeconds}s...`,
      );
      await sleepSeconds(delaySeconds);
    }
  }

  if (ignoreFailure) {
    println(
      `Failed to publish ${info.name}@${info.version} from ${directory}, but continuing because --ignore-failure is set.`,
    );
    return 0;
  }
  writeStderr(`Failed to publish ${info.name}@${info.version} from ${directory}\n`);
  return 1;
}

if (import.meta.main) {
  runMain(() => publishNpmPackage(cliArgs()));
}
