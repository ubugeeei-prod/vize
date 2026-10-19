// Stamps one release version into every versioned manifest of a checkout.
// The Release workflow commits the result only onto its tag, never onto main.
import { println, writeStderr } from "../command_io.ts";
import {
  isDir,
  joinPath,
  pathExists,
  readDir,
  readFileToString,
  tryWriteStringToFile,
} from "../command_runtime.ts";
import {
  extraReleasePackageJsonPaths,
  replaceFirstVersionLine,
  replacePackageJsonVersion,
} from "./preparation.ts";
import { releaseReadmePaths, replaceReadmeVersionReferences } from "./readmes.ts";
import { updateWorkspaceManifestVersions } from "./versions.ts";

/** Lockfile packages whose version is the workspace version. */
function isOwnedCrate(name: string): boolean {
  return (
    name === "vize" ||
    name.startsWith("vize_") ||
    name === "davinci_harness" ||
    name === "davinci_test_support" ||
    name === "vize-zed-extension"
  );
}

/** Rewrites the `version` line of every owned `[[package]]` in a Cargo.lock. */
export function stampCargoLock(content: string, current: string, next: string): string {
  let packageName = "";
  return content
    .split("\n")
    .map((line) => {
      const trimmed = line.trim();
      if (trimmed === "[[package]]") packageName = "";
      if (trimmed.startsWith('name = "') && trimmed.endsWith('"')) {
        packageName = trimmed.slice('name = "'.length, -1);
      }
      return isOwnedCrate(packageName) && trimmed === `version = "${current}"`
        ? line.replace(current, next)
        : line;
    })
    .join("\n");
}

type Rewrite = (content: string, current: string, next: string) => string;

/** Files the stamp rewrites, each with the transform that owns its format. */
async function stampTargets(): Promise<Array<[string, Rewrite, boolean]>> {
  const targets: Array<[string, Rewrite, boolean]> = [
    ["Cargo.toml", updateWorkspaceManifestVersions, true],
    ["Cargo.lock", stampCargoLock, true],
    ["editors/zed/Cargo.toml", replaceFirstVersionLine, false],
    ["editors/zed/extension.toml", replaceFirstVersionLine, false],
    ["editors/zed/Cargo.lock", stampCargoLock, false],
  ];
  for (const entry of readDir("npm")) {
    const packageJson = joinPath(joinPath("npm", entry), "package.json");
    if (isDir(joinPath("npm", entry)) && pathExists(packageJson)) {
      targets.push([packageJson, replacePackageJsonVersion, false]);
    }
  }
  for (const packageJson of await extraReleasePackageJsonPaths()) {
    if (pathExists(packageJson)) targets.push([packageJson, replacePackageJsonVersion, false]);
  }
  for (const readme of await releaseReadmePaths()) {
    targets.push([readme, replaceReadmeVersionReferences, false]);
  }
  return targets;
}

/**
 * Replaces `current` with `next` in the working tree. Required files must
 * change; the npm CLI manifest must end up at `next` as a final consistency check.
 */
export async function stampVersion(current: string, next: string): Promise<boolean> {
  for (const [path, rewrite, required] of await stampTargets()) {
    const before = readFileToString(path);
    if (before === null) {
      if (!required) continue;
      writeStderr(`Failed to read ${path}\n`);
      return false;
    }
    const after = rewrite(before, current, next);
    if (after === before) {
      if (required) {
        writeStderr(`${path} does not contain version ${current}\n`);
        return false;
      }
      continue;
    }
    if (!tryWriteStringToFile(path, after)) {
      writeStderr(`Failed to write ${path}\n`);
      return false;
    }
    println(`  stamped ${path}`);
  }
  const cli = readFileToString("npm/cli/package.json");
  if (cli === null || !cli.includes(`"version": "${next}"`)) {
    writeStderr(`npm/cli/package.json was not stamped with ${next}\n`);
    return false;
  }
  return true;
}
