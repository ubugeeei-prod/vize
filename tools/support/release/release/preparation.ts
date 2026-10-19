import {
  isDir,
  joinPath,
  lexicalCompare,
  moonTrim,
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
