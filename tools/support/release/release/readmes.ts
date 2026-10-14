import { println, writeStderr } from "../command_io.ts";
import {
  isDir,
  joinPath,
  pathExists,
  readDir,
  readFileToString,
  replaceEvery,
  tryWriteStringToFile,
} from "../command_runtime.ts";
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

export async function updateReleaseReadmes(
  currentVersion: string,
  newVersion: string,
  filesToStage: string[],
): Promise<boolean> {
  println("Updating READMEs...");
  const rootReadme = "README.md";
  if (pathExists(rootReadme) && !(await isGitIgnored(rootReadme))) {
    const content = readFileToString(rootReadme);
    if (content === null) {
      writeStderr(`Failed to read ${rootReadme}\n`);
      return false;
    }
    if (
      !tryWriteStringToFile(
        rootReadme,
        replaceReadmeVersionReferences(content, currentVersion, newVersion),
      )
    ) {
      writeStderr(`Failed to update ${rootReadme}\n`);
      return false;
    }
    filesToStage.push(rootReadme);
  }
  const readmePackageDirs: string[] = [];
  for (const entryName of readDir("npm")) {
    const packageDir = joinPath("npm", entryName);
    if (!isDir(packageDir)) continue;
    readmePackageDirs.push(packageDir);
  }
  readmePackageDirs.push(...(await nestedNpmPackageDirs()));
  for (const packageDir of readmePackageDirs) {
    const readmePath = joinPath(packageDir, "README.md");
    if (!pathExists(readmePath)) continue;
    if (await isGitIgnored(readmePath)) continue;
    const content = readFileToString(readmePath);
    if (content === null) {
      writeStderr(`Failed to read ${readmePath}\n`);
      return false;
    }
    if (
      !tryWriteStringToFile(
        readmePath,
        replaceReadmeVersionReferences(content, currentVersion, newVersion),
      )
    ) {
      writeStderr(`Failed to update ${readmePath}\n`);
      return false;
    }
    filesToStage.push(readmePath);
  }
  return true;
}
