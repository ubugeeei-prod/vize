import { isDir, joinPath, pathExists, readDir, replaceEvery } from "../command_runtime.ts";
import { nestedNpmPackageDirs } from "./preparation.ts";
import { isGitIgnored } from "./versions.ts";

/**
 * A published benchmark is evidence about the measured binary, not the
 * next release. Keep marked publication sections byte-for-byte unchanged.
 */
export function replaceReadmeVersionReferences(
  content: string,
  currentVersion: string,
  newVersion: string,
): string {
  const lines: string[] = [];
  let benchmark = false;
  for (const line of content.split("\n")) {
    if (line.includes("<!-- benchmark:readme:start -->")) {
      benchmark = true;
    }
    lines.push(benchmark ? line : replaceEvery(line, currentVersion, newVersion));
    if (line.includes("<!-- benchmark:readme:end -->")) {
      benchmark = false;
    }
  }
  return lines.join("\n");
}

/** The root README plus every tracked npm package README. */
export async function releaseReadmePaths(): Promise<string[]> {
  const packageDirs = readDir("npm")
    .map((entry) => joinPath("npm", entry))
    .filter((dir) => isDir(dir));
  packageDirs.push(...(await nestedNpmPackageDirs()));
  const paths: string[] = [];
  for (const readme of ["README.md", ...packageDirs.map((dir) => joinPath(dir, "README.md"))]) {
    if (pathExists(readme) && !(await isGitIgnored(readme))) paths.push(readme);
  }
  return paths;
}
