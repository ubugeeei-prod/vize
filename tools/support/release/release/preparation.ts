import { run, writeStderr } from "../command_io.ts";
import {
  isDir,
  joinPath,
  lexicalCompare,
  moonTrim,
  nonEmptyEnv,
  pathExists,
  readDir,
  readFileToString,
  replaceFirst,
} from "../command_runtime.ts";
import { isGitIgnored } from "./versions.ts";

/** Replaces the first package.json `"version"` field that matches the current release. */
export function replacePackageJsonVersion(
  content: string,
  currentVersion: string,
  newVersion: string,
): string {
  const updated: string[] = [];
  let replaced = false;
  for (const line of content.split("\n")) {
    if (!replaced && moonTrim(line).startsWith('"version"') && line.includes(currentVersion)) {
      updated.push(replaceFirst(line, currentVersion, newVersion));
      replaced = true;
      continue;
    }
    updated.push(line);
  }
  return updated.join("\n");
}

/**
 * Nested npm group directories (npm/builder, npm/framework, npm/compose,
 * npm/ui, ...) hold their packages one level deeper than the flat
 * npm/<package> layout. Scanning them keeps every new group aligned with the
 * release version without editing a hardcoded list — the release preflight
 * verifies all tracked non-private manifests, so any package this scan
 * misses fails the release before publication.
 */
export async function nestedNpmPackageDirs(): Promise<string[]> {
  const packageDirs: string[] = [];
  for (const groupName of readDir("npm")) {
    const groupDir = joinPath("npm", groupName);
    if (!isDir(groupDir)) continue;
    if (pathExists(joinPath(groupDir, "package.json"))) continue;
    for (const entryName of readDir(groupDir)) {
      const packageDir = joinPath(groupDir, entryName);
      if (!isDir(packageDir)) continue;
      const packageJsonPath = joinPath(packageDir, "package.json");
      if (!pathExists(packageJsonPath)) continue;
      // Mirror the release preflight exactly: it verifies tracked,
      // non-private manifests, so ignored build output and private
      // packages stay out of the release version sweep.
      if (await isGitIgnored(packageJsonPath)) continue;
      const content = readFileToString(packageJsonPath);
      if (content === null || content.includes('"private": true')) continue;
      packageDirs.push(packageDir);
    }
  }
  packageDirs.sort(lexicalCompare);
  return packageDirs;
}

/**
 * Package manifests that are intentionally outside the npm workspace package
 * globs but still ship release artifacts and must stay aligned with the
 * workspace version.
 */
export async function extraReleasePackageJsonPaths(): Promise<string[]> {
  const paths = ["editors/vscode/package.json", "editors/vscode-art/package.json"];
  for (const packageDir of await nestedNpmPackageDirs()) {
    paths.push(joinPath(packageDir, "package.json"));
  }
  return paths;
}

/**
 * Replaces the first TOML `version = "..."` line in a file.
 * This is used for the Zed extension manifests where a full TOML parser would
 * be overkill for the repository's simple version synchronization needs.
 */
export function replaceFirstVersionLine(
  content: string,
  currentVersion: string,
  newVersion: string,
): string {
  const updated: string[] = [];
  let replaced = false;
  for (const line of content.split("\n")) {
    if (!replaced) {
      const trimmed = moonTrim(line);
      if (
        trimmed.startsWith('version = "') &&
        trimmed.endsWith('"') &&
        line.includes(currentVersion)
      ) {
        updated.push(replaceFirst(line, currentVersion, newVersion));
        replaced = true;
        continue;
      }
    }
    updated.push(line);
  }
  return updated.join("\n");
}

/**
 * Detects whether the script can safely prompt the user for confirmation.
 * The release script is allowed to be interactive when run locally, but CI and
 * sessions without a controlling terminal must fail explicitly instead of
 * hanging on stdin reads.
 */
export async function hasInteractiveTerminal(): Promise<boolean> {
  return (await run("sh", ["-c", ": </dev/tty >/dev/tty"])) === 0;
}

/** Prompts through the controlling terminal because task runners keep stdin. */
export async function confirmRelease(): Promise<boolean> {
  const command =
    "printf 'Proceed with release? [y/N] ' > /dev/tty; IFS= read -r answer < /dev/tty; case \"$answer\" in y|Y|yes|Yes|YES) exit 0 ;; *) exit 1 ;; esac";
  return (await run("sh", ["-c", command])) === 0;
}

/**
 * Filters a list of candidate paths down to the ones that currently exist.
 * The release commit stages files opportunistically because some package
 * variants are optional in this repository.
 */
export function stageFiles(files: readonly string[]): string[] {
  return files.filter((file) => pathExists(file));
}

/**
 * Runs the shared immutable-tag and current-main guard before any mutation.
 * Tests may override the script path while exercising isolated repositories.
 */
export async function runLocalReleaseGuard(tag: string): Promise<boolean> {
  const script =
    nonEmptyEnv("VIZE_RELEASE_GUARD_SCRIPT") ?? "tools/commands/ci/github/release-local-guard.rs";
  if (!pathExists(script)) {
    writeStderr(`Release preflight script not found: ${script}\n`);
    return false;
  }
  const runner =
    nonEmptyEnv("VIZE_RELEASE_GUARD_RUNNER") ?? (script.endsWith(".mjs") ? "node" : "rust-script");
  if ((await run(runner, [script, tag, "--candidate"])) !== 0) {
    writeStderr("Release preflight failed before repository mutation.\n");
    return false;
  }
  return true;
}
